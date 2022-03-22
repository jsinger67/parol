use crate::calc_grammar_trait::{Calc, CalcGrammarTrait};
use miette::Result;
use std::fmt::{Debug, Display, Error, Formatter};

///
/// Data structure that implements the semantic actions for our calc grammar
///
#[derive(Debug, Default)]
pub struct CalcGrammar {
    pub calc: Option<Calc>,
}

impl CalcGrammar {
    pub fn new() -> Self {
        CalcGrammar::default()
    }
}

impl Display for CalcGrammar {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match &self.calc {
            Some(calc) => writeln!(f, "{:#?}", calc),
            None => write!(f, "No parse result"),
        }
    }
}

impl CalcGrammarTrait for CalcGrammar {
    /// Semantic action for user production 0:
    ///
    /// calc: {instruction <0>";"};
    ///
    fn calc(&mut self, arg: Calc) -> Result<()> {
        self.calc = Some(arg);
        Ok(())
    }

}
