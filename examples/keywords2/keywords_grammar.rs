use crate::keywords_grammar_trait::KeywordsGrammarTrait;
use std::fmt::{Debug, Display, Error, Formatter};

///
/// Data structure that implements the semantic actions for our keywords grammar
///
#[derive(Debug, Default)]
pub struct KeywordsGrammar;

impl KeywordsGrammar {
    pub fn new() -> Self {
        KeywordsGrammar
    }
}

impl Display for KeywordsGrammar {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        writeln!(f, ":-)")
    }
}

///
/// We don't implement any semantic actions. We use the parser solely as acceptor.
///
impl KeywordsGrammarTrait for KeywordsGrammar {}
