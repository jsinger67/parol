use crate::scanner_states_grammar_trait::ScannerStatesGrammarTrait;
use id_tree::Tree;
use log::trace;
use parol_runtime::errors::*;
use parol_runtime::parser::ScannerAccess;
use parol_runtime::parser::{ParseTreeStackEntry, ParseTreeType};
use std::cell::RefMut;
use std::fmt::{Debug, Display, Error, Formatter};

///
/// The value range for the supported scanner_states elements
///
pub type DefinitionRange = usize;

///
/// Data structure used to build up a scanner_states during parsing
///
#[derive(Debug, Clone)]
pub enum ScannerStatesGrammarItem {
    Identifier(String),
    String(String),
}

impl Display for ScannerStatesGrammarItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match self {
            Self::Identifier(i) => write!(f, "{}", i),
            Self::String(s) => write!(f, "\"{}\"", s),
        }
    }
}

///
/// Data structure that implements the semantic actions for our scanner_states grammar
///
#[derive(Debug, Default)]
pub struct ScannerStatesGrammar {
    in_string: bool,
    pub item_stack: Vec<ScannerStatesGrammarItem>,
}

impl ScannerStatesGrammar {
    pub fn new() -> Self {
        ScannerStatesGrammar::default()
    }

    fn push(&mut self, item: ScannerStatesGrammarItem, context: &str) {
        trace!("push   {}: {}", context, item);
        self.item_stack.push(item)
    }

    fn pop(&mut self, context: &str) -> Option<ScannerStatesGrammarItem> {
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

impl Display for ScannerStatesGrammar {
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

impl ScannerStatesGrammarTrait for ScannerStatesGrammar {
    /// Semantic action for production 21:
    ///
    /// StringDelimiter: "\u{22}";
    ///
    fn string_delimiter_21(
        &mut self,
        _block_comment_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
        _scanner_access: RefMut<dyn ScannerAccess>,
        scanner_access: RefMut<dyn ScannerAccess>,
    ) -> Result<()> {
        if self.in_string {
            scanner_access.switch_scanner("INITIAL")?;
            self.in_string = false;
        } else {
            scanner_access.switch_scanner("String")?;
            self.in_string = true;
        }
        Ok(())
    }
}
