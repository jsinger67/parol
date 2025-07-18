use crate::{
    formatting::Comments,
    parol_ls_grammar_trait::{
        self, ASTControl, Declaration, NonTerminal, ParolLs, ParolLsGrammarTrait, Production,
        ProductionLHS, Prolog, ScannerDirectives, ScannerState, ScannerStateDirectives,
        SimpleToken, StartDeclaration, TokenLiteral, TokenWithStates, UserTypeDeclaration,
        UserTypeName,
    },
    rng::Rng,
    symbol_def::SymbolDefs,
    utils::{extract_text_range, location_to_range, to_markdown},
};
use lsp_types::{
    DocumentChanges, DocumentFormattingParams, DocumentSymbol, DocumentSymbolParams,
    DocumentSymbolResponse, Hover, HoverContents::Markup, HoverParams, MarkupContent, MarkupKind,
    OneOf, OptionalVersionedTextDocumentIdentifier, Position, PrepareRenameResponse, Range,
    RenameParams, SymbolKind, TextDocumentEdit, TextDocumentPositionParams, TextEdit,
    WorkspaceEdit,
};
use parol::TerminalKind;
#[allow(unused_imports)]
use parol_runtime::Result;
use parol_runtime::lexer::Token;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Error, Formatter, Write as _};

#[derive(Debug, Clone)]
pub(crate) enum SymbolDefsType {
    Terminal,
    NonTerminal,
    UserType,
    ScannerState,
}

///
/// Data structure that implements the semantic actions for our ParolLs grammar
///
#[derive(Debug, Clone, Default)]
pub struct ParolLsGrammar {
    // The start symbol of the grammar
    pub start_symbol: String,

    // Scanner state definitions and references
    pub(crate) scanner_state_definitions: SymbolDefs,

    // User type definitions and references
    pub(crate) user_type_definitions: SymbolDefs,

    // Non-terminal definitions and references
    pub(crate) non_terminal_definitions: SymbolDefs,

    // Terminal type definition and references
    pub(crate) terminal_type: SymbolDefs,

    // A hash that maps non-terminals to their productions
    pub productions: HashMap<String, Vec<Production>>,

    // A list of document symbols
    pub symbols: Vec<DocumentSymbol>,

    // The grammar
    pub grammar: Option<ParolLs>,

    // A list of comments
    pub(crate) comments: Comments,
}

impl ParolLsGrammar {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn ident_at_position(&self, position: Position) -> Option<(&str, SymbolDefsType)> {
        self.non_terminal_definitions
            .find_reference(position)
            .map(|name| (name, SymbolDefsType::NonTerminal))
            .or_else(|| {
                self.user_type_definitions
                    .find_reference(position)
                    .map(|name| (name, SymbolDefsType::UserType))
            })
            .or_else(|| {
                self.scanner_state_definitions
                    .find_reference(position)
                    .map(|name| (name, SymbolDefsType::ScannerState))
            })
            .or_else(|| {
                self.terminal_type
                    .find_reference(position)
                    .map(|name| (name, SymbolDefsType::Terminal))
            })
    }

    pub(crate) fn find_non_terminal_definitions(&self, non_terminal: &str) -> Option<Vec<Range>> {
        self.non_terminal_definitions.find_definitions(non_terminal)
    }

    fn find_non_terminal_range(&self, non_terminal: &str, position: Position) -> Option<&Range> {
        self.non_terminal_definitions
            .find_reference_range(non_terminal, position)
    }

    fn add_non_terminal_ref(&mut self, token: &OwnedToken) {
        let range = location_to_range(&token.location);
        // eprintln!("add_non_terminal_ref: {range:?}, {}", token);
        self.non_terminal_definitions
            .add_reference(range, token.text());
    }

    fn add_non_terminal_definition(&mut self, token: &OwnedToken) {
        self.non_terminal_definitions.add_definition_by_token(token)
    }

    /// Adds a scanner state definition to the list of scanner state definitions
    fn add_scanner_state_definition(&mut self, identifier: &OwnedToken, range: Range) {
        // Hover support, range is the range of the whole scanner state
        self.scanner_state_definitions
            .add_definition(identifier.text().to_string(), range);
        // Rename support
        self.add_scanner_state_ref(identifier);
    }

    fn add_scanner_state_ref(&mut self, token: &OwnedToken) {
        // eprintln!("add_scanner_state_ref: {range:?}, {}", token);
        self.scanner_state_definitions.add_reference_by_token(token);
    }

    fn add_user_type_ref(&mut self, range: Range, token: &OwnedToken) {
        self.user_type_definitions
            .add_reference(range, token.text());
    }

    fn add_user_type_definition(&mut self, range: Range, token: &OwnedToken) -> Range {
        // eprint!("add_user_type_definition: {range:?}, {}", token);
        self.user_type_definitions
            .add_definition(token.text().to_string(), range);
        range
    }

    fn add_terminal_type_def(&mut self, t_type: &UserTypeName) {
        self.terminal_type
            .add_definition("%t_type".to_owned(), Into::<Rng>::into(t_type).0);
    }

    fn add_terminal_ref(&mut self, range: Range) {
        if self.terminal_type.find_definitions("%t_type").is_some() {
            // eprintln!("add_terminal_ref: {range:?}");
            self.terminal_type.add_reference(range, "%t_type");
        }
    }

    fn add_scanner_symbols(&mut self, symbols: &mut Vec<DocumentSymbol>, arg: &ScannerDirectives) {
        match arg {
            ScannerDirectives::PercentLineUnderscoreCommentTokenLiteral(line_comment) => {
                #[allow(deprecated)]
                symbols.push(DocumentSymbol {
                    name: line_comment
                        .percent_line_underscore_comment
                        .text()
                        .to_string(),
                    detail: Some("Line comment for the scanner state".to_string()),
                    kind: SymbolKind::PROPERTY,
                    tags: None,
                    deprecated: None,
                    range: Into::<Rng>::into(arg).0,
                    selection_range: Into::<Rng>::into(
                        &line_comment.percent_line_underscore_comment,
                    )
                    .0,
                    children: Some(vec![DocumentSymbol {
                        name: Self::expanded_token_literal(&line_comment.token_literal),
                        detail: Some("Text".to_string()),
                        kind: SymbolKind::STRING,
                        tags: None,
                        deprecated: None,
                        range: Into::<Rng>::into(arg).0,
                        selection_range: Into::<Rng>::into(&line_comment.token_literal).0,
                        children: None,
                    }]),
                });
            }
            ScannerDirectives::PercentBlockUnderscoreCommentTokenLiteralTokenLiteral(
                block_comment,
            ) => {
                #[allow(deprecated)]
                symbols.push(DocumentSymbol {
                    name: block_comment
                        .percent_block_underscore_comment
                        .text()
                        .to_string(),
                    detail: Some("Block comment for the scanner state".to_string()),
                    kind: SymbolKind::PROPERTY,
                    tags: None,
                    deprecated: None,
                    range: Into::<Rng>::into(arg).0,
                    selection_range: Into::<Rng>::into(
                        &block_comment.percent_block_underscore_comment,
                    )
                    .0,
                    children: Some(vec![
                        DocumentSymbol {
                            name: Self::expanded_token_literal(&block_comment.token_literal),
                            detail: Some("Text".to_string()),
                            kind: SymbolKind::STRING,
                            tags: None,
                            deprecated: None,
                            range: Into::<Rng>::into(arg).0,
                            selection_range: Into::<Rng>::into(&block_comment.token_literal).0,
                            children: None,
                        },
                        DocumentSymbol {
                            name: Self::expanded_token_literal(&block_comment.token_literal0),
                            detail: Some("Text".to_string()),
                            kind: SymbolKind::STRING,
                            tags: None,
                            deprecated: None,
                            range: Into::<Rng>::into(arg).0,
                            selection_range: Into::<Rng>::into(&block_comment.token_literal0).0,
                            children: None,
                        },
                    ]),
                });
            }
            ScannerDirectives::PercentAutoUnderscoreNewlineUnderscoreOff(auto_newline) => {
                #[allow(deprecated)]
                symbols.push(DocumentSymbol {
                    name: auto_newline
                        .percent_auto_underscore_newline_underscore_off
                        .text()
                        .to_string(),
                    detail: Some("Handle newlines alone".to_string()),
                    kind: SymbolKind::PROPERTY,
                    tags: None,
                    deprecated: None,
                    range: Into::<Rng>::into(arg).0,
                    selection_range: Into::<Rng>::into(
                        &auto_newline.percent_auto_underscore_newline_underscore_off,
                    )
                    .0,
                    children: None,
                });
            }
            ScannerDirectives::PercentAutoUnderscoreWsUnderscoreOff(auto_ws) => {
                #[allow(deprecated)]
                symbols.push(DocumentSymbol {
                    name: auto_ws
                        .percent_auto_underscore_ws_underscore_off
                        .text()
                        .to_string(),
                    detail: Some("Handle whitespace alone".to_string()),
                    kind: SymbolKind::PROPERTY,
                    tags: None,
                    deprecated: None,
                    range: Into::<Rng>::into(arg).0,
                    selection_range: Into::<Rng>::into(
                        &auto_ws.percent_auto_underscore_ws_underscore_off,
                    )
                    .0,
                    children: None,
                });
            }
            ScannerDirectives::PercentOnIdentifierListScannerStateDirectives(trans) => {
                // Add the reference to the non-terminal for hover and rename support
                // This is the first non-terminal in the struct `identifier_list`
                self.add_non_terminal_ref(&trans.identifier_list.identifier.identifier);

                let mut first_id: DocumentSymbol =
                    (&trans.identifier_list.identifier.identifier).into();
                first_id.detail = Some("Initiating terminal".to_string());

                let mut children: Vec<DocumentSymbol> = trans
                    .identifier_list
                    .identifier_list_list
                    .iter()
                    .fold(vec![first_id], |mut acc, id| {
                        let mut id_sym: DocumentSymbol = (&id.identifier.identifier).into();
                        id_sym.detail = Some("Initiating terminal".to_string());

                        // Add the reference to the non-terminal for hover and rename support
                        self.add_non_terminal_ref(&id.identifier.identifier);

                        acc.push(id_sym);
                        acc
                    });
                match &trans.scanner_state_directives {
                    ScannerStateDirectives::PercentEnterIdentifier(
                        scanner_state_directives_percent_enter_identifier,
                    ) => {
                        // Add the reference to the scanner state for hover and rename support
                        self.add_scanner_state_ref(
                            &scanner_state_directives_percent_enter_identifier
                                .identifier
                                .identifier,
                        );

                        let mut target_state: DocumentSymbol =
                            (&scanner_state_directives_percent_enter_identifier
                                .identifier
                                .identifier)
                                .into();
                        target_state.detail = Some("Target state".to_string());
                        target_state.kind = SymbolKind::STRUCT;
                        target_state.range = Into::<Rng>::into(arg).0;
                        target_state.selection_range = Into::<Rng>::into(&trans.percent_on).0;

                        children.push(target_state);

                        let mut on_enter_directive: DocumentSymbol = (&trans.percent_on).into();
                        on_enter_directive.detail = Some("Scanner state transition".to_string());
                        // Extend the range to include the target state
                        on_enter_directive.range = Into::<Rng>::into(arg).0;
                        on_enter_directive.selection_range = Into::<Rng>::into(&trans.percent_on).0;
                        on_enter_directive.kind = SymbolKind::STRUCT;
                        on_enter_directive.children = Some(children);

                        symbols.push(on_enter_directive);
                    }
                    ScannerStateDirectives::PercentPushIdentifier(
                        scanner_state_directives_percent_push_identifier,
                    ) => {
                        // Add the reference to the scanner state for hover and rename support
                        self.add_scanner_state_ref(
                            &scanner_state_directives_percent_push_identifier
                                .identifier
                                .identifier,
                        );

                        let mut target_state: DocumentSymbol =
                            (&scanner_state_directives_percent_push_identifier
                                .identifier
                                .identifier)
                                .into();
                        target_state.detail = Some("Target state".to_string());
                        target_state.kind = SymbolKind::STRUCT;
                        target_state.range = Into::<Rng>::into(arg).0;
                        target_state.selection_range = Into::<Rng>::into(&trans.percent_on).0;

                        children.push(target_state);

                        let mut on_push_directive: DocumentSymbol = (&trans.percent_on).into();
                        on_push_directive.detail =
                            Some("Scanner state transition with push".to_string());
                        // Extend the range to include the target state
                        on_push_directive.range = Into::<Rng>::into(arg).0;
                        on_push_directive.selection_range = Into::<Rng>::into(&trans.percent_on).0;
                        on_push_directive.kind = SymbolKind::STRUCT;
                        on_push_directive.children = Some(children);

                        symbols.push(on_push_directive);
                    }
                    ScannerStateDirectives::PercentPop(scanner_state_directives_percent_pop) => {
                        let mut on_pop_directive: DocumentSymbol =
                            (&scanner_state_directives_percent_pop.percent_pop).into();
                        on_pop_directive.detail =
                            Some("Scanner state transition with pop".to_string());
                        // Extend the range to include the target state
                        on_pop_directive.range = Into::<Rng>::into(arg).0;
                        on_pop_directive.selection_range = Into::<Rng>::into(&trans.percent_on).0;
                        on_pop_directive.kind = SymbolKind::STRUCT;
                        on_pop_directive.children = Some(children);

                        symbols.push(on_pop_directive);
                    }
                }
            }
        }
    }

    pub(crate) fn hover(&self, params: HoverParams, input: &str) -> Hover {
        let mut value = String::new();
        let ident = self.ident_at_position(params.text_document_position_params.position);
        if let Some((item, kind)) = ident {
            if let SymbolDefsType::Terminal = kind {
                // Terminals show their %t_type definition if available
                if let Some(ranges) = self.terminal_type.find_definitions(item) {
                    debug_assert!(ranges.len() == 1);
                    let _ = write!(
                        value,
                        "{}",
                        to_markdown(extract_text_range(input, Rng(ranges[0])))
                    );
                }
            } else {
                value = format!("## {}", item);
                if let Some(productions) = self.productions.get(item) {
                    for p in productions {
                        let rng: Rng = p.into();
                        let _ = write!(value, "\n{}", to_markdown(extract_text_range(input, rng)));
                    }
                } else if let Some(ranges) = self.user_type_definitions.find_definitions(item) {
                    let _ = write!(
                        value,
                        "\n{}",
                        to_markdown(extract_text_range(input, Rng(ranges[0])))
                    );
                } else if let Some(ranges) = self.scanner_state_definitions.find_definitions(item) {
                    let _ = write!(
                        value,
                        "\n{}",
                        to_markdown(extract_text_range(input, Rng(ranges[0])))
                    );
                }
            }
        }
        let markup_content = MarkupContent {
            kind: MarkupKind::Markdown,
            value,
        };
        let contents = Markup(markup_content);
        Hover {
            contents,
            range: None,
        }
    }

    pub(crate) fn document_symbols(
        &self,
        _params: DocumentSymbolParams,
        _input: &str,
    ) -> DocumentSymbolResponse {
        DocumentSymbolResponse::Nested(self.symbols.clone())
    }

    pub(crate) fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> Option<PrepareRenameResponse> {
        let ident = self.ident_at_position(params.position);
        if let Some((ident, _kind)) = ident {
            if let Some(range) = self.find_non_terminal_range(ident, params.position) {
                // Currently we don't support renaming the start symbol because this would have
                // impact on the whole structure of the user's crate.
                if ident != self.start_symbol {
                    return Some(PrepareRenameResponse::Range(*range));
                }
            } else if let Some(range) = self
                .scanner_state_definitions
                .find_reference_range(ident, params.position)
            {
                // The INITIAL scanner state is a special case. We don't want to rename it.
                if ident != "INITIAL" {
                    return Some(PrepareRenameResponse::Range(*range));
                }
            }
        }
        None
    }

    pub(crate) fn rename(&self, params: RenameParams) -> Option<WorkspaceEdit> {
        let ident = self.ident_at_position(params.text_document_position.position);
        if let Some((ident, kind)) = ident {
            match kind {
                SymbolDefsType::NonTerminal => {
                    // Currently we don't support renaming the start symbol because this would have
                    // impact on the whole structure of the user's crate.
                    if ident != self.start_symbol {
                        let text_document_edits = TextDocumentEdit {
                            text_document: OptionalVersionedTextDocumentIdentifier {
                                uri: params.text_document_position.text_document.uri.clone(),
                                version: None,
                            },
                            edits: self
                                .non_terminal_definitions
                                .find_references(ident)
                                .iter()
                                .fold(vec![], |mut acc, r| {
                                    acc.push(OneOf::Left(TextEdit {
                                        range: **r,
                                        new_text: params.new_name.clone(),
                                    }));
                                    acc
                                }),
                        };
                        let document_changes =
                            Some(DocumentChanges::Edits(vec![text_document_edits]));
                        return Some(WorkspaceEdit {
                            document_changes,
                            ..Default::default()
                        });
                    }
                }
                SymbolDefsType::UserType => (),
                SymbolDefsType::ScannerState => {
                    // The INITIAL scanner state is a special case. We don't want to rename it.
                    if ident != "INITIAL" {
                        let text_document_edits = TextDocumentEdit {
                            text_document: OptionalVersionedTextDocumentIdentifier {
                                uri: params.text_document_position.text_document.uri.clone(),
                                version: None,
                            },
                            edits: self
                                .scanner_state_definitions
                                .find_references(ident)
                                .iter()
                                .fold(vec![], |mut acc, r| {
                                    acc.push(OneOf::Left(TextEdit {
                                        range: **r,
                                        new_text: params.new_name.clone(),
                                    }));
                                    acc
                                }),
                        };
                        let document_changes =
                            Some(DocumentChanges::Edits(vec![text_document_edits]));
                        return Some(WorkspaceEdit {
                            document_changes,
                            ..Default::default()
                        });
                    }
                }
                SymbolDefsType::Terminal => (),
            }
        }
        // eprintln!("prepare rename request rejected");
        None
    }

    pub(crate) fn format(&self, params: DocumentFormattingParams) -> Option<Vec<TextEdit>> {
        if let Some(ref grammar) = self.grammar {
            Some(
                <&parol_ls_grammar_trait::ParolLs as crate::formatting::Format>::format(
                    &grammar,
                    &params.options,
                    self.comments.clone(),
                ),
            )
        } else {
            None
        }
    }

    fn trim_quotes(string: &str) -> String {
        let delimiters: &[_] = &['"', '\'', '/'];
        string
            .strip_prefix(delimiters)
            .unwrap()
            .strip_suffix(delimiters)
            .unwrap()
            .to_string()
    }

    pub(crate) fn expanded_token_literal(token_literal: &TokenLiteral) -> String {
        match token_literal {
            TokenLiteral::String(s) => {
                TerminalKind::Legacy.expand(Self::trim_quotes(s.string.string.text()).as_str())
            }
            TokenLiteral::LiteralString(l) => TerminalKind::Raw
                .expand(Self::trim_quotes(l.literal_string.literal_string.text()).as_str()),
            TokenLiteral::Regex(r) => {
                TerminalKind::Regex.expand(Self::trim_quotes(r.regex.regex.text()).as_str())
            }
        }
    }
}

impl ParolLsGrammarTrait for ParolLsGrammar {
    /// Semantic action for non-terminal 'ParolLs'
    fn parol_ls(&mut self, arg: &ParolLs) -> Result<()> {
        self.grammar = Some(arg.clone());
        Ok(())
    }

    /// Semantic action for non-terminal 'Prolog'
    fn prolog(&mut self, prolog: &Prolog) -> Result<()> {
        self.scanner_state_definitions.add_definition(
            "INITIAL".to_string(),
            Rng::from_slice(&prolog.prolog_list).0,
        );
        Ok(())
    }

    /// Semantic action for non-terminal 'StartDeclaration'
    fn start_declaration(&mut self, arg: &StartDeclaration) -> Result<()> {
        let token = &arg.identifier.identifier;
        let range = location_to_range(&token.location);
        self.add_non_terminal_ref(token);

        self.start_symbol = token.text().to_string();

        #[allow(deprecated)]
        self.symbols.push(DocumentSymbol {
            name: arg.percent_start.text().to_string(),
            detail: Some("Start symbol".to_string()),
            kind: SymbolKind::PROPERTY,
            tags: None,
            deprecated: None,
            range: Into::<Rng>::into(arg).0,
            selection_range: Into::<Rng>::into(&arg.percent_start).0,
            children: Some(vec![DocumentSymbol {
                name: arg.identifier.identifier.text().to_string(),
                detail: Some("Non-terminal".to_string()),
                kind: SymbolKind::VARIABLE,
                tags: None,
                deprecated: None,
                range,
                selection_range: range,
                children: None,
            }]),
        });
        Ok(())
    }

    /// Semantic action for non-terminal 'Declaration'
    fn declaration(&mut self, arg: &Declaration) -> Result<()> {
        match arg {
            Declaration::PercentTitleString(title) => {
                #[allow(deprecated)]
                self.symbols.push(DocumentSymbol {
                    name: title.percent_title.text().to_string(),
                    detail: Some("Title of the grammar".to_string()),
                    kind: SymbolKind::PROPERTY,
                    tags: None,
                    deprecated: None,
                    range: Into::<Rng>::into(arg).0,
                    selection_range: Into::<Rng>::into(&title.percent_title).0,
                    children: Some(vec![DocumentSymbol {
                        name: title.string.string.text().to_string(),
                        detail: Some("Text".to_string()),
                        kind: SymbolKind::STRING,
                        tags: None,
                        deprecated: None,
                        range: Into::<Rng>::into(arg).0,
                        selection_range: Into::<Rng>::into(&title.string.string).0,
                        children: None,
                    }]),
                });
            }
            Declaration::PercentCommentString(comment) => {
                #[allow(deprecated)]
                self.symbols.push(DocumentSymbol {
                    name: comment.percent_comment.text().to_string(),
                    detail: Some("Comment for the grammar".to_string()),
                    kind: SymbolKind::PROPERTY,
                    tags: None,
                    deprecated: None,
                    range: Into::<Rng>::into(arg).0,
                    selection_range: Into::<Rng>::into(&comment.percent_comment).0,
                    children: Some(vec![DocumentSymbol {
                        name: comment.string.string.text().to_string(),
                        detail: Some("Text".to_string()),
                        kind: SymbolKind::STRING,
                        tags: None,
                        deprecated: None,
                        range: Into::<Rng>::into(arg).0,
                        selection_range: Into::<Rng>::into(&comment.string.string).0,
                        children: None,
                    }]),
                });
            }
            Declaration::PercentUserUnderscoreTypeIdentifierEquUserTypeName(user_type_def) => {
                let token = &user_type_def.identifier.identifier;
                let range: Rng = arg.into();
                let range = self.add_user_type_definition(range.into(), token);
                self.add_user_type_ref(range, token);
                let range = Into::<Rng>::into(&user_type_def.identifier.identifier).0;

                #[allow(deprecated)]
                self.symbols.push(DocumentSymbol {
                    name: user_type_def
                        .percent_user_underscore_type
                        .text()
                        .to_string(),
                    detail: Some("User type definition".to_string()),
                    kind: SymbolKind::CONSTANT,
                    tags: None,
                    deprecated: None,
                    range: Into::<Rng>::into(arg).0,
                    selection_range: Into::<Rng>::into(&user_type_def.percent_user_underscore_type)
                        .0,
                    children: Some(vec![DocumentSymbol {
                        name: user_type_def.identifier.identifier.text().to_string(),
                        detail: Some("Type alias".to_string()),
                        kind: SymbolKind::CONSTANT,
                        tags: None,
                        deprecated: None,
                        range,
                        selection_range: range,
                        children: None,
                    }]),
                });
            }
            Declaration::ScannerDirectives(scanner) => {
                let mut scanner_symbols: Vec<DocumentSymbol> = vec![];
                self.add_scanner_symbols(&mut scanner_symbols, &scanner.scanner_directives);
                self.symbols.extend(scanner_symbols);
            }
            Declaration::PercentGrammarUnderscoreTypeLiteralString(grammar_type) => {
                #[allow(deprecated)]
                self.symbols.push(DocumentSymbol {
                    name: grammar_type
                        .percent_grammar_underscore_type
                        .text()
                        .to_string(),
                    detail: Some("Grammar type".to_string()),
                    kind: SymbolKind::TYPE_PARAMETER,
                    tags: None,
                    deprecated: None,
                    range: Into::<Rng>::into(arg).0,
                    selection_range: Into::<Rng>::into(
                        &grammar_type.percent_grammar_underscore_type,
                    )
                    .0,
                    children: Some(vec![DocumentSymbol {
                        name: grammar_type
                            .literal_string
                            .literal_string
                            .text()
                            .to_string(),
                        detail: Some("Text".to_string()),
                        kind: SymbolKind::STRING,
                        tags: None,
                        deprecated: None,
                        range: Into::<Rng>::into(arg).0,
                        selection_range: Into::<Rng>::into(
                            &grammar_type.literal_string.literal_string,
                        )
                        .0,
                        children: None,
                    }]),
                });
            }
            Declaration::PercentNtUnderscoreTypeNtNameEquNtType(nt_type) => {
                // Allow the non-terminal to be renamed
                self.add_non_terminal_ref(&nt_type.nt_name.identifier);

                let range = Into::<Rng>::into(&nt_type.percent_nt_underscore_type).0;

                let nt_types_node = if let Some(nt_types_node) = self
                    .symbols
                    .iter()
                    .position(|s| s.name == "%nt_type Definitions")
                {
                    self.symbols.get_mut(nt_types_node).unwrap()
                } else {
                    #[allow(deprecated)]
                    self.symbols.push(DocumentSymbol {
                        name: "%nt_type Definitions".to_string(),
                        detail: Some("Non-terminal type definitions".to_string()),
                        kind: SymbolKind::OBJECT,
                        tags: None,
                        deprecated: None,
                        range,
                        selection_range: range,
                        children: Some(Vec::new()),
                    });
                    self.symbols.last_mut().unwrap()
                };

                let range = Into::<Rng>::into(&nt_type.nt_type).0;

                #[allow(deprecated)]
                nt_types_node
                    .children
                    .as_mut()
                    .unwrap()
                    .push(DocumentSymbol {
                        name: format!(
                            "{} {}",
                            nt_type.percent_nt_underscore_type.text(),
                            nt_type.nt_name.identifier.text()
                        ),
                        detail: Some("Non-terminal type".to_string()),
                        kind: SymbolKind::PROPERTY,
                        tags: None,
                        deprecated: None,
                        range: Into::<Rng>::into(&nt_type.percent_nt_underscore_type)
                            .extend(Into::<Rng>::into(&nt_type.nt_name))
                            .0,
                        selection_range: Into::<Rng>::into(&nt_type.percent_nt_underscore_type).0,
                        children: Some(vec![DocumentSymbol {
                            name: nt_type.nt_name.identifier.text().to_string(),
                            detail: Some("Text".to_string()),
                            kind: SymbolKind::STRING,
                            tags: None,
                            deprecated: None,
                            range,
                            selection_range: range,
                            children: None,
                        }]),
                    });
            }
            Declaration::PercentTUnderscoreTypeTType(t_type) => {
                self.add_terminal_type_def(&t_type.t_type);
                #[allow(deprecated)]
                self.symbols.push(DocumentSymbol {
                    name: t_type.percent_t_underscore_type.text().to_string(),
                    detail: Some("Terminal type".to_string()),
                    kind: SymbolKind::PROPERTY,
                    tags: None,
                    deprecated: None,
                    range: Into::<Rng>::into(arg).0,
                    selection_range: Into::<Rng>::into(&t_type.percent_t_underscore_type).0,
                    children: Some(vec![DocumentSymbol {
                        name: t_type.t_type.to_string(),
                        detail: Some("Text".to_string()),
                        kind: SymbolKind::STRING,
                        tags: None,
                        deprecated: None,
                        range: Into::<Rng>::into(&t_type.t_type).0,
                        selection_range: Into::<Rng>::into(&t_type.t_type).0,
                        children: None,
                    }]),
                });
            }
        }
        Ok(())
    }

    /// Semantic action for non-terminal 'ProductionLHS'
    fn production_l_h_s(&mut self, arg: &ProductionLHS) -> Result<()> {
        let token = &arg.identifier.identifier;
        self.add_non_terminal_definition(token);
        self.add_non_terminal_ref(token);
        Ok(())
    }

    /// Semantic action for non-terminal 'SimpleToken'
    fn simple_token(&mut self, arg: &SimpleToken) -> Result<()> {
        if let Some(a) = arg.simple_token_opt.as_ref() {
            if !matches!(a.a_s_t_control, ASTControl::CutOperator(_)) {
                // Add terminal reference only if the terminal is not omitted by a cut operator
                self.add_terminal_ref(Into::<Rng>::into(arg).0);
            }
        } else {
            self.add_terminal_ref(Into::<Rng>::into(arg).0);
        }
        Ok(())
    }

    /// Semantic action for non-terminal 'TokenWithStates'
    fn token_with_states(&mut self, arg: &TokenWithStates) -> Result<()> {
        if let Some(a) = arg.token_with_states_opt.as_ref() {
            if !matches!(a.a_s_t_control, ASTControl::CutOperator(_)) {
                // Add terminal reference only if the terminal is not omitted by a cut operator
                self.add_terminal_ref(Into::<Rng>::into(arg).0);
            }
        } else {
            self.add_terminal_ref(Into::<Rng>::into(arg).0);
        }

        [arg.identifier_list.identifier.identifier.clone()]
            .iter()
            .chain(
                arg.identifier_list
                    .identifier_list_list
                    .iter()
                    .map(|id| &id.identifier.identifier),
            )
            .for_each(|id| {
                let token = &id;
                self.add_scanner_state_ref(token);
            });
        Ok(())
    }

    /// Semantic action for non-terminal 'Production'
    fn production(&mut self, arg: &Production) -> Result<()> {
        let nt = arg
            .production_l_h_s
            .identifier
            .identifier
            .text()
            .to_string();
        // let rng: Rng = arg.into();
        // eprintln!("Adding production {nt:?}: {rng:?}");
        let entry = self.productions.entry(nt).or_default();
        entry.push(arg.clone());
        // eprintln!("Length: {}", entry.len());

        #[allow(deprecated)]
        self.symbols.push(DocumentSymbol {
            name: arg
                .production_l_h_s
                .identifier
                .identifier
                .text()
                .to_string(),
            detail: Some("Production".to_string()),
            kind: SymbolKind::FUNCTION,
            tags: None,
            deprecated: None,
            range: Into::<Rng>::into(arg).0,
            selection_range: Into::<Rng>::into(&arg.production_l_h_s.identifier.identifier).0,
            children: None,
        });
        Ok(())
    }

    /// Semantic action for non-terminal 'NonTerminal'
    fn non_terminal(&mut self, arg: &NonTerminal) -> Result<()> {
        let token = &arg.identifier.identifier;
        self.add_non_terminal_ref(token);
        Ok(())
    }

    /// Semantic action for non-terminal 'ScannerState'
    fn scanner_state(&mut self, arg: &ScannerState) -> Result<()> {
        let scanner_state_symbols: Vec<DocumentSymbol> =
            arg.scanner_state_list.iter().fold(vec![], |mut acc, s| {
                self.add_scanner_symbols(&mut acc, &s.scanner_directives);
                acc
            });
        let name = format!("{} {}", arg.percent_scanner, arg.identifier.identifier);
        #[allow(deprecated)]
        let document_symbol = DocumentSymbol {
            name,
            detail: Some("Scanner state".to_string()),
            kind: SymbolKind::STRUCT,
            tags: None,
            deprecated: None,
            range: Into::<Rng>::into(arg).0,
            selection_range: Into::<Rng>::into(&arg.percent_scanner).0,
            children: Some(scanner_state_symbols),
        };
        self.add_scanner_state_definition(&arg.identifier.identifier, Into::<Rng>::into(arg).0);
        self.symbols.push(document_symbol);
        Ok(())
    }

    /// Semantic action for non-terminal 'UserTypeDeclaration'
    fn user_type_declaration(&mut self, arg: &UserTypeDeclaration) -> Result<()> {
        let token = &arg.user_type_name.identifier.identifier;
        let range = location_to_range(&token.location);
        self.add_user_type_ref(range, token);
        Ok(())
    }

    fn on_comment(&mut self, token: Token<'_>) {
        self.comments.push_back(OwnedToken(token.into_owned()))
    }
}

#[derive(Debug, Clone)]
pub struct OwnedToken(Token<'static>);

impl OwnedToken {
    pub(crate) fn is_line_comment(&self) -> bool {
        self.text().starts_with("//")
    }
}

impl<'t> TryFrom<&Token<'t>> for OwnedToken {
    type Error = anyhow::Error;

    fn try_from(token: &Token<'t>) -> std::result::Result<Self, Self::Error> {
        let owned_token = token.clone().into_owned();
        Ok(Self(owned_token))
    }
}

impl std::ops::Deref for OwnedToken {
    type Target = Token<'static>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for OwnedToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(f, "{}", self.0.text())
    }
}

impl Display for UserTypeName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(f, "{}", self.identifier.identifier)?;
        for id in &self.user_type_name_list {
            write!(f, "::{}", id.identifier.identifier)?;
        }
        Ok(())
    }
}
