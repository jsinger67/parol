use std::{collections::HashMap, error::Error, path::Path};

use lsp_server::Message;
use lsp_types::{
    notification::{
        DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, Notification,
        PublishDiagnostics,
    },
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    GotoDefinitionParams, GotoDefinitionResponse, Location, PublishDiagnosticsParams, Range,
    TextDocumentContentChangeEvent, Url,
};
use miette::miette;
use parol::{calculate_lookahead_dfas, check_and_transform_grammar, GrammarConfig, ParolGrammar};

use crate::{diagnostics::Diagnostics, document_state::DocumentState, parol_ls_parser::parse};

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
                    Diagnostics::to_diagnostics(&params.text_document.uri, document_state, err),
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
                    Diagnostics::to_diagnostics(&params.text_document.uri, document_state, err),
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

    pub(crate) fn find_non_terminal_definitions<'a>(
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
