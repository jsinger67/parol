use crate::parol_ls_grammar_trait::{ParolLs, ParolLsGrammarTrait};
#[allow(unused_imports)]
use miette::Result;
use std::fmt::{Debug, Display, Error, Formatter};

///
/// Data structure that implements the semantic actions for our ParolLs grammar
/// !Change this type as needed!
///
#[derive(Debug, Default)]
pub struct ParolLsGrammar<'t> {
    pub parol_ls: Option<ParolLs<'t>>,
}

impl ParolLsGrammar<'_> {
    pub fn new() -> Self {
        ParolLsGrammar::default()
    }
}

impl Display for ParolLs<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(f, "{:?}", self)
    }
}

impl Display for ParolLsGrammar<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match &self.parol_ls {
            Some(parol_ls) => writeln!(f, "{}", parol_ls),
            None => write!(f, "No parse result"),
        }
    }
}

impl<'t> ParolLsGrammarTrait<'t> for ParolLsGrammar<'t> {
    // !Adjust your implementation as needed!

    /// Semantic action for non-terminal 'ParolLs'
    fn parol_ls(&mut self, arg: &ParolLs<'t>) -> Result<()> {
        self.parol_ls = Some(arg.clone());
        Ok(())
    }
}
