use crate::list_grammar_trait::{List, ListGrammarTrait, ListList};
use miette::Result;
use std::fmt::{Debug, Display, Error, Formatter};

///
/// Data structure that implements the semantic actions for our list grammar
///
#[derive(Debug, Default)]
pub struct ListGrammar<'t> {
    pub list: Option<List<'t>>,
}

impl ListGrammar<'_> {
    pub fn new() -> Self {
        ListGrammar::default()
    }
}

impl Display for List<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match &self {
            List::List0(l) => write!(
                f,
                "[{}{}]",
                l.num.num.symbol,
                l.list_list
                    .iter()
                    .map(|e| format!("{}", e))
                    .collect::<Vec<std::string::String>>()
                    .join("")
            ),
            List::List1(_) => write!(f, "[]"),
        }
    }
}

impl Display for ListList<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(f, "{} {}", self.comma.symbol, self.num.num.symbol)
    }
}

impl Display for ListGrammar<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match &self.list {
            Some(list) => writeln!(f, "{}", list),
            None => write!(f, "No parse result"),
        }
    }
}

impl<'t> ListGrammarTrait<'t> for ListGrammar<'t> {
    /// Semantic action for user production 0:
    ///
    /// List: [Num {<0>"," Num}];
    ///
    fn list(&mut self, arg: &List<'t>) -> Result<()> {
        self.list = Some(arg.clone());
        Ok(())
    }
}
