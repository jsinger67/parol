// ---------------------------------------------------------
// This file was generated by parol.
// It is not intended for manual editing and changes will be
// lost after next build.
// ---------------------------------------------------------

// Disable clippy warnings that can result in the way how parol generates code.
#![allow(clippy::enum_variant_names)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::upper_case_acronyms)]

use parol_runtime::derive_builder::Builder;
use parol_runtime::log::trace;
#[allow(unused_imports)]
use parol_runtime::parol_macros::{pop_and_reverse_item, pop_item};
use parol_runtime::parser::{ParseTreeType, UserActionsTrait};
use parol_runtime::{ParserError, Result, Token};

/// Semantic actions trait generated for the user grammar
/// All functions have default implementations.
pub trait JsonGrammarTrait<'t> {
    /// Semantic action for non-terminal 'Json'
    fn json(&mut self, _arg: &Json<'t>) -> Result<()> {
        Ok(())
    }

    /// Semantic action for non-terminal 'Object'
    fn object(&mut self, _arg: &Object<'t>) -> Result<()> {
        Ok(())
    }

    /// Semantic action for non-terminal 'Pair'
    fn pair(&mut self, _arg: &Pair<'t>) -> Result<()> {
        Ok(())
    }

    /// Semantic action for non-terminal 'Array'
    fn array(&mut self, _arg: &Array<'t>) -> Result<()> {
        Ok(())
    }

    /// Semantic action for non-terminal 'Value'
    fn value(&mut self, _arg: &Value<'t>) -> Result<()> {
        Ok(())
    }

    /// Semantic action for non-terminal 'String'
    fn string(&mut self, _arg: &String<'t>) -> Result<()> {
        Ok(())
    }

    /// Semantic action for non-terminal 'Number'
    fn number(&mut self, _arg: &Number<'t>) -> Result<()> {
        Ok(())
    }

    /// This method provides skipped language comments.
    /// If you need comments please provide your own implementation of this method.
    fn on_comment_parsed(&mut self, _token: Token<'t>) {}
}

// -------------------------------------------------------------------------------------------------
//
// Output Types of productions deduced from the structure of the transformed grammar
//

///
/// Type derived for production 2
///
/// `ObjectSuffix: Pair ObjectList /* Vec */ '}'^ /* Clipped */;`
///
#[allow(dead_code)]
#[derive(Builder, Debug, Clone)]
#[builder(crate = "parol_runtime::derive_builder")]
pub struct ObjectSuffixPairObjectListRBrace<'t> {
    pub pair: Box<Pair<'t>>,
    pub object_list: Vec<ObjectList<'t>>,
}

///
/// Type derived for production 3
///
/// `ObjectSuffix: '}'^ /* Clipped */;`
///
#[allow(dead_code)]
#[derive(Builder, Debug, Clone)]
#[builder(crate = "parol_runtime::derive_builder")]
pub struct ObjectSuffixRBrace {}

///
/// Type derived for production 8
///
/// `ArraySuffix: Value ArrayList /* Vec */ ']'^ /* Clipped */;`
///
#[allow(dead_code)]
#[derive(Builder, Debug, Clone)]
#[builder(crate = "parol_runtime::derive_builder")]
pub struct ArraySuffixValueArrayListRBracket<'t> {
    pub value: Box<Value<'t>>,
    pub array_list: Vec<ArrayList<'t>>,
}

///
/// Type derived for production 9
///
/// `ArraySuffix: ']'^ /* Clipped */;`
///
#[allow(dead_code)]
#[derive(Builder, Debug, Clone)]
#[builder(crate = "parol_runtime::derive_builder")]
pub struct ArraySuffixRBracket {}

///
/// Type derived for production 12
///
/// `Value: String;`
///
#[allow(dead_code)]
#[derive(Builder, Debug, Clone)]
#[builder(crate = "parol_runtime::derive_builder")]
pub struct ValueString<'t> {
    pub string: Box<String<'t>>,
}

///
/// Type derived for production 13
///
/// `Value: Number;`
///
#[allow(dead_code)]
#[derive(Builder, Debug, Clone)]
#[builder(crate = "parol_runtime::derive_builder")]
pub struct ValueNumber<'t> {
    pub number: Box<Number<'t>>,
}

///
/// Type derived for production 14
///
/// `Value: Object;`
///
#[allow(dead_code)]
#[derive(Builder, Debug, Clone)]
#[builder(crate = "parol_runtime::derive_builder")]
pub struct ValueObject<'t> {
    pub object: Box<Object<'t>>,
}

///
/// Type derived for production 15
///
/// `Value: Array;`
///
#[allow(dead_code)]
#[derive(Builder, Debug, Clone)]
#[builder(crate = "parol_runtime::derive_builder")]
pub struct ValueArray<'t> {
    pub array: Box<Array<'t>>,
}

///
/// Type derived for production 16
///
/// `Value: 'true'^ /* Clipped */;`
///
#[allow(dead_code)]
#[derive(Builder, Debug, Clone)]
#[builder(crate = "parol_runtime::derive_builder")]
pub struct ValueTrue {}

///
/// Type derived for production 17
///
/// `Value: 'false'^ /* Clipped */;`
///
#[allow(dead_code)]
#[derive(Builder, Debug, Clone)]
#[builder(crate = "parol_runtime::derive_builder")]
pub struct ValueFalse {}

///
/// Type derived for production 18
///
/// `Value: 'null'^ /* Clipped */;`
///
#[allow(dead_code)]
#[derive(Builder, Debug, Clone)]
#[builder(crate = "parol_runtime::derive_builder")]
pub struct ValueNull {}

// -------------------------------------------------------------------------------------------------
//
// Types of non-terminals deduced from the structure of the transformed grammar
//

///
/// Type derived for non-terminal Array
///
#[allow(dead_code)]
#[derive(Builder, Debug, Clone)]
#[builder(crate = "parol_runtime::derive_builder")]
pub struct Array<'t> {
    pub array_suffix: Box<ArraySuffix<'t>>,
}

///
/// Type derived for non-terminal ArrayList
///
#[allow(dead_code)]
#[derive(Builder, Debug, Clone)]
#[builder(crate = "parol_runtime::derive_builder")]
pub struct ArrayList<'t> {
    pub value: Box<Value<'t>>,
}

///
/// Type derived for non-terminal ArraySuffix
///
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ArraySuffix<'t> {
    ValueArrayListRBracket(ArraySuffixValueArrayListRBracket<'t>),
    RBracket(ArraySuffixRBracket),
}

///
/// Type derived for non-terminal Json
///
#[allow(dead_code)]
#[derive(Builder, Debug, Clone)]
#[builder(crate = "parol_runtime::derive_builder")]
pub struct Json<'t> {
    pub value: Box<Value<'t>>,
}

///
/// Type derived for non-terminal Number
///
#[allow(dead_code)]
#[derive(Builder, Debug, Clone)]
#[builder(crate = "parol_runtime::derive_builder")]
pub struct Number<'t> {
    pub number: Token<'t>, /* -?(0|[1-9][0-9]*)(\.[0-9]+)?([eE][-+]?(0|[1-9][0-9]*)?)? */
}

///
/// Type derived for non-terminal Object
///
#[allow(dead_code)]
#[derive(Builder, Debug, Clone)]
#[builder(crate = "parol_runtime::derive_builder")]
pub struct Object<'t> {
    pub object_suffix: Box<ObjectSuffix<'t>>,
}

///
/// Type derived for non-terminal ObjectList
///
#[allow(dead_code)]
#[derive(Builder, Debug, Clone)]
#[builder(crate = "parol_runtime::derive_builder")]
pub struct ObjectList<'t> {
    pub pair: Box<Pair<'t>>,
}

///
/// Type derived for non-terminal ObjectSuffix
///
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ObjectSuffix<'t> {
    PairObjectListRBrace(ObjectSuffixPairObjectListRBrace<'t>),
    RBrace(ObjectSuffixRBrace),
}

///
/// Type derived for non-terminal Pair
///
#[allow(dead_code)]
#[derive(Builder, Debug, Clone)]
#[builder(crate = "parol_runtime::derive_builder")]
pub struct Pair<'t> {
    pub string: Box<String<'t>>,
    pub value: Box<Value<'t>>,
}

///
/// Type derived for non-terminal String
///
#[allow(dead_code)]
#[derive(Builder, Debug, Clone)]
#[builder(crate = "parol_runtime::derive_builder")]
pub struct String<'t> {
    pub string: Token<'t>, /* "(\\.|[^"])*" */
}

///
/// Type derived for non-terminal Value
///
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Value<'t> {
    String(ValueString<'t>),
    Number(ValueNumber<'t>),
    Object(ValueObject<'t>),
    Array(ValueArray<'t>),
    True(ValueTrue),
    False(ValueFalse),
    Null(ValueNull),
}

// -------------------------------------------------------------------------------------------------

///
/// Deduced ASTType of expanded grammar
///
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ASTType<'t> {
    Array(Array<'t>),
    ArrayList(Vec<ArrayList<'t>>),
    ArraySuffix(ArraySuffix<'t>),
    Json(Json<'t>),
    Number(Number<'t>),
    Object(Object<'t>),
    ObjectList(Vec<ObjectList<'t>>),
    ObjectSuffix(ObjectSuffix<'t>),
    Pair(Pair<'t>),
    String(String<'t>),
    Value(Value<'t>),
}

/// Auto-implemented adapter grammar
///
/// The lifetime parameter `'t` refers to the lifetime of the scanned text.
/// The lifetime parameter `'u` refers to the lifetime of user grammar object.
///
#[allow(dead_code)]
pub struct JsonGrammarAuto<'t, 'u>
where
    't: 'u,
{
    // Mutable reference of the actual user grammar to be able to call the semantic actions on it
    user_grammar: &'u mut dyn JsonGrammarTrait<'t>,
    // Stack to construct the AST on it
    item_stack: Vec<ASTType<'t>>,
}

///
/// The `JsonGrammarAuto` impl is automatically generated for the
/// given grammar.
///
impl<'t, 'u> JsonGrammarAuto<'t, 'u> {
    pub fn new(user_grammar: &'u mut dyn JsonGrammarTrait<'t>) -> Self {
        Self {
            user_grammar,
            item_stack: Vec::new(),
        }
    }

    #[allow(dead_code)]
    fn push(&mut self, item: ASTType<'t>, context: &str) {
        trace!("push    {}: {:?}", context, item);
        self.item_stack.push(item)
    }

    #[allow(dead_code)]
    fn pop(&mut self, context: &str) -> Option<ASTType<'t>> {
        let item = self.item_stack.pop();
        if let Some(ref item) = item {
            trace!("pop     {}: {:?}", context, item);
        }
        item
    }

    #[allow(dead_code)]
    // Use this function for debugging purposes:
    // trace!("{}", self.trace_item_stack(context));
    fn trace_item_stack(&self, context: &str) -> std::string::String {
        format!(
            "Item stack at {}:\n{}",
            context,
            self.item_stack
                .iter()
                .rev()
                .map(|s| format!("  {:?}", s))
                .collect::<Vec<std::string::String>>()
                .join("\n")
        )
    }

    /// Semantic action for production 0:
    ///
    /// `Json: Value;`
    ///
    #[parol_runtime::function_name::named]
    fn json(&mut self, _value: &ParseTreeType<'t>) -> Result<()> {
        let context = function_name!();
        trace!("{}", self.trace_item_stack(context));
        let value = pop_item!(self, value, Value, context);
        let json_built = Json {
            value: Box::new(value),
        };
        // Calling user action here
        self.user_grammar.json(&json_built)?;
        self.push(ASTType::Json(json_built), context);
        Ok(())
    }

    /// Semantic action for production 1:
    ///
    /// `Object: '{'^ /* Clipped */ ObjectSuffix;`
    ///
    #[parol_runtime::function_name::named]
    fn object(
        &mut self,
        _l_brace: &ParseTreeType<'t>,
        _object_suffix: &ParseTreeType<'t>,
    ) -> Result<()> {
        let context = function_name!();
        trace!("{}", self.trace_item_stack(context));
        let object_suffix = pop_item!(self, object_suffix, ObjectSuffix, context);
        let object_built = Object {
            object_suffix: Box::new(object_suffix),
        };
        // Calling user action here
        self.user_grammar.object(&object_built)?;
        self.push(ASTType::Object(object_built), context);
        Ok(())
    }

    /// Semantic action for production 2:
    ///
    /// `ObjectSuffix: Pair ObjectList /* Vec */ '}'^ /* Clipped */;`
    ///
    #[parol_runtime::function_name::named]
    fn object_suffix_0(
        &mut self,
        _pair: &ParseTreeType<'t>,
        _object_list: &ParseTreeType<'t>,
        _r_brace: &ParseTreeType<'t>,
    ) -> Result<()> {
        let context = function_name!();
        trace!("{}", self.trace_item_stack(context));
        let object_list = pop_and_reverse_item!(self, object_list, ObjectList, context);
        let pair = pop_item!(self, pair, Pair, context);
        let object_suffix_0_built = ObjectSuffixPairObjectListRBrace {
            pair: Box::new(pair),
            object_list,
        };
        let object_suffix_0_built = ObjectSuffix::PairObjectListRBrace(object_suffix_0_built);
        self.push(ASTType::ObjectSuffix(object_suffix_0_built), context);
        Ok(())
    }

    /// Semantic action for production 3:
    ///
    /// `ObjectSuffix: '}'^ /* Clipped */;`
    ///
    #[parol_runtime::function_name::named]
    fn object_suffix_1(&mut self, _r_brace: &ParseTreeType<'t>) -> Result<()> {
        let context = function_name!();
        trace!("{}", self.trace_item_stack(context));
        let object_suffix_1_built = ObjectSuffixRBrace {};
        let object_suffix_1_built = ObjectSuffix::RBrace(object_suffix_1_built);
        self.push(ASTType::ObjectSuffix(object_suffix_1_built), context);
        Ok(())
    }

    /// Semantic action for production 4:
    ///
    /// `ObjectList /* Vec<T>::Push */: ','^ /* Clipped */ Pair ObjectList;`
    ///
    #[parol_runtime::function_name::named]
    fn object_list_0(
        &mut self,
        _comma: &ParseTreeType<'t>,
        _pair: &ParseTreeType<'t>,
        _object_list: &ParseTreeType<'t>,
    ) -> Result<()> {
        let context = function_name!();
        trace!("{}", self.trace_item_stack(context));
        let mut object_list = pop_item!(self, object_list, ObjectList, context);
        let pair = pop_item!(self, pair, Pair, context);
        let object_list_0_built = ObjectList {
            pair: Box::new(pair),
        };
        // Add an element to the vector
        object_list.push(object_list_0_built);
        self.push(ASTType::ObjectList(object_list), context);
        Ok(())
    }

    /// Semantic action for production 5:
    ///
    /// `ObjectList /* Vec<T>::New */: ;`
    ///
    #[parol_runtime::function_name::named]
    fn object_list_1(&mut self) -> Result<()> {
        let context = function_name!();
        trace!("{}", self.trace_item_stack(context));
        let object_list_1_built = Vec::new();
        self.push(ASTType::ObjectList(object_list_1_built), context);
        Ok(())
    }

    /// Semantic action for production 6:
    ///
    /// `Pair: String ':'^ /* Clipped */ Value;`
    ///
    #[parol_runtime::function_name::named]
    fn pair(
        &mut self,
        _string: &ParseTreeType<'t>,
        _colon: &ParseTreeType<'t>,
        _value: &ParseTreeType<'t>,
    ) -> Result<()> {
        let context = function_name!();
        trace!("{}", self.trace_item_stack(context));
        let value = pop_item!(self, value, Value, context);
        let string = pop_item!(self, string, String, context);
        let pair_built = Pair {
            string: Box::new(string),
            value: Box::new(value),
        };
        // Calling user action here
        self.user_grammar.pair(&pair_built)?;
        self.push(ASTType::Pair(pair_built), context);
        Ok(())
    }

    /// Semantic action for production 7:
    ///
    /// `Array: '['^ /* Clipped */ ArraySuffix;`
    ///
    #[parol_runtime::function_name::named]
    fn array(
        &mut self,
        _l_bracket: &ParseTreeType<'t>,
        _array_suffix: &ParseTreeType<'t>,
    ) -> Result<()> {
        let context = function_name!();
        trace!("{}", self.trace_item_stack(context));
        let array_suffix = pop_item!(self, array_suffix, ArraySuffix, context);
        let array_built = Array {
            array_suffix: Box::new(array_suffix),
        };
        // Calling user action here
        self.user_grammar.array(&array_built)?;
        self.push(ASTType::Array(array_built), context);
        Ok(())
    }

    /// Semantic action for production 8:
    ///
    /// `ArraySuffix: Value ArrayList /* Vec */ ']'^ /* Clipped */;`
    ///
    #[parol_runtime::function_name::named]
    fn array_suffix_0(
        &mut self,
        _value: &ParseTreeType<'t>,
        _array_list: &ParseTreeType<'t>,
        _r_bracket: &ParseTreeType<'t>,
    ) -> Result<()> {
        let context = function_name!();
        trace!("{}", self.trace_item_stack(context));
        let array_list = pop_and_reverse_item!(self, array_list, ArrayList, context);
        let value = pop_item!(self, value, Value, context);
        let array_suffix_0_built = ArraySuffixValueArrayListRBracket {
            value: Box::new(value),
            array_list,
        };
        let array_suffix_0_built = ArraySuffix::ValueArrayListRBracket(array_suffix_0_built);
        self.push(ASTType::ArraySuffix(array_suffix_0_built), context);
        Ok(())
    }

    /// Semantic action for production 9:
    ///
    /// `ArraySuffix: ']'^ /* Clipped */;`
    ///
    #[parol_runtime::function_name::named]
    fn array_suffix_1(&mut self, _r_bracket: &ParseTreeType<'t>) -> Result<()> {
        let context = function_name!();
        trace!("{}", self.trace_item_stack(context));
        let array_suffix_1_built = ArraySuffixRBracket {};
        let array_suffix_1_built = ArraySuffix::RBracket(array_suffix_1_built);
        self.push(ASTType::ArraySuffix(array_suffix_1_built), context);
        Ok(())
    }

    /// Semantic action for production 10:
    ///
    /// `ArrayList /* Vec<T>::Push */: ','^ /* Clipped */ Value ArrayList;`
    ///
    #[parol_runtime::function_name::named]
    fn array_list_0(
        &mut self,
        _comma: &ParseTreeType<'t>,
        _value: &ParseTreeType<'t>,
        _array_list: &ParseTreeType<'t>,
    ) -> Result<()> {
        let context = function_name!();
        trace!("{}", self.trace_item_stack(context));
        let mut array_list = pop_item!(self, array_list, ArrayList, context);
        let value = pop_item!(self, value, Value, context);
        let array_list_0_built = ArrayList {
            value: Box::new(value),
        };
        // Add an element to the vector
        array_list.push(array_list_0_built);
        self.push(ASTType::ArrayList(array_list), context);
        Ok(())
    }

    /// Semantic action for production 11:
    ///
    /// `ArrayList /* Vec<T>::New */: ;`
    ///
    #[parol_runtime::function_name::named]
    fn array_list_1(&mut self) -> Result<()> {
        let context = function_name!();
        trace!("{}", self.trace_item_stack(context));
        let array_list_1_built = Vec::new();
        self.push(ASTType::ArrayList(array_list_1_built), context);
        Ok(())
    }

    /// Semantic action for production 12:
    ///
    /// `Value: String;`
    ///
    #[parol_runtime::function_name::named]
    fn value_0(&mut self, _string: &ParseTreeType<'t>) -> Result<()> {
        let context = function_name!();
        trace!("{}", self.trace_item_stack(context));
        let string = pop_item!(self, string, String, context);
        let value_0_built = ValueString {
            string: Box::new(string),
        };
        let value_0_built = Value::String(value_0_built);
        // Calling user action here
        self.user_grammar.value(&value_0_built)?;
        self.push(ASTType::Value(value_0_built), context);
        Ok(())
    }

    /// Semantic action for production 13:
    ///
    /// `Value: Number;`
    ///
    #[parol_runtime::function_name::named]
    fn value_1(&mut self, _number: &ParseTreeType<'t>) -> Result<()> {
        let context = function_name!();
        trace!("{}", self.trace_item_stack(context));
        let number = pop_item!(self, number, Number, context);
        let value_1_built = ValueNumber {
            number: Box::new(number),
        };
        let value_1_built = Value::Number(value_1_built);
        // Calling user action here
        self.user_grammar.value(&value_1_built)?;
        self.push(ASTType::Value(value_1_built), context);
        Ok(())
    }

    /// Semantic action for production 14:
    ///
    /// `Value: Object;`
    ///
    #[parol_runtime::function_name::named]
    fn value_2(&mut self, _object: &ParseTreeType<'t>) -> Result<()> {
        let context = function_name!();
        trace!("{}", self.trace_item_stack(context));
        let object = pop_item!(self, object, Object, context);
        let value_2_built = ValueObject {
            object: Box::new(object),
        };
        let value_2_built = Value::Object(value_2_built);
        // Calling user action here
        self.user_grammar.value(&value_2_built)?;
        self.push(ASTType::Value(value_2_built), context);
        Ok(())
    }

    /// Semantic action for production 15:
    ///
    /// `Value: Array;`
    ///
    #[parol_runtime::function_name::named]
    fn value_3(&mut self, _array: &ParseTreeType<'t>) -> Result<()> {
        let context = function_name!();
        trace!("{}", self.trace_item_stack(context));
        let array = pop_item!(self, array, Array, context);
        let value_3_built = ValueArray {
            array: Box::new(array),
        };
        let value_3_built = Value::Array(value_3_built);
        // Calling user action here
        self.user_grammar.value(&value_3_built)?;
        self.push(ASTType::Value(value_3_built), context);
        Ok(())
    }

    /// Semantic action for production 16:
    ///
    /// `Value: 'true'^ /* Clipped */;`
    ///
    #[parol_runtime::function_name::named]
    fn value_4(&mut self, _true: &ParseTreeType<'t>) -> Result<()> {
        let context = function_name!();
        trace!("{}", self.trace_item_stack(context));
        let value_4_built = ValueTrue {};
        let value_4_built = Value::True(value_4_built);
        // Calling user action here
        self.user_grammar.value(&value_4_built)?;
        self.push(ASTType::Value(value_4_built), context);
        Ok(())
    }

    /// Semantic action for production 17:
    ///
    /// `Value: 'false'^ /* Clipped */;`
    ///
    #[parol_runtime::function_name::named]
    fn value_5(&mut self, _false: &ParseTreeType<'t>) -> Result<()> {
        let context = function_name!();
        trace!("{}", self.trace_item_stack(context));
        let value_5_built = ValueFalse {};
        let value_5_built = Value::False(value_5_built);
        // Calling user action here
        self.user_grammar.value(&value_5_built)?;
        self.push(ASTType::Value(value_5_built), context);
        Ok(())
    }

    /// Semantic action for production 18:
    ///
    /// `Value: 'null'^ /* Clipped */;`
    ///
    #[parol_runtime::function_name::named]
    fn value_6(&mut self, _null: &ParseTreeType<'t>) -> Result<()> {
        let context = function_name!();
        trace!("{}", self.trace_item_stack(context));
        let value_6_built = ValueNull {};
        let value_6_built = Value::Null(value_6_built);
        // Calling user action here
        self.user_grammar.value(&value_6_built)?;
        self.push(ASTType::Value(value_6_built), context);
        Ok(())
    }

    /// Semantic action for production 19:
    ///
    /// `String: /"(\\.|[^"])*"/;`
    ///
    #[parol_runtime::function_name::named]
    fn string(&mut self, string: &ParseTreeType<'t>) -> Result<()> {
        let context = function_name!();
        trace!("{}", self.trace_item_stack(context));
        let string = string.token()?.clone();
        let string_built = String { string };
        // Calling user action here
        self.user_grammar.string(&string_built)?;
        self.push(ASTType::String(string_built), context);
        Ok(())
    }

    /// Semantic action for production 20:
    ///
    /// `Number: /-?(0|[1-9][0-9]*)(\.[0-9]+)?([eE][-+]?(0|[1-9][0-9]*)?)?/;`
    ///
    #[parol_runtime::function_name::named]
    fn number(&mut self, number: &ParseTreeType<'t>) -> Result<()> {
        let context = function_name!();
        trace!("{}", self.trace_item_stack(context));
        let number = number.token()?.clone();
        let number_built = Number { number };
        // Calling user action here
        self.user_grammar.number(&number_built)?;
        self.push(ASTType::Number(number_built), context);
        Ok(())
    }
}

impl<'t> UserActionsTrait<'t> for JsonGrammarAuto<'t, '_> {
    ///
    /// This function is implemented automatically for the user's item JsonGrammar.
    ///
    fn call_semantic_action_for_production_number(
        &mut self,
        prod_num: usize,
        children: &[ParseTreeType<'t>],
    ) -> Result<()> {
        match prod_num {
            0 => self.json(&children[0]),
            1 => self.object(&children[0], &children[1]),
            2 => self.object_suffix_0(&children[0], &children[1], &children[2]),
            3 => self.object_suffix_1(&children[0]),
            4 => self.object_list_0(&children[0], &children[1], &children[2]),
            5 => self.object_list_1(),
            6 => self.pair(&children[0], &children[1], &children[2]),
            7 => self.array(&children[0], &children[1]),
            8 => self.array_suffix_0(&children[0], &children[1], &children[2]),
            9 => self.array_suffix_1(&children[0]),
            10 => self.array_list_0(&children[0], &children[1], &children[2]),
            11 => self.array_list_1(),
            12 => self.value_0(&children[0]),
            13 => self.value_1(&children[0]),
            14 => self.value_2(&children[0]),
            15 => self.value_3(&children[0]),
            16 => self.value_4(&children[0]),
            17 => self.value_5(&children[0]),
            18 => self.value_6(&children[0]),
            19 => self.string(&children[0]),
            20 => self.number(&children[0]),
            _ => Err(ParserError::InternalError(format!(
                "Unhandled production number: {}",
                prod_num
            ))
            .into()),
        }
    }

    fn on_comment_parsed(&mut self, token: Token<'t>) {
        self.user_grammar.on_comment_parsed(token)
    }
}
