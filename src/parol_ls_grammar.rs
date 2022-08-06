use crate::{
    parol_ls_grammar_trait::{
        Declaration, NonTerminal, ParolLsGrammarTrait, Production, StartDeclaration,
        UserTypeDeclaration,
    },
    utils::location_to_range,
};
use lsp_types::{Position, Range};
#[allow(unused_imports)]
use miette::Result;
use parol_runtime::lexer::OwnedToken;
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
}

impl ParolLsGrammarTrait for ParolLsGrammar {
    /// Semantic action for non-terminal 'StartDeclaration'
    fn start_declaration(&mut self, arg: &StartDeclaration) -> Result<()> {
        let token = &arg.identifier.identifier;
        let range = location_to_range(&token.location);
        self.add_non_terminal_ref(range, &token);
        Ok(())
    }

    /// Semantic action for non-terminal 'Declaration'
    fn declaration(&mut self, arg: &Declaration) -> Result<()> {
        if let Declaration::Declaration2(user_type_def) = arg {
            let token = &user_type_def.identifier.identifier;
            let range = self.add_user_type_definition(token);
            self.add_user_type_ref(range, &token);
        }
        Ok(())
    }

    /// Semantic action for non-terminal 'Production'
    fn production(&mut self, arg: &Production) -> Result<()> {
        let token = &arg.identifier.identifier;
        let range = self.add_non_terminal_definition(token);
        self.add_non_terminal_ref(range, &token);
        Ok(())
    }

    /// Semantic action for non-terminal 'NonTerminal'
    fn non_terminal(&mut self, arg: &NonTerminal) -> Result<()> {
        let token = &arg.identifier.identifier;
        let range = location_to_range(&token.location);
        self.add_non_terminal_ref(range, &token);
        Ok(())
    }

    /// Semantic action for non-terminal 'UserTypeDeclaration'
    fn user_type_declaration(&mut self, arg: &UserTypeDeclaration) -> Result<()> {
        let token = &arg.user_type_name.identifier.identifier;
        let range = location_to_range(&token.location);
        self.add_user_type_ref(range, &token);
        Ok(())
    }
}
