use parol_runtime::Result;
use std::fmt::{Debug, Display, Error, Formatter};

use crate::allow_unmatched_grammar_trait::{AllowUnmatchedGrammarTrait, Expr};

///
/// Data structure that implements the semantic actions for our list grammar
///
#[derive(Debug, Default)]
pub struct AllowUnmatchedGrammar<'t> {
    pub expr: Option<Expr<'t>>,
}

impl<'t> AllowUnmatchedGrammar<'t> {
    pub fn new() -> Self {
        AllowUnmatchedGrammar::default()
    }
}

impl Display for AllowUnmatchedGrammar<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match &self.expr {
            Some(expr) => writeln!(f, "{expr:?}"),
            None => write!(f, "No parse result"),
        }
    }
}

impl<'t> AllowUnmatchedGrammarTrait<'t> for AllowUnmatchedGrammar<'t> {
    /// Semantic action for non-terminal 'Expr'
    fn expr(&mut self, arg: &Expr<'t>) -> Result<()> {
        self.expr = Some(arg.clone());
        Ok(())
    }
}
