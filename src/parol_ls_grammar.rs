use crate::{
    parol_ls_grammar_trait::{
        Declaration, NonTerminal, ParolLs, ParolLsGrammarTrait, Production, ProductionLHS,
        StartDeclaration, UserTypeDeclaration,
    },
    rng::Rng,
    utils::{extract_text_range, location_to_range, to_markdown},
};
use lsp_types::{
    Hover, HoverContents::Markup, HoverParams, MarkupContent, MarkupKind, Position, Range,
};
#[allow(unused_imports)]
use miette::Result;
use parol_runtime::lexer::OwnedToken;
use std::fmt::Write as _;
use std::{collections::HashMap, fmt::Debug};

///
/// Data structure that implements the semantic actions for our ParolLs grammar
///
#[derive(Debug, Default)]
pub struct ParolLsGrammar {
    // A hash that maps non-terminals to their productions' left-hand side.
    pub non_terminal_definitions: HashMap<String, Vec<Range>>,
    pub non_terminals: Vec<(Range, String)>,

    pub user_type_definitions: HashMap<String, Vec<Range>>,
    pub user_types: Vec<(Range, String)>,

    // A hash that maps non-terminals to their productions
    pub productions: HashMap<String, Vec<Production>>,

    pub grammar: Option<ParolLs>,
}

impl ParolLsGrammar {
    pub fn new() -> Self {
        ParolLsGrammar::default()
    }

    pub(crate) fn ident_at_position(&self, position: Position) -> Option<String> {
        if let Some((_, non_terminal)) = self
            .non_terminals
            .iter()
            .find(|(k, _)| k.start <= position && k.end > position)
        {
            Some(non_terminal.clone())
        } else if let Some((_, user_type)) = self
            .user_types
            .iter()
            .find(|(k, _)| k.start <= position && k.end > position)
        {
            Some(user_type.clone())
        } else {
            None
        }
    }

    // pub(crate) fn item_at_position(&self, _position: Position) -> Option<&ASTType> {
    //     todo!()
    // }

    pub(crate) fn find_non_terminal_definitions<'a>(
        &'a self,
        non_terminal: &str,
    ) -> Option<&'a Vec<Range>> {
        eprintln!(
            "{non_terminal} included: {}",
            self.non_terminal_definitions.contains_key(non_terminal)
        );
        self.non_terminal_definitions.get(non_terminal)
    }

    fn add_non_terminal_ref(&mut self, range: Range, token: &OwnedToken) {
        eprintln!("add_non_terminal_ref: {range:?}, {}", token.symbol);
        self.non_terminals.push((range, token.symbol.clone()));
    }

    fn add_non_terminal_definition(&mut self, token: &OwnedToken) -> Range {
        let entry = self
            .non_terminal_definitions
            .entry(token.symbol.clone())
            .or_default();
        let range = location_to_range(&token.location);
        eprintln!("add_non_terminal_definition: {range:?}, {}", token.symbol);
        entry.push(range);
        range
    }

    fn add_user_type_ref(&mut self, range: Range, token: &OwnedToken) {
        self.user_types.push((range, token.symbol.clone()));
    }

    fn add_user_type_definition(&mut self, token: &OwnedToken) -> Range {
        let entry = self
            .user_type_definitions
            .entry(token.symbol.clone())
            .or_default();
        let range = location_to_range(&token.location);
        entry.push(range);
        range
    }

    pub(crate) fn hover(&self, params: HoverParams, input: &str) -> Hover {
        let mut value = String::new();
        if let Some(item) = self.ident_at_position(params.text_document_position_params.position) {
            value = format!("## {}", item);
            if let Some(productions) = self.productions.get(&item) {
                for p in productions {
                    let rng: Rng = p.into();
                    let _ = write!(value, "\n{}", to_markdown(extract_text_range(input, rng)));
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

    pub(crate) fn clear(&mut self) {
        self.non_terminal_definitions.clear();
        self.non_terminals.clear();
        self.user_type_definitions.clear();
        self.user_types.clear();
        self.productions.clear();
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
        Ok(())
    }

    /// Semantic action for non-terminal 'Declaration'
    fn declaration(&mut self, arg: &Declaration) -> Result<()> {
        if let Declaration::Declaration2(user_type_def) = arg {
            let token = &user_type_def.identifier.identifier;
            let range = self.add_user_type_definition(token);
            self.add_user_type_ref(range, token);
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
        let nt = arg.production_l_h_s.identifier.identifier.symbol.clone();
        let rng: Rng = arg.into();
        eprintln!("Adding production {nt:?}: {rng:?}");
        let entry = self.productions.entry(nt).or_default();
        entry.push(arg.clone());
        eprintln!("Length: {}", entry.len());
        Ok(())
    }

    /// Semantic action for non-terminal 'NonTerminal'
    fn non_terminal(&mut self, arg: &NonTerminal) -> Result<()> {
        let token = &arg.identifier.identifier;
        let range = location_to_range(&token.location);
        self.add_non_terminal_ref(range, token);
        Ok(())
    }

    /// Semantic action for non-terminal 'UserTypeDeclaration'
    fn user_type_declaration(&mut self, arg: &UserTypeDeclaration) -> Result<()> {
        let token = &arg.user_type_name.identifier.identifier;
        let range = location_to_range(&token.location);
        self.add_user_type_ref(range, token);
        Ok(())
    }
}
