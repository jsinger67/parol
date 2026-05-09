use std::{
    collections::{HashMap, HashSet},
    error::Error,
    path::{Path, PathBuf},
    sync::Arc,
    thread,
};

use lsp_server::Message;
use lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, CodeActionParams, CodeActionResponse,
    DidChangeConfigurationParams, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, DocumentFormattingParams, DocumentSymbolParams,
    DocumentSymbolResponse, GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverParams,
    Location, Position, PrepareRenameResponse, PublishDiagnosticsParams, Range, RenameParams,
    TextDocumentContentChangeEvent, TextDocumentPositionParams, TextEdit, Uri, WorkspaceEdit,
    notification::{
        DidChangeConfiguration, DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument,
        Notification, PublishDiagnostics,
    },
};
use parol::generators::grammar_trans::check_and_transform_grammar_with_ignored;
use parol::{
    GrammarConfig, ParolGrammar, analysis::lalr1_parse_table::calculate_lalr1_parse_table,
    calculate_lookahead_dfas, parser::parol_grammar::GrammarType,
};

use crate::{
    config::ConfigProperties, diagnostics::Diagnostics, document_state::DocumentState,
    formatting::FormattingSettings, parol_ls_parser::parse, utils::pos_to_offset,
};

use regex::Regex;

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
        let ignored_unreachable_non_terminals = grammar_config
            .unreachable_non_terminals_to_ignore
            .iter()
            .cloned()
            .collect();
        let cfg = check_and_transform_grammar_with_ignored(
            &grammar_config.cfg,
            grammar_config.grammar_type,
            &ignored_unreachable_non_terminals,
        )?;
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

    pub(crate) fn handle_code_action(
        &self,
        params: CodeActionParams,
    ) -> Option<CodeActionResponse> {
        let uri = &params.text_document.uri;
        let document_state = self.documents.get(uri)?;
        let mut actions: Vec<CodeActionOrCommand> = Vec::new();
        let non_terminals: HashSet<String> = document_state
            .parsed_data
            .productions
            .keys()
            .cloned()
            .collect();
        let scanner_tokens = Self::scanner_token_map(&document_state.input, &non_terminals);

        for diagnostic in params.context.diagnostics {
            let Some(code) = Self::diagnostic_code(&diagnostic) else {
                continue;
            };

            if code == "parol::parser::invalid_token_in_transition" {
                let candidates = Self::replacement_candidates(
                    &document_state.input,
                    &diagnostic,
                    &scanner_tokens,
                );
                actions.extend(
                    Self::make_replace_token_actions(uri, &diagnostic, &candidates)
                        .into_iter()
                        .map(CodeActionOrCommand::CodeAction),
                );
                if let Some(action) = Self::make_delete_line_action(
                    uri,
                    document_state,
                    &diagnostic,
                    "Remove invalid %on transition directive line",
                ) {
                    actions.push(CodeActionOrCommand::CodeAction(action));
                }
            }

            if code == "parol::parser::token_not_in_scanner" {
                let candidates = Self::replacement_candidates(
                    &document_state.input,
                    &diagnostic,
                    &scanner_tokens,
                );
                actions.extend(
                    Self::make_replace_token_actions(uri, &diagnostic, &candidates)
                        .into_iter()
                        .map(CodeActionOrCommand::CodeAction),
                );
                if let Some(action) =
                    Self::make_remove_from_skip_action(uri, document_state, &diagnostic)
                {
                    actions.push(CodeActionOrCommand::CodeAction(action));
                }
                if let Some(action) = Self::make_delete_line_action(
                    uri,
                    document_state,
                    &diagnostic,
                    "Remove invalid %skip directive line",
                ) {
                    actions.push(CodeActionOrCommand::CodeAction(action));
                }
            }
        }

        Some(actions)
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

    fn diagnostic_code(diagnostic: &lsp_types::Diagnostic) -> Option<&str> {
        match diagnostic.code.as_ref() {
            Some(lsp_types::NumberOrString::String(code)) => Some(code.as_str()),
            _ => None,
        }
    }

    fn make_replace_token_action(
        uri: &Uri,
        diagnostic: &lsp_types::Diagnostic,
        replacement: &str,
    ) -> Option<CodeAction> {
        if diagnostic.range.start == diagnostic.range.end {
            return None;
        }
        let edit = TextEdit {
            range: diagnostic.range,
            new_text: replacement.to_owned(),
        };
        Some(CodeAction {
            title: format!("Replace token with '{replacement}'"),
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: Some(vec![diagnostic.clone()]),
            edit: Some(WorkspaceEdit {
                changes: Some(HashMap::from([(uri.clone(), vec![edit])])),
                ..Default::default()
            }),
            is_preferred: Some(false),
            ..Default::default()
        })
    }

    fn make_replace_token_actions(
        uri: &Uri,
        diagnostic: &lsp_types::Diagnostic,
        replacements: &[String],
    ) -> Vec<CodeAction> {
        replacements
            .iter()
            .take(5)
            .filter_map(|replacement| Self::make_replace_token_action(uri, diagnostic, replacement))
            .collect()
    }

    fn replacement_candidates(
        input: &str,
        diagnostic: &lsp_types::Diagnostic,
        scanner_tokens: &HashMap<String, Vec<String>>,
    ) -> Vec<String> {
        let current = Self::text_from_range(input, diagnostic.range).unwrap_or_default();
        let scanner = Self::scanner_from_diagnostic_message(&diagnostic.message)
            .or_else(|| Self::scanner_at_line(input, diagnostic.range.start.line))
            .unwrap_or_else(|| "INITIAL".to_owned());

        let in_scanner = scanner_tokens.get(&scanner).cloned().unwrap_or_default();
        let in_initial = if scanner != "INITIAL" {
            scanner_tokens.get("INITIAL").cloned().unwrap_or_default()
        } else {
            Vec::new()
        };

        let mut joined = in_scanner;
        joined.extend(in_initial);
        joined.sort();
        joined.dedup();
        joined.retain(|candidate| candidate != &current);
        if joined.is_empty() {
            return scanner_tokens
                .values()
                .flat_map(|values| values.iter().cloned())
                .filter(|candidate| candidate != &current)
                .take(5)
                .collect();
        }
        joined
    }

    fn scanner_token_map(
        input: &str,
        non_terminals: &HashSet<String>,
    ) -> HashMap<String, Vec<String>> {
        let token_def_re = Regex::new(
            r#"^\s*([A-Za-z_][A-Za-z0-9_]*)\s*:\s*(?:<\s*([^>]+)\s*>)?\s*(?:"[^"]*"|'[^']*'|/[^/]+/)"#,
        )
        .expect("token definition regex must compile");

        let mut map: HashMap<String, Vec<String>> = HashMap::new();
        for line in input.lines() {
            let Some(captures) = token_def_re.captures(line) else {
                continue;
            };
            let Some(name) = captures.get(1).map(|m| m.as_str().to_owned()) else {
                continue;
            };
            if non_terminals.contains(&name) {
                continue;
            }

            if let Some(states) = captures.get(2) {
                for scanner in states
                    .as_str()
                    .split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                {
                    map.entry(scanner.to_owned())
                        .or_default()
                        .push(name.clone());
                }
            } else {
                map.entry("INITIAL".to_owned())
                    .or_default()
                    .push(name.clone());
            }
        }

        for values in map.values_mut() {
            values.sort();
            values.dedup();
        }
        map
    }

    fn scanner_from_diagnostic_message(message: &str) -> Option<String> {
        let scanner_re =
            Regex::new(r"scanner '([^']+)'").expect("diagnostic scanner regex must compile");
        scanner_re
            .captures(message)
            .and_then(|captures| captures.get(1).map(|m| m.as_str().to_owned()))
    }

    fn scanner_at_line(input: &str, line: u32) -> Option<String> {
        let scanner_start_re = Regex::new(r"^\s*%scanner\s+([A-Za-z_][A-Za-z0-9_]*)\s*\{")
            .expect("scanner start regex must compile");
        let mut current: Option<String> = None;
        let target = line as usize;
        for (idx, text) in input.lines().enumerate() {
            if idx > target {
                break;
            }
            if let Some(captures) = scanner_start_re.captures(text) {
                current = captures.get(1).map(|m| m.as_str().to_owned());
            }
            if current.is_some() && text.contains('}') {
                current = None;
            }
        }
        current.or_else(|| Some("INITIAL".to_owned()))
    }

    fn make_delete_line_action(
        uri: &Uri,
        document_state: &DocumentState,
        diagnostic: &lsp_types::Diagnostic,
        title: &str,
    ) -> Option<CodeAction> {
        let line = diagnostic.range.start.line;
        let delete_range = Self::line_delete_range(&document_state.input, line)?;
        let edit = TextEdit {
            range: delete_range,
            new_text: String::new(),
        };
        Some(CodeAction {
            title: title.to_owned(),
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: Some(vec![diagnostic.clone()]),
            edit: Some(WorkspaceEdit {
                changes: Some(HashMap::from([(uri.clone(), vec![edit])])),
                ..Default::default()
            }),
            is_preferred: Some(false),
            ..Default::default()
        })
    }

    fn make_remove_from_skip_action(
        uri: &Uri,
        document_state: &DocumentState,
        diagnostic: &lsp_types::Diagnostic,
    ) -> Option<CodeAction> {
        let token = Self::text_from_range(&document_state.input, diagnostic.range)?;
        let line = diagnostic.range.start.line;
        let line_text = Self::line_text(&document_state.input, line)?;
        if !line_text.contains("%skip") {
            return None;
        }

        let (before_comment, comment) = Self::split_inline_comment(line_text);
        let skip_pos = before_comment.find("%skip")?;
        let (prefix, suffix) = before_comment.split_at(skip_pos + "%skip".len());
        let mut identifiers: Vec<String> = suffix
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(ToOwned::to_owned)
            .collect();
        let original_len = identifiers.len();
        identifiers.retain(|s| s != &token);
        if identifiers.len() == original_len || identifiers.is_empty() {
            return None;
        }

        let mut new_line = format!("{prefix} {}", identifiers.join(", "));
        if let Some(comment) = comment {
            if !new_line.ends_with(' ') {
                new_line.push(' ');
            }
            new_line.push_str(comment.trim_start());
        }

        let replace_range = Self::line_content_range(&document_state.input, line)?;
        let edit = TextEdit {
            range: replace_range,
            new_text: new_line,
        };
        Some(CodeAction {
            title: "Remove token from %skip list".to_owned(),
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: Some(vec![diagnostic.clone()]),
            edit: Some(WorkspaceEdit {
                changes: Some(HashMap::from([(uri.clone(), vec![edit])])),
                ..Default::default()
            }),
            is_preferred: Some(true),
            ..Default::default()
        })
    }

    fn split_inline_comment(line: &str) -> (&str, Option<&str>) {
        if let Some(idx) = line.find("//") {
            let (left, right) = line.split_at(idx);
            return (left, Some(right));
        }
        if let Some(idx) = line.find('#') {
            let (left, right) = line.split_at(idx);
            return (left, Some(right));
        }
        (line, None)
    }

    fn line_text(input: &str, line: u32) -> Option<&str> {
        input.lines().nth(line as usize)
    }

    fn line_char_len(input: &str, line: u32) -> Option<u32> {
        Self::line_text(input, line).map(|line_text| line_text.chars().count() as u32)
    }

    fn line_content_range(input: &str, line: u32) -> Option<Range> {
        let line_len = Self::line_char_len(input, line)?;
        Some(Range {
            start: Position { line, character: 0 },
            end: Position {
                line,
                character: line_len,
            },
        })
    }

    fn line_delete_range(input: &str, line: u32) -> Option<Range> {
        let line_count = input.lines().count() as u32;
        if line >= line_count {
            return None;
        }
        let end = if line + 1 < line_count {
            Position {
                line: line + 1,
                character: 0,
            }
        } else {
            Position {
                line,
                character: Self::line_char_len(input, line)?,
            }
        };
        Some(Range {
            start: Position { line, character: 0 },
            end,
        })
    }

    fn text_from_range(input: &str, range: Range) -> Option<String> {
        let start = pos_to_offset(input, range.start);
        let end = pos_to_offset(input, range.end);
        if start >= end || end > input.len() {
            return None;
        }
        Some(input.get(start..end)?.trim().to_owned())
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

#[cfg(test)]
mod tests {
    use super::{DocumentState, Server};
    use lsp_types::{Diagnostic, Position, Range};
    use std::collections::HashSet;
    use std::str::FromStr;

    const SAMPLE_GRAMMAR: &str = r#"%start S
%scanner Esc {
    %skip BAD
}

%%

S: <Esc>"x";
BAD: "bad";
GOOD: <Esc>"good";
WS: /[ \t\r\n]+/;
"#;

    fn range_of(input: &str, needle: &str) -> Range {
        let offset = input.find(needle).expect("needle must exist in test input");
        let before = &input[..offset];
        let line = before.lines().count().saturating_sub(1) as u32;
        let col = before
            .lines()
            .last()
            .map(|l| l.chars().count() as u32)
            .unwrap_or(0);
        Range {
            start: Position {
                line,
                character: col,
            },
            end: Position {
                line,
                character: col + needle.chars().count() as u32,
            },
        }
    }

    #[test]
    fn scanner_token_map_parses_scoped_and_initial_tokens() {
        let non_terminals = HashSet::from(["S".to_string()]);
        let map = Server::scanner_token_map(SAMPLE_GRAMMAR, &non_terminals);

        assert_eq!(
            map.get("Esc").cloned().unwrap_or_default(),
            vec!["GOOD".to_string()]
        );
        assert_eq!(
            map.get("INITIAL").cloned().unwrap_or_default(),
            vec!["BAD".to_string(), "WS".to_string()]
        );
    }

    #[test]
    fn scanner_at_line_detects_current_scanner_block() {
        // %skip BAD is on line 2 (0-based)
        let scanner = Server::scanner_at_line(SAMPLE_GRAMMAR, 2)
            .expect("scanner should be resolved for line");
        assert_eq!(scanner, "Esc");

        // Grammar definition lines should fall back to INITIAL
        let scanner =
            Server::scanner_at_line(SAMPLE_GRAMMAR, 6).expect("scanner should default to INITIAL");
        assert_eq!(scanner, "INITIAL");
    }

    #[test]
    fn replacement_candidates_prefer_scanner_and_filter_current_token() {
        let non_terminals = HashSet::from(["S".to_string()]);
        let map = Server::scanner_token_map(SAMPLE_GRAMMAR, &non_terminals);
        let diagnostic = Diagnostic {
            message: "Token 'BAD' is referenced in '%skip' but is not available in scanner 'Esc'."
                .to_string(),
            range: range_of(SAMPLE_GRAMMAR, "BAD"),
            ..Default::default()
        };

        let candidates = Server::replacement_candidates(SAMPLE_GRAMMAR, &diagnostic, &map);

        assert!(candidates.contains(&"GOOD".to_string()));
        assert!(!candidates.contains(&"BAD".to_string()));
    }

    #[test]
    fn scanner_from_diagnostic_message_extracts_scanner_name() {
        let scanner = Server::scanner_from_diagnostic_message(
            "Token 'BAD' is referenced in '%skip' but is not available in scanner 'Esc'.",
        )
        .expect("scanner should be extracted from message");
        assert_eq!(scanner, "Esc");
    }

    #[test]
    fn make_remove_from_skip_action_rewrites_skip_list_and_preserves_inline_comment() {
        let input = r#"%start S
%scanner Esc {
    %skip BAD, GOOD // keep scanner hygiene
}

%%
S: "x";
BAD: "bad";
GOOD: "good";
"#;
        let uri = lsp_types::Uri::from_str("file:///test.par").expect("valid URI");
        let document_state = DocumentState {
            input: input.to_string(),
            ..Default::default()
        };
        let diagnostic = Diagnostic {
            message: "Token 'BAD' is referenced in '%skip' but is not available in scanner 'Esc'."
                .to_string(),
            range: range_of(input, "BAD"),
            ..Default::default()
        };

        let action = Server::make_remove_from_skip_action(&uri, &document_state, &diagnostic)
            .expect("remove-from-skip action should be generated");
        assert_eq!(action.title, "Remove token from %skip list");

        let edit = action
            .edit
            .expect("action must contain edits")
            .changes
            .expect("workspace edit must contain changes")
            .get(&uri)
            .expect("changes for URI must exist")[0]
            .clone();

        assert_eq!(edit.new_text, "    %skip GOOD // keep scanner hygiene");
    }

    #[test]
    fn make_remove_from_skip_action_is_none_for_single_token_skip_list() {
        let input = r#"%start S
%scanner Esc {
    %skip BAD
}

%%
S: "x";
BAD: "bad";
"#;
        let uri = lsp_types::Uri::from_str("file:///test.par").expect("valid URI");
        let document_state = DocumentState {
            input: input.to_string(),
            ..Default::default()
        };
        let diagnostic = Diagnostic {
            message: "Token 'BAD' is referenced in '%skip' but is not available in scanner 'Esc'."
                .to_string(),
            range: range_of(input, "BAD"),
            ..Default::default()
        };

        let action = Server::make_remove_from_skip_action(&uri, &document_state, &diagnostic);
        assert!(action.is_none());
    }
}
