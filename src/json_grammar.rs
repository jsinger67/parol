use crate::json_grammar_trait::{JsonGrammarTrait, Json};
use miette::{Result};

///
/// Data structure used to build up a json structure during parsing
///
#[derive(Debug, Default)]
pub struct JsonGrammar {
    pub json: Option<Json>,
}

impl JsonGrammar {
    pub fn new() -> Self {
        JsonGrammar::default()
    }
}

impl JsonGrammarTrait for JsonGrammar {
    fn json(&mut self, arg: Json) -> Result<()> {
        self.json = Some(arg);
        Ok(())
    }
}
