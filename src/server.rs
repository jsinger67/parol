use std::{collections::HashMap, error::Error, path::Path};

use lsp_server::Message;
use lsp_types::{
    notification::{
        DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, Notification,
        PublishDiagnostics,
    },
    Diagnostic, DiagnosticRelatedInformation, DidChangeTextDocumentParams,
    DidCloseTextDocumentParams, DidOpenTextDocumentParams, GotoDefinitionParams,
    GotoDefinitionResponse, Location, NumberOrString, Position, PublishDiagnosticsParams, Range,
    TextDocumentContentChangeEvent, Url,
};
use miette::miette;
use parol::{
    analysis::GrammarAnalysisError, calculate_lookahead_dfas, check_and_transform_grammar,
    GrammarConfig, ParolGrammar,
};

use crate::{
    parol_ls_grammar::ParolLsGrammar, parol_ls_parser::parse, utils::source_code_span_to_range,
};

#[derive(Debug, Default)]
pub(crate) struct DocumentState {
    input: String,
    parsed_data: ParolLsGrammar,
}

impl DocumentState {
    fn ident_at_position(&self, position: Position) -> Option<String> {
        self.parsed_data.ident_at_position(position)
    }
}

#[derive(Debug, Default)]
pub(crate) struct Server {
    documents: HashMap<String, DocumentState>,
}

impl Server {
    // Todo: Make this constant configurable.
    const MAX_K: usize = 3;

    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn analyze(&mut self, uri: &Url) -> miette::Result<()> {
        let file_path = uri
            .to_file_path()
            .map_err(|_| miette!("Failed interpreting file path {}", uri.path()))?;
        let document_state = self.documents.get_mut(uri.path()).unwrap();
        eprintln!("analyze: step 1 - parse");
        parse(
            &document_state.input,
            &file_path,
            &mut document_state.parsed_data,
        )?;
        eprintln!("analyze: step 2 - check_grammar");
        Self::check_grammar(&document_state.input, &file_path)?;
        eprintln!("analyze: finished");
        Ok(())
    }

    pub(crate) fn obtain_grammar_config_from_string(
        input: &str,
        file_name: &Path,
    ) -> miette::Result<GrammarConfig> {
        let mut parol_grammar = ParolGrammar::new();
        parol::parser::parol_parser::parse(input, file_name, &mut parol_grammar)?;
        GrammarConfig::try_from(parol_grammar)
    }

    pub(crate) fn check_grammar(input: &str, file_name: &Path) -> miette::Result<()> {
        let grammar_config = Self::obtain_grammar_config_from_string(input, file_name)?;
        check_and_transform_grammar(&grammar_config.cfg)?;
        calculate_lookahead_dfas(&grammar_config, Self::MAX_K)?;
        Ok(())
    }

    pub(crate) fn handle_open_document(
        &mut self,
        connection: &lsp_server::Connection,
        n: lsp_server::Notification,
    ) -> Result<(), Box<dyn Error>> {
        let params: DidOpenTextDocumentParams = n.extract(DidOpenTextDocument::METHOD)?;
        self.documents.insert(
            params.text_document.uri.path().to_string(),
            DocumentState {
                input: params.text_document.text.clone(),
                ..Default::default()
            },
        );
        match self.analyze(&params.text_document.uri) {
            Ok(()) => {
                eprintln!("handle_open_document: ok");
                let result = PublishDiagnosticsParams::new(
                    params.text_document.uri,
                    vec![],
                    Some(params.text_document.version),
                );
                let params = serde_json::to_value(&result).unwrap();
                let method = <PublishDiagnostics as Notification>::METHOD.to_string();
                connection
                    .sender
                    .send(Message::Notification(lsp_server::Notification {
                        method,
                        params,
                    }))?;
            }
            Err(err) => {
                eprintln!("handle_open_document: error");
                let path = params.text_document.uri.path();
                let document_state = self.documents.get(path).unwrap();
                eprintln!("handle_open_document: document obtained");
                let result = PublishDiagnosticsParams::new(
                    params.text_document.uri.clone(),
                    Self::to_diagnostics(&params.text_document.uri, document_state, err),
                    Some(params.text_document.version),
                );
                let params = serde_json::to_value(&result).unwrap();
                let method = <PublishDiagnostics as Notification>::METHOD.to_string();
                eprintln!("handle_open_document: sending response\n{:?}", params);
                connection
                    .sender
                    .send(Message::Notification(lsp_server::Notification {
                        method,
                        params,
                    }))?;
            }
        }
        Ok(())
    }

    pub(crate) fn handle_change_document(
        &mut self,
        connection: &lsp_server::Connection,
        n: lsp_server::Notification,
    ) -> Result<(), Box<dyn Error>> {
        let params: DidChangeTextDocumentParams = n.extract(DidChangeTextDocument::METHOD)?;
        self.apply_changes(params.text_document.uri.path(), &params.content_changes);
        match self.analyze(&params.text_document.uri) {
            Ok(()) => {
                eprintln!("handle_change_document: ok");
                let result = PublishDiagnosticsParams::new(
                    params.text_document.uri,
                    vec![],
                    Some(params.text_document.version),
                );
                let params = serde_json::to_value(&result).unwrap();
                let method = <PublishDiagnostics as Notification>::METHOD.to_string();
                connection
                    .sender
                    .send(Message::Notification(lsp_server::Notification {
                        method,
                        params,
                    }))?;
            }
            Err(err) => {
                eprintln!("handle_change_document: error");
                let path = params.text_document.uri.path();
                let document_state = self.documents.get(path).unwrap();
                eprintln!("handle_change_document: document obtained");
                let result = PublishDiagnosticsParams::new(
                    params.text_document.uri.clone(),
                    Self::to_diagnostics(&params.text_document.uri, document_state, err),
                    Some(params.text_document.version),
                );
                let params = serde_json::to_value(&result).unwrap();
                let method = <PublishDiagnostics as Notification>::METHOD.to_string();
                eprintln!("handle_change_document: sending response\n{:?}", params);
                connection
                    .sender
                    .send(Message::Notification(lsp_server::Notification {
                        method,
                        params,
                    }))?;
            }
        }
        Ok(())
    }

    pub(crate) fn handle_close_document(
        &mut self,
        n: lsp_server::Notification,
    ) -> Result<(), Box<dyn Error>> {
        let params: DidCloseTextDocumentParams = n.extract(DidCloseTextDocument::METHOD)?;
        self.cleanup(params.text_document.uri.path());
        Ok(())
    }

    pub(crate) fn handle_goto_definition(
        &mut self,
        params: GotoDefinitionParams,
    ) -> GotoDefinitionResponse {
        let document_state = self
            .documents
            .get(
                params
                    .text_document_position_params
                    .text_document
                    .uri
                    .path(),
            )
            .unwrap();
        let mut locations = Vec::new();
        if let Some(text_at_position) =
            document_state.ident_at_position(params.text_document_position_params.position)
        {
            eprintln!("text_at_position: {}", text_at_position);

            // Handle non-terminals here
            Self::add_non_terminal_definitions(
                document_state,
                &text_at_position,
                &mut locations,
                &params,
            );

            // Handle user types here
            Self::find_user_type_definitions(
                document_state,
                text_at_position,
                &mut locations,
                params,
            );
        }
        GotoDefinitionResponse::Array(locations)
    }

    fn cleanup(&mut self, file_path: &str) {
        self.documents.remove(file_path);
    }

    fn apply_changes(
        &mut self,
        file_path: &str,
        content_changes: &[TextDocumentContentChangeEvent],
    ) {
        if let Some(change) = content_changes.last() {
            self.apply_change(file_path, change);
        }
    }

    fn apply_change(&mut self, file_path: &str, change: &TextDocumentContentChangeEvent) {
        self.documents.get_mut(file_path).unwrap().input = change.text.clone();
    }

    fn to_diagnostics(
        uri: &Url,
        document_state: &DocumentState,
        err: miette::ErrReport,
    ) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let parser_err_diag: &dyn miette::Diagnostic = err.as_ref();
        let range = if let Some(mut labels) = parser_err_diag.labels() {
            labels.next().map_or(Range::default(), |src| {
                source_code_span_to_range(&document_state.input, src.inner())
            })
        } else {
            Range::default()
        };
        let source = Some("parol-ls".to_string());
        let code = parser_err_diag
            .code()
            .map(|d| NumberOrString::String(format!("{d}")));
        let mut related_information = vec![];

        // We need to find the correct Display implementation!
        let message = if let Some(e) = err.downcast_ref::<parol_runtime::errors::ParserError>() {
            format!("{err}:\n{e}")
        } else if let Some(e) = err.downcast_ref::<parol_runtime::errors::LookaheadError>() {
            format!("{err}:\n{e}")
        } else if let Some(e) = err.downcast_ref::<GrammarAnalysisError>() {
            match e {
                GrammarAnalysisError::LeftRecursion { recursions } => {
                    for (i, rec) in recursions.iter().enumerate() {
                        related_information.push(DiagnosticRelatedInformation {
                            location: Location {
                                uri: uri.to_owned(),
                                range,
                            },
                            message: format!("Recursion #{}:", i + 1),
                        });
                        for hint in &rec.hints {
                            eprintln!("{}", hint);
                            if let Some(non_terminals) =
                                Self::find_non_terminal_definitions(document_state, &hint.hint)
                            {
                                for rng in non_terminals {
                                    let (range, message) =
                                        (*rng, format!("Non-terminal: {}", hint.hint));
                                    related_information.push(DiagnosticRelatedInformation {
                                        location: Location {
                                            uri: uri.to_owned(),
                                            range,
                                        },
                                        message,
                                    })
                                }
                            } else {
                                let (range, message) = (
                                    Range::default(),
                                    format!("Production reference: {}", hint.hint),
                                );
                                related_information.push(DiagnosticRelatedInformation {
                                    location: Location {
                                        uri: uri.to_owned(),
                                        range,
                                    },
                                    message,
                                })
                            }
                        }
                    }
                }
                GrammarAnalysisError::UnreachableNonTerminals { non_terminals }
                | GrammarAnalysisError::NonProductiveNonTerminals { non_terminals } => {
                    for hint in non_terminals {
                        eprintln!("{}", hint);
                        if let Some(non_terminals) =
                            Self::find_non_terminal_definitions(document_state, &hint.hint)
                        {
                            for rng in non_terminals {
                                let (range, message) =
                                    (*rng, format!("Non-terminal: {}", hint.hint));
                                related_information.push(DiagnosticRelatedInformation {
                                    location: Location {
                                        uri: uri.to_owned(),
                                        range,
                                    },
                                    message,
                                })
                            }
                        }
                    }
                }
                GrammarAnalysisError::MaxKExceeded { max_k: _ } => ()
            }
            format!("{err}")
        } else {
            format!("{err}")
        };
        let diagnostic = Diagnostic {
            source,
            code,
            range,
            message,
            related_information: if related_information.is_empty() {
                None
            } else {
                Some(related_information)
            },
            ..Default::default()
        };
        diagnostics.push(diagnostic);
        diagnostics
    }

    fn find_user_type_definitions(
        document_state: &DocumentState,
        text_at_position: String,
        locations: &mut Vec<Location>,
        params: GotoDefinitionParams,
    ) {
        if let Some(user_type_definitions) = document_state
            .parsed_data
            .user_type_definitions
            .get(&text_at_position)
        {
            for range in user_type_definitions {
                locations.push(Location {
                    uri: params
                        .text_document_position_params
                        .text_document
                        .uri
                        .clone(),
                    range: *range,
                });
            }
        }
    }

    fn find_non_terminal_definitions<'a>(
        document_state: &'a DocumentState,
        non_terminal: &str,
    ) -> Option<&'a Vec<Range>> {
        document_state
            .parsed_data
            .find_non_terminal_definitions(non_terminal)
    }

    fn add_non_terminal_definitions(
        document_state: &DocumentState,
        text_at_position: &str,
        locations: &mut Vec<Location>,
        params: &GotoDefinitionParams,
    ) {
        if let Some(non_terminal_definitions) =
            Self::find_non_terminal_definitions(document_state, text_at_position)
        {
            for range in non_terminal_definitions {
                locations.push(Location {
                    uri: params
                        .text_document_position_params
                        .text_document
                        .uri
                        .clone(),
                    range: *range,
                });
            }
        }
    }
}
