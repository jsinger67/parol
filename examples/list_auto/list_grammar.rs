use crate::list_grammar_trait::{
    List, ListGrammarTrait, ListOpt, ListOpt0, ListOptList, TrailingComma, TrailingCommaOpt,
    TrailingCommaOpt0,
};
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
        write!(f, "[{}{}]", self.list_opt, self.trailing_comma)
    }
}

impl Display for ListOpt<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        if let ListOpt::ListOpt0(ListOpt0 { num, list_opt_list }) = self {
            write!(
                f,
                "{}{}",
                num.num.symbol,
                list_opt_list
                    .iter()
                    .map(|e| format!("{}", e))
                    .collect::<Vec<std::string::String>>()
                    .join("")
            )
        } else {
            Ok(())
        }
    }
}

impl Display for ListOptList<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(f, "{} {}", self.comma.symbol, self.num.num.symbol)
    }
}

impl Display for TrailingComma<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        if let TrailingCommaOpt::TrailingCommaOpt0(TrailingCommaOpt0 { comma }) =
            *self.trailing_comma_opt
        {
            write!(f, "{}", comma.symbol)
        } else {
            Ok(())
        }
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
