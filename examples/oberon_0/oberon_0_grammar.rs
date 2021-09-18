use crate::oberon_0_grammar_trait::Oberon0GrammarTrait;
use log::trace;
use std::fmt::{Debug, Display, Error, Formatter};

///
/// Data structure used to build up a oberon_0 structure item during parsing
///
#[derive(Debug, Clone)]
pub enum Oberon0GrammarItem {}

impl Display for Oberon0GrammarItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(f, "TODO!")
    }
}

///
/// Data structure used to build up a oberon_0 structure during parsing
///
#[derive(Debug, Default)]
pub struct Oberon0Grammar {
    pub ast_stack: Vec<Oberon0GrammarItem>,
}

impl Oberon0Grammar {
    pub fn new() -> Self {
        Oberon0Grammar::default()
    }

    fn _push(&mut self, item: Oberon0GrammarItem, context: &str) {
        trace!("push    {}: {}", context, item);
        self.ast_stack.push(item)
    }

    fn _pop(&mut self, context: &str) -> Option<Oberon0GrammarItem> {
        if !self.ast_stack.is_empty() {
            let item = self.ast_stack.pop();
            if let Some(ref item) = item {
                trace!("pop     {}: {}", context, item);
            }
            item
        } else {
            None
        }
    }
}

impl Display for Oberon0Grammar {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        writeln!(f, "")
    }
}

impl Oberon0GrammarTrait for Oberon0Grammar {}
