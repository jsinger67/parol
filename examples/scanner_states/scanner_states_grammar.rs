use crate::scanner_states_grammar_trait::ScannerStatesGrammarTrait;
use parol_macros::{bail, parol};
use parol_runtime::Result;
use parol_runtime::{log::trace, ParseTreeType};
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
    fn identifier(&mut self, identifier: &ParseTreeType<'_>) -> Result<()> {
        let context = "identifier";
        let id = identifier.text()?;
        self.push(ScannerStatesGrammarItem::Identifier(id.to_owned()), context);
        Ok(())
    }

    /// Semantic action for production 11:
    ///
    /// Escaped: <String>"\u{5c}[\u{22}\u{5c}bfnt]";
    ///
    fn escaped(&mut self, escaped: &ParseTreeType<'_>) -> Result<()> {
        let context = "escaped";
        if let Some(ScannerStatesGrammarItem::String(mut s)) = self.pop(context) {
            let mut element = escaped.text()?.to_string();
            element.remove(0);
            trace!("Escaped: {}", element);
            match element.as_str() {
                "\u{5c}" | "\u{22}" => s.push_str(&element),
                "b" => s.push('\u{8}'), // Backspace
                "f" => s.push('\u{c}'), // Formfeed
                "n" => s.push('\n'),    // Newline
                "t" => s.push('\t'),    // Tab
                _ => bail!("Unhandled escape sequence"),
            }
            self.push(ScannerStatesGrammarItem::String(s), context);
            Ok(())
        } else {
            Err(parol!("{}: Expected 'String' on TOS.", context))
        }
    }

    /// Semantic action for production 13:
    ///
    /// NoneQuote: <String>"[^\u{22}\u{5c}]+";
    ///
    fn none_quote(&mut self, none_quote: &ParseTreeType<'_>) -> Result<()> {
        let context = "none_quote";
        if let Some(ScannerStatesGrammarItem::String(mut s)) = self.pop(context) {
            let element = none_quote.text()?;
            s.push_str(element);
            self.push(ScannerStatesGrammarItem::String(s), context);
            Ok(())
        } else {
            Err(parol!("{}: Expected 'String' on TOS.", context))
        }
    }

    /// Semantic action for production 14:
    ///
    /// StringDelimiter: <INITIAL, String>"\u{22}";
    ///
    fn string_delimiter(&mut self, _string_delimiter: &ParseTreeType<'_>) -> Result<()> {
        let context = "string_delimiter";
        if self.in_string {
            self.in_string = false;
        } else {
            self.in_string = true;
            self.push(ScannerStatesGrammarItem::String(String::new()), context);
        }
        Ok(())
    }
}
