use crate::scanner_states_grammar_trait::ScannerStatesGrammarTrait;
use id_tree::Tree;
use log::trace;
use parol_runtime::errors::*;
use parol_runtime::parser::{ParseTreeStackEntry, ParseTreeType};
use std::fmt::{Debug, Display, Error, Formatter};

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
    /// Semantic action for production 10:
    ///
    /// Identifier: "[a-zA-Z_]\w*";
    ///
    fn identifier_10(
        &mut self,
        identifier_0: &ParseTreeStackEntry,
        parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "identifier_10";
        let id = identifier_0.symbol(parse_tree)?;
        self.push(ScannerStatesGrammarItem::Identifier(id.clone()), context);
        Ok(())
    }

    /// Semantic action for production 11:
    ///
    /// Escaped: <String>"\u{5c}[\u{22}\u{5c}bfnt]";
    ///
    fn escaped_11(
        &mut self,
        escaped_0: &ParseTreeStackEntry,
        parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "escaped_11";
        if let Some(ScannerStatesGrammarItem::String(mut s)) = self.pop(context) {
            let mut element = escaped_0.symbol(parse_tree)?.to_string();
            element.remove(0);
            trace!("Escaped: {}", element);
            match element.as_str() {
                "\u{5c}" | "\u{22}" => s.push_str(&element),
                "b" => s.push('\u{8}'), // Backspace
                "f" => s.push('\u{c}'), // Formfeed
                "n" => s.push('\n'),    // Newline
                "t" => s.push('\t'),    // Tab
                _ => return Err("Unhandled escape sequence".into()),
            }
            self.push(ScannerStatesGrammarItem::String(s), context);
            Ok(())
        } else {
            Err(format!("{}: Expected 'String' on TOS.", context).into())
        }
    }

    /// Semantic action for production 13:
    ///
    /// NoneQuote: <String>"[^\u{22}\u{5c}]+";
    ///
    fn none_quote_13(
        &mut self,
        none_quote_0: &ParseTreeStackEntry,
        parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "none_quote_13";
        if let Some(ScannerStatesGrammarItem::String(mut s)) = self.pop(context) {
            let element = none_quote_0.symbol(parse_tree)?;
            s.push_str(element);
            self.push(ScannerStatesGrammarItem::String(s), context);
            Ok(())
        } else {
            Err(format!("{}: Expected 'String' on TOS.", context).into())
        }
    }

    /// Semantic action for production 14:
    ///
    /// StringDelimiter: <INITIAL, String>"\u{22}";
    ///
    fn string_delimiter_14(
        &mut self,
        _string_delimiter_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "string_delimiter_14";
        if self.in_string {
            self.in_string = false;
        } else {
            self.in_string = true;
            self.push(ScannerStatesGrammarItem::String(String::new()), context);
        }
        Ok(())
    }
}
