use crate::list_grammar_trait::{List, ListGrammarTrait, ListList};
use miette::Result;
use std::fmt::{Debug, Display, Error, Formatter};

///
/// Data structure that implements the semantic actions for our list grammar
///
#[derive(Debug, Default)]
pub struct ListGrammar {
    pub list: Option<List>,
}

impl ListGrammar {
    pub fn new() -> Self {
        ListGrammar::default()
    }
}

impl Display for List {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match &self {
            List::List0(l) => write!(
                f,
                "[{}{}]",
                l.num_0.num_0.symbol,
                l.list_list_1
                    .iter()
                    .map(|e| format!("{}", e))
                    .collect::<Vec<std::string::String>>()
                    .join("")
            ),
            List::List3(_) => write!(f, "[]"),
        }
    }
}

impl Display for ListList {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(f, "{} {}", self.comma_0.symbol, self.num_1.num_0.symbol)
    }
}

impl Display for ListGrammar {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match &self.list {
            Some(list) => writeln!(f, "{}", list),
            None => write!(f, "No parse result"),
        }
    }
}

impl ListGrammarTrait for ListGrammar {
    /// Semantic action for user production 0:
    ///
    /// List: [Num {<0>"," Num}];
    ///
    fn list(&mut self, arg: &List) -> Result<()> {
        self.list = Some(arg.clone());
        Ok(())
    }
}
