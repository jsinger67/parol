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
use parol::{
    analysis::lalr1_parse_table::calculate_lalr1_parse_table, calculate_lookahead_dfas,
    check_and_transform_grammar, parser::parol_grammar::SupportedGrammarType, GrammarConfig,
    ParolGrammar,
};

use crate::{
    config::ConfigProperties, diagnostics::Diagnostics, document_state::DocumentState,
    parol_ls_parser::parse,
};

macro_rules! add_boolean_option {
    ($self:ident, $( $container:ident ).+, $( $option_name:ident ).+, $member_name:ident) => {
        $($container).+.insert(
            stringify!($( $option_name ).+).to_owned(),
            lsp_types::FormattingProperty::Bool($self.$member_name),
        );
    };
}

macro_rules! add_boolean_formatting_option {
    ($self:ident, $( $container:ident ).+, $option_name:ident) => {
        add_boolean_option!($self, $($container).+, formatting.$option_name, $option_name)
    };
}

macro_rules! add_number_option {
    ($self:ident, $( $container:ident ).+, $( $option_name:ident ).+, $member_name:ident) => {
        $($container).+.insert(
            stringify!($( $option_name ).+).to_owned(),
            lsp_types::FormattingProperty::Number($self.$member_name as i32),
        );
    };
}

macro_rules! add_number_formatting_option {
    ($self:ident, $( $container:ident ).+, $option_name:ident) => {
        add_number_option!($self, $($container).+, formatting.$option_name, $option_name)
    };
}

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

macro_rules! update_number_formatting_option {
    ($self:ident, $props:ident, $member_name:ident, $default:literal) => {
        update_number_option!(
            $self,
            $props,
            formatting.$member_name,
            $member_name,
            $default
        );
    };
}

macro_rules! update_boolean_option {
    ($self:ident, $props:ident, $( $option_name:ident ).+, $member_name:ident, $default:literal) => {
        if $props
            .0
            .contains_key(stringify!($($option_name).+))
        {
            $self.$member_name = serde_json::from_value(
                $props
                    .0
                    .get(stringify!($($option_name).+))
                    .unwrap_or(&serde_json::Value::Bool($default))
                    .clone(),
            )?;
            eprintln!(
                concat!(stringify!($($option_name).+), ": {}"),
                $self.$member_name
            );
        }
    };
}

macro_rules! update_boolean_formatting_option {
    ($self:ident, $props:ident, $member_name:ident, $default:literal) => {
        update_boolean_option!(
            $self,
            $props,
            formatting.$member_name,
            $member_name,
            $default
        );
    };
}

#[derive(Debug, Default)]
pub(crate) struct Server {
    /// Any documents the server has handled, indexed by their URL
    documents: HashMap<String, DocumentState>,

    /// Limit for lookahead calculation.
    /// Be careful with high values. The server can get stuck for some grammars.
    max_k: usize,

    /// Add an empty line after each production
    /// * Formatting option
    empty_line_after_prod: bool,

    /// Place the semicolon after each production on a new line
    /// * Formatting option
    prod_semicolon_on_nl: bool,

    /// Number of characters per line
    /// * Formatting option
    max_line_length: usize,
}

impl Server {
    pub(crate) fn new(max_k: usize) -> Self {
        Self {
            max_k,
            max_line_length: 100,
            ..Default::default()
        }
    }
    pub(crate) fn update_configuration(
        &mut self,
        props: &ConfigProperties,
    ) -> Result<(), serde_json::error::Error> {
        update_number_option!(self, props, max_k, max_k, 3);
        update_boolean_formatting_option!(self, props, empty_line_after_prod, true);
        update_boolean_formatting_option!(self, props, prod_semicolon_on_nl, true);
        update_number_formatting_option!(self, props, max_line_length, 100);
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
        let cfg = check_and_transform_grammar(&grammar_config.cfg, grammar_config.grammar_type)?;
        grammar_config.update_cfg(cfg);
        let grammar_config = grammar_config.clone();
        thread::spawn(move || match grammar_config.grammar_type {
            SupportedGrammarType::LLK => {
                if let Err(err) = calculate_lookahead_dfas(&grammar_config, max_k) {
                    eprintln!("check_grammar: errors from calculate_lookahead_dfas");
                    let _ =
                        Self::notify_analysis_error(err, connection, uri, version, document_state);
                }
            }
            SupportedGrammarType::LALR1 => {
                if let Err(err) = calculate_lalr1_parse_table(&grammar_config) {
                    eprintln!("check_grammar: errors from calculate_lookahead_dfas");
                    let _ =
                        Self::notify_analysis_error(err, connection, uri, version, document_state);
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
        mut params: DocumentFormattingParams,
    ) -> Option<Vec<TextEdit>> {
        if let Some(document_state) = self.documents.get(params.text_document.uri.path()) {
            add_boolean_formatting_option!(self, params.options.properties, empty_line_after_prod);
            add_boolean_formatting_option!(self, params.options.properties, prod_semicolon_on_nl);
            add_number_formatting_option!(self, params.options.properties, max_line_length);
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
