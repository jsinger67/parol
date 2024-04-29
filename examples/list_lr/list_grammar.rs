use crate::list_grammar_trait::{Items, List, ListGrammarTrait};
#[allow(unused_imports)]
use parol_runtime::{Result, Token};
use std::fmt::{Debug, Display, Error, Formatter};

///
/// Data structure that implements the semantic actions for our List grammar
///
#[derive(Debug, Default)]
pub struct ListGrammar<'t> {
    pub list: Option<List>,
    _phantom: std::marker::PhantomData<&'t ()>,
}

impl ListGrammar<'_> {
    pub fn new() -> Self {
        ListGrammar::default()
    }
}

impl Display for List {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        if let Some(list) = &self.list_opt {
            write!(f, "[{}]", list.items)
        } else {
            write!(f, "[]")
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

impl<'t> ListGrammarTrait for ListGrammar<'t> {
    /// Semantic action for non-terminal 'List'
    fn list(&mut self, arg: &List) -> Result<()> {
        self.list = Some(arg.clone());
        Ok(())
    }
}

/// User defined type for a single number
#[derive(Clone, Debug, Default)]
pub struct Number(u32);

impl<'t> TryFrom<&Token<'t>> for Number {
    type Error = anyhow::Error;

    fn try_from(number: &Token<'t>) -> std::result::Result<Self, Self::Error> {
        Ok(Self(number.text().parse::<u32>()?))
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
