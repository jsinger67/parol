use crate::json_grammar_trait::JsonGrammarTrait;
use std::fmt::{Debug, Display, Error, Formatter};

///
/// Data structure used to build up a json structure item during parsing
///
#[derive(Debug, Clone)]
pub enum JsonGrammarItem {}

impl Display for JsonGrammarItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(f, "TODO!")
    }
}

///
/// Data structure used to build up a json structure during parsing
///
#[derive(Debug, Default)]
pub struct JsonGrammar {
    pub ast_stack: Vec<JsonGrammarItem>,
}

impl JsonGrammar {
    pub fn new() -> Self {
        JsonGrammar::default()
    }
}

impl Display for JsonGrammar {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        writeln!(f, "")
    }
}

impl JsonGrammarTrait for JsonGrammar {}
