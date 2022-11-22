use crate::errors::JsonError;
use crate::json_grammar_trait::JsonGrammarTrait;
use id_tree::Tree;
use log::trace;
use miette::{miette, Result, WrapErr};
use parol_runtime::errors::FileSource;
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
    ///
    /// Information provided by parser
    ///
    fn init(&mut self, file_name: &std::path::Path) {
        self.file_name = file_name.into();
    }

    /// Semantic action for production 1:
    ///
    /// Object: "\{" ObjectSuffix1;
    ///
    fn object_1(
        &mut self,
        _l_brace_0: &ParseTreeStackEntry,
        _object_suffix1_1: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "object_1";
        match self.pop(context) {
            Some(JsonGrammarItem::Object(mut pairs)) => {
                pairs.reverse();
                self.push(JsonGrammarItem::Object(pairs), context);
                Ok(())
            }
            _ => Err(miette!("{}: expecting Object on top of stack", context)),
        }
    }

    /// Semantic action for production 2:
    ///
    /// ObjectSuffix: Pair ObjectList "\}";
    ///
    fn object_suffix_2(
        &mut self,
        _pair_0: &ParseTreeStackEntry,
        _object_list_1: &ParseTreeStackEntry,
        _r_brace_2: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "object_suffix_2";
        match (self.pop(context), self.pop(context)) {
            (Some(JsonGrammarItem::Object(mut pairs)), Some(pair)) => {
                pairs.push(pair);
                self.push(JsonGrammarItem::Object(pairs), context);
                Ok(())
            }
            _ => Err(miette!(
                "{}: expected Object, Pair on top of stack",
                context
            )),
        }
    }

    /// Semantic action for production 3:
    ///
    /// ObjectSuffix: "\}";
    ///
    fn object_suffix_3(
        &mut self,
        _r_brace_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "object_suffix_3";
        self.push(JsonGrammarItem::Object(Vec::new()), context);
        Ok(())
    }

    /// Semantic action for production 4:
    ///
    /// ObjectList: "," Pair ObjectList;
    ///
    fn object_list_4(
        &mut self,
        _comma_0: &ParseTreeStackEntry,
        _pair_1: &ParseTreeStackEntry,
        _object_list_2: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "object_list_4";
        match (self.pop(context), self.pop(context)) {
            (Some(JsonGrammarItem::Object(mut pairs)), Some(pair)) => {
                pairs.push(pair);
                self.push(JsonGrammarItem::Object(pairs), context);
                Ok(())
            }
            _ => Err(miette!(
                "{}: expected Object, Pair on top of stack",
                context,
            )),
        }
    }

    /// Semantic action for production 5:
    ///
    /// ObjectList: ;
    ///
    fn object_list_5(&mut self, _parse_tree: &Tree<ParseTreeType>) -> Result<()> {
        let context = "object_list_5";
        self.push(JsonGrammarItem::Object(Vec::new()), context);
        Ok(())
    }

    /// Semantic action for production 6:
    ///
    /// Pair: String ":" Value;
    ///
    fn pair_6(
        &mut self,
        _string_0: &ParseTreeStackEntry,
        _colon_1: &ParseTreeStackEntry,
        _value_2: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "pair_6";
        match (self.pop(context), self.pop(context)) {
            (Some(value), Some(JsonGrammarItem::String(string))) => {
                self.push(JsonGrammarItem::Pair((string, Box::new(value))), context);
                Ok(())
            }
            _ => Err(miette!("{}: expected Value, Name on top of stack", context)),
        }
    }

    /// Semantic action for production 7:
    ///
    /// Array: "\[" ArraySuffix;
    ///
    fn array_7(
        &mut self,
        _l_bracket_0: &ParseTreeStackEntry,
        _array_suffix_1: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "array_7";
        match self.pop(context) {
            Some(JsonGrammarItem::Array(mut list)) => {
                list.reverse();
                self.push(JsonGrammarItem::Array(list), context);
                Ok(())
            }
            _ => Err(miette!("{}: Expecting Array on top of stack", context)),
        }
    }

    /// Semantic action for production 8:
    ///
    /// ArraySuffix: Value ArrayList "\]";
    ///
    fn array_suffix_8(
        &mut self,
        _value_0: &ParseTreeStackEntry,
        _array_list_1: &ParseTreeStackEntry,
        _r_bracket_2: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "array_suffix_8";
        match (self.pop(context), self.pop(context)) {
            (Some(JsonGrammarItem::Array(mut array)), Some(elem)) => {
                array.push(elem);
                self.push(JsonGrammarItem::Array(array), context);
                Ok(())
            }
            _ => Err(miette!(
                "{}: expecting Array, Value on top of stack",
                context
            )),
        }
    }

    /// Semantic action for production 9:
    ///
    /// ArraySuffix: "\]";
    ///
    fn array_suffix_9(
        &mut self,
        _r_bracket_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "array_suffix_9";
        self.push(JsonGrammarItem::Array(Vec::new()), context);
        Ok(())
    }

    /// Semantic action for production 10:
    ///
    /// ArrayList: "," Value ArrayList;
    ///
    fn array_list_10(
        &mut self,
        _comma_0: &ParseTreeStackEntry,
        _value_1: &ParseTreeStackEntry,
        _array_list_2: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "array_list_10";
        match (self.pop(context), self.pop(context)) {
            (Some(JsonGrammarItem::Array(mut array)), Some(elem)) => {
                array.push(elem);
                self.push(JsonGrammarItem::Array(array), context);
                Ok(())
            }
            _ => Err(miette!(
                "{}: expecting Array, Value on top of stack",
                context
            )),
        }
    }

    /// Semantic action for production 11:
    ///
    /// ArrayList: ;
    ///
    fn array_list_11(&mut self, _parse_tree: &Tree<ParseTreeType>) -> Result<()> {
        let context = "array_list_11";
        self.push(JsonGrammarItem::Array(Vec::new()), context);
        Ok(())
    }

    /// Semantic action for production 16:
    ///
    /// Value: "true";
    ///
    fn value_16(
        &mut self,
        _true_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "value_16";
        self.push(JsonGrammarItem::True, context);
        Ok(())
    }

    /// Semantic action for production 17:
    ///
    /// Value: "false";
    ///
    fn value_17(
        &mut self,
        _false_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "value_17";
        self.push(JsonGrammarItem::False, context);
        Ok(())
    }

    /// Semantic action for production 18:
    ///
    /// Value: "null";
    ///
    fn value_18(
        &mut self,
        _null_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "value_18";
        self.push(JsonGrammarItem::Null, context);
        Ok(())
    }

    /// Semantic action for production 19:
    ///
    /// String: "\u{0022}(\\[\u{0022}\\/bfnrt]|u[0-9a-fA-F]{4}|[^\u{0022}\\\u0000-\u001F])*\u{0022}";
    ///
    fn string_19(
        &mut self,
        string_0: &ParseTreeStackEntry,
        parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "string_19";
        let string = string_0.symbol(parse_tree)?;
        self.push(JsonGrammarItem::String(string.to_string()), context);
        Ok(())
    }

    /// Semantic action for production 20:
    ///
    /// Number: "-?(0|[1-9][0-9]*)(\.[0-9]+)?([eE][-+]?(0|[1-9][0-9]*)?)?";
    ///
    fn number_20(
        &mut self,
        number_0: &ParseTreeStackEntry,
        parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "number_20";
        let number = match number_0.symbol(parse_tree)?.parse::<f64>() {
            Ok(number) => number,
            Err(error) => {
                return Err(miette!(JsonError::ParseF64Failed {
                    input: FileSource::try_new(self.file_name.clone())?.into(),
                    token: number_0.token(parse_tree)?.into()
                }))
                .wrap_err(miette!(error))
            }
        };

        self.push(JsonGrammarItem::Number(number), context);
        Ok(())
    }
}
