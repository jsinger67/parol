use crate::list_grammar_trait::ListGrammarTrait;
use id_tree::Tree;
use log::trace;
use parol_runtime::parser::errors::*;
use parol_runtime::parser::{ParseTreeStackEntry, ParseTreeType};
use std::fmt::{Debug, Display, Error, Formatter};

///
/// The value range for the supported list elements
///
pub type DefinitionRange = usize;

///
/// Data structure used to build up a list during parsing
///
#[derive(Debug, Clone)]
pub enum ListGrammarItem {
    Num(DefinitionRange),
    List(Vec<DefinitionRange>),
}

impl Display for ListGrammarItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match self {
            Self::Num(n) => write!(f, "{}", n),
            Self::List(l) => {
                write!(
                    f,
                    "[{}]",
                    l.iter()
                        .map(|e| format!("{}", e))
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            }
        }
    }
}

///
/// Data structure that implements the semantic actions for our list grammar
///
#[derive(Debug, Default)]
pub struct ListGrammar {
    pub item_stack: Vec<ListGrammarItem>,
}

impl ListGrammar {
    pub fn new() -> Self {
        ListGrammar::default()
    }

    fn push(&mut self, item: ListGrammarItem, context: &str) {
        trace!("push   {}: {}", context, item);
        self.item_stack.push(item)
    }

    fn pop(&mut self, context: &str) -> Option<ListGrammarItem> {
        if !self.item_stack.is_empty() {
            let item = self.item_stack.pop();
            if let Some(ref item) = item {
                trace!("pop    {}: {}", context, item);
            }
            item
        } else {
            None
        }
    }
}

impl Display for ListGrammar {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        writeln!(
            f,
            "{}",
            self.item_stack
                .iter()
                .map(|e| format!("{}", e))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

impl ListGrammarTrait for ListGrammar {
    /// Semantic action for production 0:
    ///
    /// list: ;
    ///
    fn list_0(&mut self, _parse_tree: &Tree<ParseTreeType>) -> Result<()> {
        let context = "list_0";
        // This is the empty list case
        self.push(ListGrammarItem::List(Vec::new()), context);
        Ok(())
    }

    /// Semantic action for production 1:
    ///
    /// list: num list_rest;
    ///
    fn list_1(
        &mut self,
        _num_0: &ParseTreeStackEntry,
        _list_rest_1: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "list_1";
        let top_of_stack1 = self.pop(context);
        let top_of_stack2 = self.pop(context);
        match (&top_of_stack1, &top_of_stack2) {
            (Some(ListGrammarItem::List(list)), Some(ListGrammarItem::Num(num))) => {
                let mut list = list.clone();
                list.push(*num);
                // Due to using push our result list has an inverse ordering.
                // Correct it here.
                list.reverse();
                self.push(ListGrammarItem::List(list.to_vec()), context);
                Ok(())
            }
            _ => Err(format!(
                "{}: unexpected ({:?}, {:?}",
                context, top_of_stack1, top_of_stack2
            )
            .into()),
        }
    }

    /// Semantic action for production 2:
    ///
    /// list_rest: list_item list_rest;
    ///
    fn list_rest_2(
        &mut self,
        _list_item_0: &ParseTreeStackEntry,
        _list_rest_1: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "list_rest_2";
        let top_of_stack1 = self.pop(context);
        let top_of_stack2 = self.pop(context);
        match (&top_of_stack1, &top_of_stack2) {
            (Some(ListGrammarItem::List(list)), Some(ListGrammarItem::Num(num))) => {
                let mut list = list.clone();
                list.push(*num);
                self.push(ListGrammarItem::List(list.to_vec()), context);
                Ok(())
            }
            _ => Err(format!(
                "{}: unexpected ({:?}, {:?}",
                context, top_of_stack1, top_of_stack2
            )
            .into()),
        }
    }

    /// Semantic action for production 4:
    ///
    /// list_rest: ;
    ///
    fn list_rest_4(&mut self, _parse_tree: &Tree<ParseTreeType>) -> Result<()> {
        let context = "list_rest_4";
        // Start here with an empty list
        self.push(ListGrammarItem::List(Vec::new()), context);
        Ok(())
    }

    /// Semantic action for production 5:
    ///
    /// list_rest: ",";
    ///
    fn list_rest_5(
        &mut self,
        _comma_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "list_rest_5";
        // Start here with an empty list
        self.push(ListGrammarItem::List(Vec::new()), context);
        Ok(())
    }

    /// Semantic action for production 6:
    ///
    /// num: "\d+";
    ///
    fn num_6(
        &mut self,
        num_0: &ParseTreeStackEntry,
        parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "num_6";
        let symbol = num_0.symbol(parse_tree)?;
        let number = symbol.parse::<DefinitionRange>().chain_err(|| {
            format!(
                "{}: Error accessing token from ParseTreeStackEntry",
                context
            )
        })?;
        self.push(ListGrammarItem::Num(number), context);
        Ok(())
    }
}
