use crate::keywords_grammar_trait::KeywordsGrammarTrait;
use std::fmt::{Debug, Display, Error, Formatter};
use std::marker::PhantomData;

///
/// Data structure that implements the semantic actions for our keywords grammar
///
#[derive(Debug, Default)]
pub struct KeywordsGrammar<'t> {
    // Just to hold the lifetime generated by parol
    phantom: PhantomData<&'t str>,
}

impl KeywordsGrammar<'_> {
    pub fn new() -> Self {
        KeywordsGrammar {
            phantom: PhantomData,
        }
    }
}

impl Display for KeywordsGrammar<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        writeln!(f, ":-)")
    }
}

///
/// We don't implement any semantic actions. We use the parser solely as acceptor.
///
impl<'t> KeywordsGrammarTrait<'t> for KeywordsGrammar<'t> {}
