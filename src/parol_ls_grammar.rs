use crate::{
    parol_ls_grammar_trait::{
        Declaration, NonTerminal, ParolLsGrammarTrait, Production, UserTypeDeclaration,
    },
    utils::location_to_range,
};
use lsp_types::Range;
#[allow(unused_imports)]
use miette::Result;
use std::{collections::HashMap, fmt::Debug};

///
/// Data structure that implements the semantic actions for our ParolLs grammar
///
#[derive(Debug, Default)]
pub struct ParolLsGrammar {
    pub non_terminal_definitions: HashMap<String, Vec<Range>>,
    pub non_terminals: Vec<(Range, String)>,

    pub user_type_definitions: HashMap<String, Vec<Range>>,
    pub user_types: Vec<(Range, String)>,
}

impl ParolLsGrammar {
    pub fn new() -> Self {
        ParolLsGrammar::default()
    }
}

impl ParolLsGrammarTrait for ParolLsGrammar {
    /// Semantic action for non-terminal 'Declaration'
    fn declaration(&mut self, arg: &Declaration) -> Result<()> {
        if let Declaration::Declaration2(user_type_def) = arg {
            let token = &user_type_def.identifier.identifier;
            let entry = self
                .user_type_definitions
                .entry(token.symbol.clone())
                .or_default();
            let range = location_to_range(&token.location);
            entry.push(range);
            self.user_types.push((range, token.symbol.clone()));
        }
        Ok(())
    }

    /// Semantic action for non-terminal 'Production'
    fn production(&mut self, arg: &Production) -> Result<()> {
        let token = &arg.identifier.identifier;
        let entry = self
            .non_terminal_definitions
            .entry(token.symbol.clone())
            .or_default();
        let range = location_to_range(&token.location);
        entry.push(range);
        self.non_terminals.push((range, token.symbol.clone()));
        Ok(())
    }

    /// Semantic action for non-terminal 'NonTerminal'
    fn non_terminal(&mut self, arg: &NonTerminal) -> Result<()> {
        let token = &arg.identifier.identifier;
        let range = location_to_range(&token.location);
        self.non_terminals.push((range, token.symbol.clone()));
        Ok(())
    }

    /// Semantic action for non-terminal 'UserTypeDeclaration'
    fn user_type_declaration(&mut self, arg: &UserTypeDeclaration) -> Result<()> {
        let token = &arg.user_type_name.identifier.identifier;
        let range = location_to_range(&token.location);
        self.user_types.push((range, token.symbol.clone()));
        Ok(())
    }
}
