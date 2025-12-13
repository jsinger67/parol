use crate::list_grammar_trait::{Items, List, ListGrammarTrait, ListOpt};
use parol_runtime::Result;
use parol_runtime::lexer::Token;
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

/// User defined type for a single number
#[derive(Clone, Debug, Default)]
pub struct Number(u32);

impl<'t> TryFrom<&Token<'t>> for Number {
    type Error = anyhow::Error;

    fn try_from(number: &Token<'t>) -> std::result::Result<Self, Self::Error> {
        match number.text().parse::<u32>() {
            Ok(num) => Ok(Self(num)),
            Err(e) => {
                let context = format!("'{}' at {}", number.text(), number.location);
                Err(anyhow::Error::new(e).context(context))
            }
        }
    }
}

/// User defined type for a vector of number
#[derive(Clone, Debug, Default)]
pub struct Numbers(Vec<u32>);

impl Display for Numbers {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl TryFrom<&Items> for Numbers {
    type Error = anyhow::Error;

    fn try_from(items: &Items) -> std::result::Result<Self, Self::Error> {
        Ok(Self(items.items_list.iter().fold(
            vec![items.num.num.0],
            |mut acc, e| {
                acc.push(e.num.num.0);
                acc
            },
        )))
    }
}

impl Display for List {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        if let Some(list) = &self.list_opt {
            write!(f, "[{}]", list)
        } else {
            write!(f, "[]")
        }
    }
}

impl Display for ListOpt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(f, "{}", self.items)
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
    /// Semantic action for non-terminal 'List'
    fn list(&mut self, arg: &List) -> Result<()> {
        self.list = Some(arg.clone());
        Ok(())
    }
}
