use parol_runtime::Token;

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

    pub fn on_comment(&mut self, _token: Token<'_>) {}
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
