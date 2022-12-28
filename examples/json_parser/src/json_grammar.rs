use crate::errors::JsonError;
use crate::json_grammar_trait::JsonGrammarTrait;
use anyhow::{anyhow, Result};
use id_tree::Tree;
use parol_runtime::errors::FileSource;
use parol_runtime::log::trace;
use parol_runtime::parser::{ParseTreeStackEntry, ParseTreeType};
use std::fmt::{Debug, Display, Error, Formatter};
use std::path::PathBuf;

///
/// Data structure used to build up a json structure item during parsing
///
#[derive(Debug, Clone)]
pub enum JsonGrammarItem {
    Null,
    True,
    False,
    String(String),
    Number(f64),
    Array(Vec<JsonGrammarItem>),
    Pair((String, Box<JsonGrammarItem>)),
    Object(Vec<JsonGrammarItem>),
}

impl Display for JsonGrammarItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match self {
            Self::Null => write!(f, "Null"),
            Self::True => write!(f, "True"),
            Self::False => write!(f, "False"),
            Self::String(s) => write!(f, "{}", s),
            Self::Number(n) => write!(f, "{}", n),
            Self::Array(v) => write!(
                f,
                "[{}]",
                v.iter()
                    .map(|e| format!("{}", e))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Self::Pair((s, v)) => write!(f, "{}: {}", s, v),
            Self::Object(p) => write!(
                f,
                "{{{}}}",
                p.iter()
                    .map(|e| format!("{}", e))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        }
    }
}

///
/// Data structure used to build up a json structure during parsing
///
#[derive(Debug, Default)]
pub struct JsonGrammar {
    pub item_stack: Vec<JsonGrammarItem>,
    file_name: PathBuf,
}

impl JsonGrammar {
    pub fn new() -> Self {
        JsonGrammar::default()
    }

    fn push(&mut self, item: JsonGrammarItem, context: &str) {
        trace!("push   {}: {}", context, item);
        self.item_stack.push(item)
    }

    fn pop(&mut self, context: &str) -> Option<JsonGrammarItem> {
        if !self.item_stack.is_empty() {
            let item = self.item_stack.pop();
            if let Some(ref item) = item {
                trace!("pop    {}: {}", context, item);
            }
            item
        } else {
            trace!("pop    {}: item_stack is empty", context);
            None
        }
    }

    #[allow(dead_code)]
    // Use this function for debugging purposes:
    // $env:RUST_LOG="json_parser::json_grammar=trace"
    // trace!("{}", self.trace_ast_stack(context));
    fn trace_ast_stack(&self, context: &str) -> String {
        format!(
            "Ast stack at {}:\n{}",
            context,
            self.item_stack
                .iter()
                .rev()
                .map(|s| format!("  {}", s))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

impl Display for JsonGrammar {
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

impl JsonGrammarTrait for JsonGrammar {
    /// Semantic action for production 1:
    ///
    /// Object: "\{" ObjectSuffix;
    ///
    fn object(
        &mut self,
        _l_brace: &ParseTreeStackEntry,
        _object_suffix: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "object";
        match self.pop(context) {
            Some(JsonGrammarItem::Object(mut pairs)) => {
                pairs.reverse();
                self.push(JsonGrammarItem::Object(pairs), context);
                Ok(())
            }
            _ => Err(anyhow!("{}: expecting Object on top of stack", context)),
        }
    }

    /// Semantic action for production 2:
    ///
    /// ObjectSuffix: Pair ObjectList /* Vec */ "\}";
    ///
    fn object_suffix_0(
        &mut self,
        _pair: &ParseTreeStackEntry,
        _object_list: &ParseTreeStackEntry,
        _r_brace: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "object_suffix_0";
        match (self.pop(context), self.pop(context)) {
            (Some(JsonGrammarItem::Object(mut pairs)), Some(pair)) => {
                pairs.push(pair);
                self.push(JsonGrammarItem::Object(pairs), context);
                Ok(())
            }
            _ => Err(anyhow!(
                "{}: expected Object, Pair on top of stack",
                context
            )),
        }
    }

    /// Semantic action for production 3:
    ///
    /// ObjectSuffix: "\}";
    ///
    fn object_suffix_1(
        &mut self,
        _r_brace: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "object_suffix_1";
        self.push(JsonGrammarItem::Object(Vec::new()), context);
        Ok(())
    }

    /// Semantic action for production 4:
    ///
    /// ObjectList /* Vec<T>::Push */: "," Pair ObjectList;
    ///
    fn object_list_0(
        &mut self,
        _comma: &ParseTreeStackEntry,
        _pair: &ParseTreeStackEntry,
        _object_list: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "object_list_0";
        match (self.pop(context), self.pop(context)) {
            (Some(JsonGrammarItem::Object(mut pairs)), Some(pair)) => {
                pairs.push(pair);
                self.push(JsonGrammarItem::Object(pairs), context);
                Ok(())
            }
            _ => Err(anyhow!(
                "{}: expected Object, Pair on top of stack",
                context,
            )),
        }
    }

    /// Semantic action for production 5:
    ///
    /// ObjectList /* Vec<T>::New */: ;
    ///
    fn object_list_1(&mut self, _parse_tree: &Tree<ParseTreeType>) -> Result<()> {
        let context = "object_list_1";
        self.push(JsonGrammarItem::Object(Vec::new()), context);
        Ok(())
    }

    /// Semantic action for production 6:
    ///
    /// Pair: String ":" Value;
    ///
    fn pair(
        &mut self,
        _string: &ParseTreeStackEntry,
        _colon: &ParseTreeStackEntry,
        _value: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "pair";
        match (self.pop(context), self.pop(context)) {
            (Some(value), Some(JsonGrammarItem::String(string))) => {
                self.push(JsonGrammarItem::Pair((string, Box::new(value))), context);
                Ok(())
            }
            _ => Err(anyhow!("{}: expected Value, Name on top of stack", context)),
        }
    }

    /// Semantic action for production 7:
    ///
    /// Array: "\[" ArraySuffix;
    ///
    fn array(
        &mut self,
        _l_bracket: &ParseTreeStackEntry,
        _array_suffix: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "array";
        match self.pop(context) {
            Some(JsonGrammarItem::Array(mut list)) => {
                list.reverse();
                self.push(JsonGrammarItem::Array(list), context);
                Ok(())
            }
            _ => Err(anyhow!("{}: Expecting Array on top of stack", context)),
        }
    }

    /// Semantic action for production 8:
    ///
    /// ArraySuffix: Value ArrayList /* Vec */ "\]";
    ///
    fn array_suffix_0(
        &mut self,
        _value: &ParseTreeStackEntry,
        _array_list: &ParseTreeStackEntry,
        _r_bracket: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "array_suffix_0";
        match (self.pop(context), self.pop(context)) {
            (Some(JsonGrammarItem::Array(mut array)), Some(elem)) => {
                array.push(elem);
                self.push(JsonGrammarItem::Array(array), context);
                Ok(())
            }
            _ => Err(anyhow!(
                "{}: expecting Array, Value on top of stack",
                context
            )),
        }
    }

    /// Semantic action for production 9:
    ///
    /// ArraySuffix: "\]";
    ///
    fn array_suffix_1(
        &mut self,
        _r_bracket: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "array_suffix_1";
        self.push(JsonGrammarItem::Array(Vec::new()), context);
        Ok(())
    }

    /// Semantic action for production 10:
    ///
    /// ArrayList /* Vec<T>::Push */: "," Value ArrayList;
    ///
    fn array_list_0(
        &mut self,
        _comma: &ParseTreeStackEntry,
        _value: &ParseTreeStackEntry,
        _array_list: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "array_list_0";
        match (self.pop(context), self.pop(context)) {
            (Some(JsonGrammarItem::Array(mut array)), Some(elem)) => {
                array.push(elem);
                self.push(JsonGrammarItem::Array(array), context);
                Ok(())
            }
            _ => Err(anyhow!(
                "{}: expecting Array, Value on top of stack",
                context
            )),
        }
    }

    /// Semantic action for production 11:
    ///
    /// ArrayList /* Vec<T>::New */: ;
    ///
    fn array_list_1(&mut self, _parse_tree: &Tree<ParseTreeType>) -> Result<()> {
        let context = "array_list_11";
        self.push(JsonGrammarItem::Array(Vec::new()), context);
        Ok(())
    }

    /// Semantic action for production 16:
    ///
    /// Value: "true";
    ///
    fn value_4(
        &mut self,
        _true: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "value_4";
        self.push(JsonGrammarItem::True, context);
        Ok(())
    }

    /// Semantic action for production 17:
    ///
    /// Value: "false";
    ///
    fn value_5(
        &mut self,
        _false: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "value_5";
        self.push(JsonGrammarItem::False, context);
        Ok(())
    }

    /// Semantic action for production 18:
    ///
    /// Value: "null";
    ///
    fn value_6(
        &mut self,
        _null: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "value_6";
        self.push(JsonGrammarItem::Null, context);
        Ok(())
    }

    /// Semantic action for production 19:
    ///
    /// String: "\u{0022}(?:\\[\u{0022}\\/bfnrt]|u[0-9a-fA-F]{4}|[^\u{0022}\\\u0000-\u001F])*\u{0022}";
    ///
    fn string(
        &mut self,
        string: &ParseTreeStackEntry,
        parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "string";
        let string = string.text(parse_tree)?;
        self.push(JsonGrammarItem::String(string.to_string()), context);
        Ok(())
    }

    /// Semantic action for production 20:
    ///
    /// Number: "-?(?:0|[1-9][0-9]*)(?:\.[0-9]+)?(?:[eE][-+]?(?:0|[1-9][0-9]*)?)?";
    ///
    fn number(
        &mut self,
        number: &ParseTreeStackEntry,
        parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "number_20";
        let number = match number.text(parse_tree)?.parse::<f64>() {
            Ok(number) => number,
            Err(source) => {
                return Err(anyhow!(JsonError::ParseF64Failed {
                    input: FileSource::try_new(self.file_name.clone())?,
                    token: number.token(parse_tree)?.into(),
                    source
                }))
            }
        };

        self.push(JsonGrammarItem::Number(number), context);
        Ok(())
    }
}
