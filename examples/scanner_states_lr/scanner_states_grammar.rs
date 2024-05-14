use crate::scanner_states_grammar_trait::{ScannerStatesGrammarTrait, Start};
#[allow(unused_imports)]
use parol_runtime::{Result, Token};
use std::fmt::{Debug, Display, Error, Formatter};

///
/// Data structure that implements the semantic actions for our Start grammar
/// !Change this type as needed!
///
#[derive(Debug, Default)]
pub struct ScannerStatesGrammar<'t> {
    pub start: Option<Start<'t>>,
}

impl ScannerStatesGrammar<'_> {
    pub fn new() -> Self {
        ScannerStatesGrammar::default()
    }
}

impl Display for Start<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(f, "{:?}", self)
    }
}

impl Display for ScannerStatesGrammar<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match &self.start {
            Some(start) => writeln!(f, "{}", start),
            None => write!(f, "No parse result"),
        }
    }
}

impl<'t> ScannerStatesGrammarTrait<'t> for ScannerStatesGrammar<'t> {
    // !Adjust your implementation as needed!

    /// Semantic action for non-terminal 'Start'
    fn start(&mut self, arg: &Start<'t>) -> Result<()> {
        self.start = Some(arg.clone());
        Ok(())
    }
}
