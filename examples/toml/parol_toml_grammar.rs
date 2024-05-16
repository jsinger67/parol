use crate::parol_toml_grammar_trait::{ParolToml, ParolTomlGrammarTrait};
#[allow(unused_imports)]
use parol_runtime::Result;
use std::fmt::{Debug, Display, Error, Formatter};

///
/// Data structure that implements the semantic actions for our ParolToml grammar
/// !Change this type as needed!
///
#[derive(Debug, Default)]
pub struct ParolTomlGrammar<'t> {
    pub parol_toml: Option<ParolToml<'t>>,
}

impl ParolTomlGrammar<'_> {
    pub fn new() -> Self {
        ParolTomlGrammar::default()
    }
}

impl Display for ParolToml<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(f, "{:?}", self)
    }
}

impl Display for ParolTomlGrammar<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match &self.parol_toml {
            Some(parol_toml) => writeln!(f, "{}", parol_toml),
            None => write!(f, "No parse result"),
        }
    }
}

impl<'t> ParolTomlGrammarTrait<'t> for ParolTomlGrammar<'t> {
    // !Adjust your implementation as needed!

    /// Semantic action for non-terminal 'ParolToml'
    fn parol_toml(&mut self, arg: &ParolToml<'t>) -> Result<()> {
        self.parol_toml = Some(arg.clone());
        Ok(())
    }
}
