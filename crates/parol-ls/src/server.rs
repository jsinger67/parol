use derive_new::new;
use std::{collections::HashMap, error::Error, path::Path, sync::Arc, thread};

use anyhow::anyhow;
use lsp_server::Message;
use lsp_types::{
    notification::{
        DidChangeConfiguration, DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument,
        Notification, PublishDiagnostics,
    },
    DidChangeConfigurationParams, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, DocumentFormattingParams, DocumentSymbolParams,
    DocumentSymbolResponse, GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverParams,
    Location, PrepareRenameResponse, PublishDiagnosticsParams, Range, RenameParams,
    TextDocumentContentChangeEvent, TextDocumentPositionParams, TextEdit, Url, WorkspaceEdit,
};
use parol::{calculate_lookahead_dfas, check_and_transform_grammar, GrammarConfig, ParolGrammar};

use crate::{
    config::ConfigProperties, diagnostics::Diagnostics, document_state::DocumentState,
    parol_ls_parser::parse,
};

#[derive(Debug, Default, new)]
pub(crate) struct Server {
    /// Any documents the server has handled, indexed by their URL
    #[new(default)]
    documents: HashMap<String, DocumentState>,

    /// Limit for lookahead calculation.
    /// Be careful with high values. The server can get stuck for some grammars.
    max_k: usize,
}

impl Server {
    pub(crate) fn update_configuration(
        &mut self,
        props: &ConfigProperties,
    ) -> Result<(), serde_json::error::Error> {
        if props.0.contains_key("max_k") {
            self.max_k = serde_json::from_value(
                props
                    .0
                    .get("max_k")
                    .unwrap_or(&serde_json::Value::Null)
                    .clone(),
            )?;
            eprintln!("Using a maximum lookahead of {} now", self.max_k);
        }
        Ok(())
    }

    pub(crate) fn analyze(
        &mut self,
        uri: Url,
        version: i32,
        connection: Arc<lsp_server::Connection>,
    ) -> anyhow::Result<()> {
        let file_path = uri
            .to_file_path()
            .map_err(|_| anyhow!("Failed interpreting file path {}", uri.path()))?;
        let document_state = self.documents.get_mut(uri.path()).unwrap();
        eprintln!("analyze: step 1 - parse");
        document_state.clear();
        parse(
            &document_state.input,
            &file_path,
            &mut document_state.parsed_data,
        )?;
        eprintln!("analyze: step 2 - check_grammar");
        let document_state = self.documents.get(uri.path()).unwrap();
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
        uri: Url,
        version: i32,
        document_state: DocumentState,
    ) -> anyhow::Result<()> {
        let mut grammar_config = Self::obtain_grammar_config_from_string(input, file_name)?;
        let cfg = check_and_transform_grammar(&grammar_config.cfg)?;
        grammar_config.update_cfg(cfg);
        let grammar_config = grammar_config.clone();
        thread::spawn(move || {
            if let Err(err) = calculate_lookahead_dfas(&grammar_config, max_k) {
                eprintln!("check_grammar: errors from calculate_lookahead_dfas");
                let _ = Self::notify_analysis_error(err, connection, uri, version, document_state);
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
            params.text_document.uri.path().to_string(),
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
                let path = params.text_document.uri.path();
                let document_state = self.documents.get(path).unwrap().clone();
                Self::notify_analysis_error(
                    err,
                    connection,
                    params.text_document.uri,
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
        self.apply_changes(params.text_document.uri.path(), &params.content_changes);
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
                let path = params.text_document.uri.path();
                let document_state = self.documents.get(path).unwrap().clone();
                Self::notify_analysis_error(
                    err,
                    connection,
                    params.text_document.uri,
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

    pub(crate) fn handle_hover(&mut self, params: HoverParams) -> Hover {
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
        document_state.hover(params)
    }

    pub(crate) fn handle_document_symbols(
        &self,
        params: DocumentSymbolParams,
    ) -> DocumentSymbolResponse {
        let document_state = self.documents.get(params.text_document.uri.path()).unwrap();
        document_state.document_symbols(params)
    }

    pub(crate) fn handle_prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> Option<PrepareRenameResponse> {
        if let Some(document_state) = self.documents.get(params.text_document.uri.path()) {
            document_state.prepare_rename(params)
        } else {
            None
        }
    }

    pub(crate) fn handle_rename(&self, params: RenameParams) -> Option<WorkspaceEdit> {
        if let Some(document_state) = self
            .documents
            .get(params.text_document_position.text_document.uri.path())
        {
            document_state.rename(params)
        } else {
            None
        }
    }

    pub(crate) fn handle_formatting(
        &self,
        params: DocumentFormattingParams,
    ) -> Option<Vec<TextEdit>> {
        if let Some(document_state) = self.documents.get(params.text_document.uri.path()) {
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
        if let Some(range) = document_state
            .parsed_data
            .user_type_definitions
            .get(&text_at_position)
        {
            locations.push(Location {
                uri: params.text_document_position_params.text_document.uri,
                range: *range,
            });
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

    fn notify_analysis_ok(
        connection: Arc<lsp_server::Connection>,
        uri: Url,
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
        uri: Url,
        version: i32,
        document_state: DocumentState,
    ) -> Result<(), Box<dyn Error>> {
        eprintln!("handle_open_document: document obtained");
        let result = PublishDiagnosticsParams::new(
            uri.clone(),
            Diagnostics::to_diagnostics(&uri, &document_state, err),
            Some(version),
        );
        let params = serde_json::to_value(result).unwrap();
        let method = <PublishDiagnostics as Notification>::METHOD.to_string();
        eprintln!("handle_open_document: sending response\n{:?}", params);
        connection
            .sender
            .send(Message::Notification(lsp_server::Notification {
                method,
                params,
            }))?;
        Ok(())
    }
}
