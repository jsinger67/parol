use std::{
    collections::HashMap,
    error::Error,
    path::{Path, PathBuf},
    sync::Arc,
    thread,
};

use lsp_server::Message;
use lsp_types::{
    DidChangeConfigurationParams, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, DocumentFormattingParams, DocumentSymbolParams,
    DocumentSymbolResponse, GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverParams,
    Location, PrepareRenameResponse, PublishDiagnosticsParams, Range, RenameParams,
    TextDocumentContentChangeEvent, TextDocumentPositionParams, TextEdit, Uri, WorkspaceEdit,
    notification::{
        DidChangeConfiguration, DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument,
        Notification, PublishDiagnostics,
    },
};
use parol::{
    GrammarConfig, ParolGrammar, analysis::lalr1_parse_table::calculate_lalr1_parse_table,
    calculate_lookahead_dfas, check_and_transform_grammar, parser::parol_grammar::GrammarType,
};

use crate::{
    config::ConfigProperties, diagnostics::Diagnostics, document_state::DocumentState,
    formatting::FormattingSettings, parol_ls_parser::parse,
};

macro_rules! update_number_option {
    ($self:ident, $props:ident, $( $option_name:ident ).+, $member_name:ident, $default:literal) => {
        if $props.0.contains_key(stringify!($($option_name).+)) {
            $self.$member_name = serde_json::from_value(
                $props
                    .0
                    .get(stringify!($($option_name).+))
                    .unwrap_or(&serde_json::Value::Number(serde_json::Number::from(
                        $default,
                    )))
                    .clone(),
            )?;
            eprintln!(
                concat!(stringify!($($option_name).+), ": {}"),
                $self.$member_name
            );
        }
    };
}

#[derive(Debug, Default)]
pub(crate) struct Server {
    /// Any documents the server has handled, indexed by their URL
    documents: HashMap<Uri, DocumentState>,

    /// Limit for lookahead calculation.
    /// Be careful with high values. The server can get stuck for some grammars.
    max_k: usize,

    /// Aggregated formatting settings
    formatting_settings: FormattingSettings,
}

impl Server {
    pub(crate) fn new(max_k: usize) -> Self {
        Self {
            max_k,
            formatting_settings: FormattingSettings::default(),
            ..Default::default()
        }
    }
    pub(crate) fn update_configuration(
        &mut self,
        props: &ConfigProperties,
    ) -> Result<(), serde_json::error::Error> {
        update_number_option!(self, props, max_k, max_k, 3);
        self.formatting_settings
            .update_from_config_properties(props)?;
        Ok(())
    }

    pub(crate) fn analyze(
        &mut self,
        uri: Uri,
        version: i32,
        connection: Arc<lsp_server::Connection>,
    ) -> anyhow::Result<()> {
        let file_path: PathBuf = PathBuf::from(uri.path().to_string());
        let document_state = self.documents.get_mut(&uri).unwrap();
        eprintln!("analyze: step 1 - parse");
        document_state.clear();
        parse(
            &document_state.input,
            &file_path,
            &mut document_state.parsed_data,
        )?;
        eprintln!("analyze: step 2 - check_grammar");
        let document_state = self.documents.get(&uri).unwrap();
        Self::check_grammar(
            &document_state.input,
            &file_path,
            self.max_k,
            connection,
            uri,
            version,
            document_state.clone(),
        )?;
        eprintln!("analyze: finished");
        Ok(())
    }

    pub(crate) fn obtain_grammar_config_from_string(
        input: &str,
        file_name: &Path,
    ) -> anyhow::Result<GrammarConfig> {
        let mut parol_grammar = ParolGrammar::new();
        parol::parser::parol_parser::parse(input, file_name, &mut parol_grammar)?;
        GrammarConfig::try_from(parol_grammar)
    }

    pub(crate) fn check_grammar(
        input: &str,
        file_name: &Path,
        max_k: usize,
        connection: Arc<lsp_server::Connection>,
        uri: Uri,
        version: i32,
        document_state: DocumentState,
    ) -> anyhow::Result<()> {
        let mut grammar_config = Self::obtain_grammar_config_from_string(input, file_name)?;
        let cfg = check_and_transform_grammar(&grammar_config.cfg, grammar_config.grammar_type)?;
        grammar_config.update_cfg(cfg);
        let grammar_config = grammar_config.clone();
        thread::spawn(move || match grammar_config.grammar_type {
            GrammarType::LLK => {
                if let Err(err) = calculate_lookahead_dfas(&grammar_config, max_k) {
                    eprintln!("check_grammar: errors from calculate_lookahead_dfas");
                    let _ =
                        Self::notify_analysis_error(err, connection, &uri, version, document_state);
                }
            }
            GrammarType::LALR1 => {
                let result = calculate_lalr1_parse_table(&grammar_config);
                match result {
                    Ok((_, resolved_conflicts)) => {
                        let _ = Self::notify_resolved_conflicts(
                            resolved_conflicts,
                            connection,
                            &uri,
                            version,
                        );
                    }
                    Err(err) => {
                        eprintln!("check_grammar: errors from calculate_lookahead_dfas");
                        let _ = Self::notify_analysis_error(
                            err,
                            connection,
                            &uri,
                            version,
                            document_state,
                        );
                    }
                }
            }
        });
        Ok(())
    }

    pub(crate) fn handle_open_document(
        &mut self,
        connection: Arc<lsp_server::Connection>,
        n: lsp_server::Notification,
    ) -> Result<(), Box<dyn Error>> {
        let params: DidOpenTextDocumentParams = n.extract(DidOpenTextDocument::METHOD)?;
        self.documents.insert(
            params.text_document.uri.clone(),
            DocumentState {
                input: params.text_document.text.clone(),
                ..Default::default()
            },
        );
        match self.analyze(
            params.text_document.uri.clone(),
            params.text_document.version,
            connection.clone(),
        ) {
            Ok(()) => {
                eprintln!("handle_open_document: ok");
                Self::notify_analysis_ok(
                    connection,
                    params.text_document.uri,
                    params.text_document.version,
                )?;
            }
            Err(err) => {
                eprintln!("handle_open_document: error");
                let document_state = self
                    .documents
                    .get(&params.text_document.uri)
                    .unwrap()
                    .clone();
                Self::notify_analysis_error(
                    err,
                    connection,
                    &params.text_document.uri,
                    params.text_document.version,
                    document_state,
                )?;
            }
        }
        Ok(())
    }

    pub(crate) fn handle_change_document(
        &mut self,
        connection: Arc<lsp_server::Connection>,
        n: lsp_server::Notification,
    ) -> Result<(), Box<dyn Error>> {
        let params: DidChangeTextDocumentParams = n.extract(DidChangeTextDocument::METHOD)?;
        self.apply_changes(&params.text_document.uri, &params.content_changes);
        match self.analyze(
            params.text_document.uri.clone(),
            params.text_document.version,
            connection.clone(),
        ) {
            Ok(()) => {
                eprintln!("handle_change_document: ok");
                Self::notify_analysis_ok(
                    connection,
                    params.text_document.uri,
                    params.text_document.version,
                )?;
            }
            Err(err) => {
                eprintln!("handle_change_document: error");
                let document_state = self
                    .documents
                    .get(&params.text_document.uri)
                    .unwrap()
                    .clone();
                Self::notify_analysis_error(
                    err,
                    connection,
                    &params.text_document.uri,
                    params.text_document.version,
                    document_state,
                )?;
            }
        }
        Ok(())
    }

    pub(crate) fn handle_close_document(
        &mut self,
        n: lsp_server::Notification,
    ) -> Result<(), Box<dyn Error>> {
        let params: DidCloseTextDocumentParams = n.extract(DidCloseTextDocument::METHOD)?;
        self.cleanup(&params.text_document.uri);
        Ok(())
    }

    pub(crate) fn handle_goto_definition(
        &mut self,
        params: GotoDefinitionParams,
    ) -> GotoDefinitionResponse {
        let document_state = self
            .documents
            .get(&params.text_document_position_params.text_document.uri)
            .unwrap();
        let mut locations = Vec::new();
        if let Some(text_at_position) =
            document_state.ident_at_position(params.text_document_position_params.position)
        {
            eprintln!("text_at_position: {text_at_position}");

            // Handle non-terminals here
            Self::add_non_terminal_definitions(
                document_state,
                text_at_position,
                &mut locations,
                &params,
            );

            // Handle user types here
            Self::find_user_type_definitions(
                document_state,
                text_at_position.to_owned(),
                &mut locations,
                &params,
            );

            // Handle scanner states here
            Self::find_scanner_state_definitions(
                document_state,
                text_at_position.to_owned(),
                &mut locations,
                &params,
            );
        }
        GotoDefinitionResponse::Array(locations)
    }

    pub(crate) fn handle_hover(&mut self, params: HoverParams) -> Hover {
        let document_state = self
            .documents
            .get(&params.text_document_position_params.text_document.uri)
            .unwrap();
        document_state.hover(params)
    }

    pub(crate) fn handle_document_symbols(
        &self,
        params: DocumentSymbolParams,
    ) -> DocumentSymbolResponse {
        let document_state = self.documents.get(&params.text_document.uri).unwrap();
        document_state.document_symbols(params)
    }

    pub(crate) fn handle_prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> Option<PrepareRenameResponse> {
        if let Some(document_state) = self.documents.get(&params.text_document.uri) {
            document_state.prepare_rename(params)
        } else {
            None
        }
    }

    pub(crate) fn handle_rename(&self, params: RenameParams) -> Option<WorkspaceEdit> {
        if let Some(document_state) = self
            .documents
            .get(&params.text_document_position.text_document.uri)
        {
            document_state.rename(params)
        } else {
            None
        }
    }

    pub(crate) fn handle_formatting(
        &self,
        mut params: DocumentFormattingParams,
    ) -> Option<Vec<TextEdit>> {
        if let Some(document_state) = self.documents.get(&params.text_document.uri) {
            self.formatting_settings.add_to_options(&mut params.options);
            document_state.format(params)
        } else {
            None
        }
    }

    pub(crate) fn handle_changed_configuration(
        &mut self,
        n: lsp_server::Notification,
    ) -> Result<(), Box<dyn Error>> {
        let params: DidChangeConfigurationParams = n.extract(DidChangeConfiguration::METHOD)?;
        let config_properties: ConfigProperties =
            serde_json::from_value(params.settings).unwrap_or_default();
        self.update_configuration(&config_properties)?;
        Ok(())
    }

    fn cleanup(&mut self, uri: &Uri) {
        self.documents.remove(uri);
    }

    fn apply_changes(&mut self, uri: &Uri, content_changes: &[TextDocumentContentChangeEvent]) {
        if let Some(change) = content_changes.last() {
            self.apply_change(uri, change);
        }
    }

    fn apply_change(&mut self, uri: &Uri, change: &TextDocumentContentChangeEvent) {
        self.documents
            .get_mut(uri)
            .unwrap()
            .input
            .clone_from(&change.text);
    }

    fn find_user_type_definitions(
        document_state: &DocumentState,
        text_at_position: String,
        locations: &mut Vec<Location>,
        params: &GotoDefinitionParams,
    ) {
        if let Some(ranges) = document_state
            .parsed_data
            .user_type_definitions
            .find_definitions(&text_at_position)
        {
            locations.push(Location {
                uri: params
                    .text_document_position_params
                    .text_document
                    .uri
                    .clone(),
                range: ranges[0],
            });
        }
    }

    fn find_scanner_state_definitions(
        document_state: &DocumentState,
        text_at_position: String,
        locations: &mut Vec<Location>,
        params: &GotoDefinitionParams,
    ) {
        if let Some(ranges) = document_state
            .parsed_data
            .scanner_state_definitions
            .find_definitions(&text_at_position)
        {
            locations.push(Location {
                uri: params
                    .text_document_position_params
                    .text_document
                    .uri
                    .clone(),
                range: ranges[0],
            });
        }
    }

    pub(crate) fn find_non_terminal_definitions(
        document_state: &DocumentState,
        non_terminal: &str,
    ) -> Option<Vec<Range>> {
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
                    range,
                });
            }
        }
    }

    fn notify_analysis_ok(
        connection: Arc<lsp_server::Connection>,
        uri: Uri,
        version: i32,
    ) -> Result<(), Box<dyn Error>> {
        let result = PublishDiagnosticsParams::new(uri, vec![], Some(version));
        let params = serde_json::to_value(result).unwrap();
        let method = <PublishDiagnostics as Notification>::METHOD.to_string();
        connection
            .sender
            .send(Message::Notification(lsp_server::Notification {
                method,
                params,
            }))?;
        Ok(())
    }

    fn notify_analysis_error(
        err: anyhow::Error,
        connection: Arc<lsp_server::Connection>,
        uri: &Uri,
        version: i32,
        document_state: DocumentState,
    ) -> Result<(), Box<dyn Error>> {
        let result = PublishDiagnosticsParams::new(
            uri.clone(),
            Diagnostics::to_diagnostics(uri, &document_state, err),
            Some(version),
        );
        let params = serde_json::to_value(result).unwrap();
        let method = <PublishDiagnostics as Notification>::METHOD.to_string();
        connection
            .sender
            .send(Message::Notification(lsp_server::Notification {
                method,
                params,
            }))?;
        Ok(())
    }

    fn notify_resolved_conflicts(
        resolved_conflicts: Vec<parol::analysis::lalr1_parse_table::LRResolvedConflict>,
        connection: Arc<lsp_server::Connection>,
        uri: &Uri,
        version: i32,
    ) -> Result<(), Box<dyn Error>> {
        if resolved_conflicts.is_empty() {
            return Ok(());
        }
        let result = PublishDiagnosticsParams::new(
            uri.clone(),
            vec![Diagnostics::to_resolved_conflict_warning(
                uri,
                resolved_conflicts,
            )],
            Some(version),
        );
        let params = serde_json::to_value(result).unwrap();
        let method = <PublishDiagnostics as Notification>::METHOD.to_string();
        connection
            .sender
            .send(Message::Notification(lsp_server::Notification {
                method,
                params,
            }))?;
        Ok(())
    }
}
