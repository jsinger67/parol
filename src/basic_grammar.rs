use crate::basic_grammar_trait::{ Basic, BasicGrammarTrait };
#[allow(unused_imports)]
use miette::Result;
use std::fmt::{Debug, Display, Error, Formatter};

///
/// Data structure that implements the semantic actions for our Basic grammar
/// !Change this type as needed!
///
#[derive(Debug, Default)]
pub struct BasicGrammar<'t> {
    pub basic: Option<Basic<'t>>,
}

impl BasicGrammar<'_> {
    pub fn new() -> Self {
        BasicGrammar::default()
    }
}

impl Display for Basic<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(f, ":-)")
    }
}

impl Display for BasicGrammar<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match &self.basic {
            Some(basic) => writeln!(f, "{}", basic),
            None => write!(f, "No parse result"),
        }
    }
}

impl<'t> BasicGrammarTrait<'t> for BasicGrammar<'t> {
    // Your implementation starts here
}
