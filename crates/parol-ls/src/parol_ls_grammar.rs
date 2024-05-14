use crate::{
    formatting::Comments,
    parol_ls_grammar_trait::{
        self, Declaration, NonTerminal, ParolLs, ParolLsGrammarTrait, Production, ProductionLHS,
        ScannerDirectives, ScannerState, StartDeclaration, TokenLiteral, UserTypeDeclaration,
    },
    rng::Rng,
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
use parol_runtime::lexer::Token;
#[allow(unused_imports)]
use parol_runtime::Result;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Error, Formatter, Write as _};

///
/// Data structure that implements the semantic actions for our ParolLs grammar
///
#[derive(Debug, Clone, Default)]
pub struct ParolLsGrammar {
    // A hash that maps non-terminals to their productions' left-hand side.
    pub non_terminal_definitions: HashMap<String, Vec<Range>>,
    pub non_terminals: Vec<(Range, String)>,
    pub start_symbol: String,

    pub user_type_definitions: HashMap<String, Range>,
    pub user_types: Vec<(Range, String)>,

    // A hash that maps non-terminals to their productions
    pub productions: HashMap<String, Vec<Production>>,

    pub symbols: Vec<DocumentSymbol>,

    pub grammar: Option<ParolLs>,

    pub(crate) comments: Comments,
}

impl ParolLsGrammar {
    pub fn new() -> Self {
        ParolLsGrammar::default()
    }

    pub(crate) fn ident_at_position(&self, position: Position) -> Option<String> {
        if let Some((_, non_terminal)) = self
            .non_terminals
            .iter()
            .find(|(r, _)| r.start <= position && r.end > position)
        {
            Some(non_terminal.clone())
        } else if let Some((_, user_type)) = self
            .user_types
            .iter()
            .find(|(r, _)| r.start <= position && r.end > position)
        {
            Some(user_type.clone())
        } else {
            None
        }
    }

    pub(crate) fn find_non_terminal_definitions<'a>(
        &'a self,
        non_terminal: &str,
    ) -> Option<&'a Vec<Range>> {
        // eprintln!(
        //     "{non_terminal} included: {}",
        //     self.non_terminal_definitions.contains_key(non_terminal)
        // );
        self.non_terminal_definitions.get(non_terminal)
    }

    fn find_non_terminal_range(&self, non_terminal: &str, position: Position) -> Option<Range> {
        self.non_terminals.iter().find_map(|(r, n)| {
            if n == non_terminal && r.start <= position && r.end > position {
                Some(*r)
            } else {
                None
            }
        })
    }

    fn add_non_terminal_ref(&mut self, range: Range, token: &OwnedToken) {
        // eprintln!("add_non_terminal_ref: {range:?}, {}", token);
        self.non_terminals.push((range, token.text().to_string()));
    }

    fn add_non_terminal_definition(&mut self, token: &OwnedToken) -> Range {
        let entry = self
            .non_terminal_definitions
            .entry(token.text().to_string())
            .or_default();
        let range = location_to_range(&token.location);
        // eprintln!("add_non_terminal_definition: {range:?}, {}", token);
        entry.push(range);
        range
    }

    fn add_user_type_ref(&mut self, range: Range, token: &OwnedToken) {
        self.user_types.push((range, token.text().to_string()));
    }

    fn add_user_type_definition(&mut self, token: &OwnedToken, range: Range) -> Range {
        let entry = self
            .user_type_definitions
            .entry(token.text().to_string())
            .or_default();
        *entry = range;
        range
    }

    fn add_scanner_symbols(symbols: &mut Vec<DocumentSymbol>, arg: &ScannerDirectives) {
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
            ScannerDirectives::PercentOnIdentifierListPercentEnterIdentifier(trans) => {
                #[allow(deprecated)]
                symbols.push(DocumentSymbol {
                    name: trans.percent_on.text().to_string(),
                    detail: Some("Transition".to_string()),
                    kind: SymbolKind::PROPERTY,
                    tags: None,
                    deprecated: None,
                    range: Into::<Rng>::into(arg).0,
                    selection_range: Into::<Rng>::into(&trans.percent_on).0,
                    children: None,
                });
            }
        }
    }

    pub(crate) fn hover(&self, params: HoverParams, input: &str) -> Hover {
        let mut value = String::new();
        let ident = self.ident_at_position(params.text_document_position_params.position);
        if let Some(item) = ident {
            value = format!("## {}", item);
            if let Some(productions) = self.productions.get(&item) {
                for p in productions {
                    let rng: Rng = p.into();
                    let _ = write!(value, "\n{}", to_markdown(extract_text_range(input, rng)));
                }
            } else if let Some(range) = self.user_type_definitions.get(&item) {
                let _ = write!(
                    value,
                    "\n{}",
                    to_markdown(extract_text_range(input, Rng(*range)))
                );
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
        if let Some(non_terminal) = ident {
            if let Some(range) = self.find_non_terminal_range(&non_terminal, params.position) {
                // Currently we don't support renaming the start symbol because this would have
                // impact on the whole structure of the user's crate.
                if non_terminal != self.start_symbol {
                    return Some(PrepareRenameResponse::Range(range));
                }
            }
        }
        None
    }

    pub(crate) fn rename(&self, params: RenameParams) -> Option<WorkspaceEdit> {
        let ident = self.ident_at_position(params.text_document_position.position);
        if let Some(non_terminal) = ident {
            // Currently we don't support renaming the start symbol because this would have
            // impact on the whole structure of the user's crate.
            if non_terminal != self.start_symbol {
                let text_document_edits = TextDocumentEdit {
                    text_document: OptionalVersionedTextDocumentIdentifier {
                        uri: params.text_document_position.text_document.uri.clone(),
                        version: None,
                    },
                    edits: self.non_terminals.iter().fold(vec![], |mut acc, (r, n)| {
                        if n == &non_terminal {
                            acc.push(OneOf::Left(TextEdit {
                                range: *r,
                                new_text: params.new_name.clone(),
                            }));
                        }
                        acc
                    }),
                };
                let document_changes = Some(DocumentChanges::Edits(vec![text_document_edits]));
                return Some(WorkspaceEdit {
                    document_changes,
                    ..Default::default()
                });
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

    fn expanded_token_literal(token_literal: &TokenLiteral) -> String {
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

    /// Semantic action for non-terminal 'StartDeclaration'
    fn start_declaration(&mut self, arg: &StartDeclaration) -> Result<()> {
        let token = &arg.identifier.identifier;
        let range = location_to_range(&token.location);
        self.add_non_terminal_ref(range, token);

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
                let range = self.add_user_type_definition(token, range.into());
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
                Self::add_scanner_symbols(&mut self.symbols, &scanner.scanner_directives);
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
        }
        Ok(())
    }

    /// Semantic action for non-terminal 'ProductionLHS'
    fn production_l_h_s(&mut self, arg: &ProductionLHS) -> Result<()> {
        let token = &arg.identifier.identifier;
        let range = self.add_non_terminal_definition(token);
        self.add_non_terminal_ref(range, token);
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
        let range = location_to_range(&token.location);
        self.add_non_terminal_ref(range, token);
        Ok(())
    }

    /// Semantic action for non-terminal 'ScannerState'
    fn scanner_state(&mut self, arg: &ScannerState) -> Result<()> {
        let scanner_state_symbols: Vec<DocumentSymbol> =
            arg.scanner_state_list.iter().fold(vec![], |mut acc, s| {
                Self::add_scanner_symbols(&mut acc, &s.scanner_directives);
                acc
            });
        let name = format!("{} {}", arg.percent_scanner, arg.identifier.identifier);
        #[allow(deprecated)]
        self.symbols.push(DocumentSymbol {
            name,
            detail: Some("Scanner state".to_string()),
            kind: SymbolKind::STRUCT,
            tags: None,
            deprecated: None,
            range: Into::<Rng>::into(arg).0,
            selection_range: Into::<Rng>::into(&arg.percent_scanner).0,
            children: Some(scanner_state_symbols),
        });
        Ok(())
    }

    /// Semantic action for non-terminal 'UserTypeDeclaration'
    fn user_type_declaration(&mut self, arg: &UserTypeDeclaration) -> Result<()> {
        let token = &arg.user_type_name.identifier.identifier;
        let range = location_to_range(&token.location);
        self.add_user_type_ref(range, token);
        Ok(())
    }

    fn on_comment_parsed(&mut self, token: Token<'_>) {
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
