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
use parol_runtime::parser::parse_tree_type::{NonTerminalEnum, TerminalEnum};
use parol_runtime::parser::{ParseTreeType, UserActionsTrait};
use parol_runtime::{ParserError, Result, Token};

/// Semantic actions trait generated for the user grammar
/// All functions have default implementations.
pub trait ScannerStatesGrammarTrait<'t> {
    /// Semantic action for non-terminal 'Start'
    fn start(&mut self, _arg: &Start<'t>) -> Result<()> {
        Ok(())
    }

    /// Semantic action for non-terminal 'Content'
    fn content(&mut self, _arg: &Content<'t>) -> Result<()> {
        Ok(())
    }

    /// Semantic action for non-terminal 'StringContent'
    fn string_content(&mut self, _arg: &StringContent<'t>) -> Result<()> {
        Ok(())
    }

    /// Semantic action for non-terminal 'StringElement'
    fn string_element(&mut self, _arg: &StringElement<'t>) -> Result<()> {
        Ok(())
    }

    /// Semantic action for non-terminal 'Identifier'
    fn identifier(&mut self, _arg: &Identifier<'t>) -> Result<()> {
        Ok(())
    }

    /// Semantic action for non-terminal 'Escaped'
    fn escaped(&mut self, _arg: &Escaped<'t>) -> Result<()> {
        Ok(())
    }

    /// Semantic action for non-terminal 'EscapedLineEnd'
    fn escaped_line_end(&mut self, _arg: &EscapedLineEnd<'t>) -> Result<()> {
        Ok(())
    }

    /// Semantic action for non-terminal 'NoneQuote'
    fn none_quote(&mut self, _arg: &NoneQuote<'t>) -> Result<()> {
        Ok(())
    }

    /// Semantic action for non-terminal 'StringDelimiter'
    fn string_delimiter(&mut self, _arg: &StringDelimiter<'t>) -> Result<()> {
        Ok(())
    }

    /// This method provides skipped language comments.
    /// If you need comments please provide your own implementation of this method.
    fn on_comment(&mut self, _token: Token<'t>) {}
}

// -------------------------------------------------------------------------------------------------
//
// Output Types of productions deduced from the structure of the transformed grammar
//

///
/// Type derived for production 3
///
/// `Content: Identifier;`
///
#[allow(dead_code)]
#[derive(Builder, Debug, Clone)]
#[builder(crate = "parol_runtime::derive_builder")]
pub struct ContentIdentifier<'t> {
    pub identifier: Box<Identifier<'t>>,
}

///
/// Type derived for production 4
///
/// `Content: StringDelimiter %push(String) StringContent StringDelimiter %pop();`
///
#[allow(dead_code)]
#[derive(Builder, Debug, Clone)]
#[builder(crate = "parol_runtime::derive_builder")]
pub struct ContentStringDelimiterStringContentStringDelimiter<'t> {
    pub string_delimiter: Box<StringDelimiter<'t>>,
    pub string_content: Box<StringContent<'t>>,
    pub string_delimiter0: Box<StringDelimiter<'t>>,
}

///
/// Type derived for production 8
///
/// `StringElement: Escaped;`
///
#[allow(dead_code)]
#[derive(Builder, Debug, Clone)]
#[builder(crate = "parol_runtime::derive_builder")]
pub struct StringElementEscaped<'t> {
    pub escaped: Box<Escaped<'t>>,
}

///
/// Type derived for production 9
///
/// `StringElement: EscapedLineEnd;`
///
#[allow(dead_code)]
#[derive(Builder, Debug, Clone)]
#[builder(crate = "parol_runtime::derive_builder")]
pub struct StringElementEscapedLineEnd<'t> {
    pub escaped_line_end: Box<EscapedLineEnd<'t>>,
}

///
/// Type derived for production 10
///
/// `StringElement: NoneQuote;`
///
#[allow(dead_code)]
#[derive(Builder, Debug, Clone)]
#[builder(crate = "parol_runtime::derive_builder")]
pub struct StringElementNoneQuote<'t> {
    pub none_quote: Box<NoneQuote<'t>>,
}

// -------------------------------------------------------------------------------------------------
//
// Types of non-terminals deduced from the structure of the transformed grammar
//

///
/// Type derived for non-terminal Content
///
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Content<'t> {
    Identifier(ContentIdentifier<'t>),
    StringDelimiterStringContentStringDelimiter(
        ContentStringDelimiterStringContentStringDelimiter<'t>,
    ),
}

///
/// Type derived for non-terminal Escaped
///
#[allow(dead_code)]
#[derive(Builder, Debug, Clone)]
#[builder(crate = "parol_runtime::derive_builder")]
pub struct Escaped<'t> {
    pub escaped: Token<'t>, /* \\["\\bfnt] */
}

///
/// Type derived for non-terminal EscapedLineEnd
///
#[allow(dead_code)]
#[derive(Builder, Debug, Clone)]
#[builder(crate = "parol_runtime::derive_builder")]
pub struct EscapedLineEnd<'t> {
    pub escaped_line_end: Token<'t>, /* \\[\s--\n\r]*\r?\n */
}

///
/// Type derived for non-terminal Identifier
///
#[allow(dead_code)]
#[derive(Builder, Debug, Clone)]
#[builder(crate = "parol_runtime::derive_builder")]
pub struct Identifier<'t> {
    pub identifier: Token<'t>, /* [a-zA-Z_]\w* */
}

///
/// Type derived for non-terminal NoneQuote
///
#[allow(dead_code)]
#[derive(Builder, Debug, Clone)]
#[builder(crate = "parol_runtime::derive_builder")]
pub struct NoneQuote<'t> {
    pub none_quote: Token<'t>, /* [^"\\]+ */
}

///
/// Type derived for non-terminal Start
///
#[allow(dead_code)]
#[derive(Builder, Debug, Clone)]
#[builder(crate = "parol_runtime::derive_builder")]
pub struct Start<'t> {
    pub start_list: Vec<StartList<'t>>,
}

///
/// Type derived for non-terminal StartList
///
#[allow(dead_code)]
#[derive(Builder, Debug, Clone)]
#[builder(crate = "parol_runtime::derive_builder")]
pub struct StartList<'t> {
    pub content: Box<Content<'t>>,
}

///
/// Type derived for non-terminal StringContent
///
#[allow(dead_code)]
#[derive(Builder, Debug, Clone)]
#[builder(crate = "parol_runtime::derive_builder")]
pub struct StringContent<'t> {
    pub string_content_list: Vec<StringContentList<'t>>,
}

///
/// Type derived for non-terminal StringContentList
///
#[allow(dead_code)]
#[derive(Builder, Debug, Clone)]
#[builder(crate = "parol_runtime::derive_builder")]
pub struct StringContentList<'t> {
    pub string_element: Box<StringElement<'t>>,
}

///
/// Type derived for non-terminal StringDelimiter
///
#[allow(dead_code)]
#[derive(Builder, Debug, Clone)]
#[builder(crate = "parol_runtime::derive_builder")]
pub struct StringDelimiter<'t> {
    pub string_delimiter: Token<'t>, /* " */
}

///
/// Type derived for non-terminal StringElement
///
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum StringElement<'t> {
    Escaped(StringElementEscaped<'t>),
    EscapedLineEnd(StringElementEscapedLineEnd<'t>),
    NoneQuote(StringElementNoneQuote<'t>),
}

// -------------------------------------------------------------------------------------------------

///
/// Deduced ASTType of expanded grammar
///
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ASTType<'t> {
    Content(Content<'t>),
    Escaped(Escaped<'t>),
    EscapedLineEnd(EscapedLineEnd<'t>),
    Identifier(Identifier<'t>),
    NoneQuote(NoneQuote<'t>),
    Start(Start<'t>),
    StartList(Vec<StartList<'t>>),
    StringContent(StringContent<'t>),
    StringContentList(Vec<StringContentList<'t>>),
    StringDelimiter(StringDelimiter<'t>),
    StringElement(StringElement<'t>),
}

// -------------------------------------------------------------------------------------------------

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NonTerminalKind {
    Content,
    Escaped,
    EscapedLineEnd,
    Identifier,
    NoneQuote,
    Start,
    StartList,
    StringContent,
    StringContentList,
    StringDelimiter,
    StringElement,
    Root,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TerminalKind {
    NewLine,
    Whitespace,
    LineComment,
    BlockComment,
    Identifier,
    Escaped,
    EscapedLineEnd,
    NoneQuote,
    StringDelimiter,
}

impl TerminalEnum for TerminalKind {
    fn from_terminal_index(index: u16) -> Self {
        match index {
            1 => Self::NewLine,
            2 => Self::Whitespace,
            3 => Self::LineComment,
            4 => Self::BlockComment,
            5 => Self::Identifier,
            6 => Self::Escaped,
            7 => Self::EscapedLineEnd,
            8 => Self::NoneQuote,
            9 => Self::StringDelimiter,
            _ => panic!("Invalid terminal index: {}", index),
        }
    }
}

impl NonTerminalEnum for NonTerminalKind {
    fn from_non_terminal_name(name: &str) -> Self {
        match name {
            "Content" => Self::Content,
            "Escaped" => Self::Escaped,
            "EscapedLineEnd" => Self::EscapedLineEnd,
            "Identifier" => Self::Identifier,
            "NoneQuote" => Self::NoneQuote,
            "Start" => Self::Start,
            "StartList" => Self::StartList,
            "StringContent" => Self::StringContent,
            "StringContentList" => Self::StringContentList,
            "StringDelimiter" => Self::StringDelimiter,
            "StringElement" => Self::StringElement,
            "" => Self::Root,
            _ => panic!("Invalid non-terminal name: {}", name),
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Auto-implemented adapter grammar
///
/// The lifetime parameter `'t` refers to the lifetime of the scanned text.
/// The lifetime parameter `'u` refers to the lifetime of user grammar object.
///
#[allow(dead_code)]
pub struct ScannerStatesGrammarAuto<'t, 'u>
where
    't: 'u,
{
    // Mutable reference of the actual user grammar to be able to call the semantic actions on it
    user_grammar: &'u mut dyn ScannerStatesGrammarTrait<'t>,
    // Stack to construct the AST on it
    item_stack: Vec<ASTType<'t>>,
}

///
/// The `ScannerStatesGrammarAuto` impl is automatically generated for the
/// given grammar.
///
impl<'t, 'u> ScannerStatesGrammarAuto<'t, 'u> {
    pub fn new(user_grammar: &'u mut dyn ScannerStatesGrammarTrait<'t>) -> Self {
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
    /// `Start: StartList /* Vec */;`
    ///
    #[parol_runtime::function_name::named]
    fn start(&mut self, _start_list: &ParseTreeType<'t>) -> Result<()> {
        let context = function_name!();
        trace!("{}", self.trace_item_stack(context));
        let start_list = pop_and_reverse_item!(self, start_list, StartList, context);
        let start_built = Start { start_list };
        // Calling user action here
        self.user_grammar.start(&start_built)?;
        self.push(ASTType::Start(start_built), context);
        Ok(())
    }

    /// Semantic action for production 1:
    ///
    /// `StartList /* Vec<T>::Push */: Content StartList;`
    ///
    #[parol_runtime::function_name::named]
    fn start_list_0(
        &mut self,
        _content: &ParseTreeType<'t>,
        _start_list: &ParseTreeType<'t>,
    ) -> Result<()> {
        let context = function_name!();
        trace!("{}", self.trace_item_stack(context));
        let mut start_list = pop_item!(self, start_list, StartList, context);
        let content = pop_item!(self, content, Content, context);
        let start_list_0_built = StartList {
            content: Box::new(content),
        };
        // Add an element to the vector
        start_list.push(start_list_0_built);
        self.push(ASTType::StartList(start_list), context);
        Ok(())
    }

    /// Semantic action for production 2:
    ///
    /// `StartList /* Vec<T>::New */: ;`
    ///
    #[parol_runtime::function_name::named]
    fn start_list_1(&mut self) -> Result<()> {
        let context = function_name!();
        trace!("{}", self.trace_item_stack(context));
        let start_list_1_built = Vec::new();
        self.push(ASTType::StartList(start_list_1_built), context);
        Ok(())
    }

    /// Semantic action for production 3:
    ///
    /// `Content: Identifier;`
    ///
    #[parol_runtime::function_name::named]
    fn content_0(&mut self, _identifier: &ParseTreeType<'t>) -> Result<()> {
        let context = function_name!();
        trace!("{}", self.trace_item_stack(context));
        let identifier = pop_item!(self, identifier, Identifier, context);
        let content_0_built = ContentIdentifier {
            identifier: Box::new(identifier),
        };
        let content_0_built = Content::Identifier(content_0_built);
        // Calling user action here
        self.user_grammar.content(&content_0_built)?;
        self.push(ASTType::Content(content_0_built), context);
        Ok(())
    }

    /// Semantic action for production 4:
    ///
    /// `Content: StringDelimiter %push(String) StringContent StringDelimiter %pop();`
    ///
    #[parol_runtime::function_name::named]
    fn content_1(
        &mut self,
        _string_delimiter: &ParseTreeType<'t>,
        _string_content: &ParseTreeType<'t>,
        _string_delimiter0: &ParseTreeType<'t>,
    ) -> Result<()> {
        let context = function_name!();
        trace!("{}", self.trace_item_stack(context));
        let string_delimiter0 = pop_item!(self, string_delimiter0, StringDelimiter, context);
        let string_content = pop_item!(self, string_content, StringContent, context);
        let string_delimiter = pop_item!(self, string_delimiter, StringDelimiter, context);
        let content_1_built = ContentStringDelimiterStringContentStringDelimiter {
            string_delimiter: Box::new(string_delimiter),
            string_content: Box::new(string_content),
            string_delimiter0: Box::new(string_delimiter0),
        };
        let content_1_built = Content::StringDelimiterStringContentStringDelimiter(content_1_built);
        // Calling user action here
        self.user_grammar.content(&content_1_built)?;
        self.push(ASTType::Content(content_1_built), context);
        Ok(())
    }

    /// Semantic action for production 5:
    ///
    /// `StringContent: StringContentList /* Vec */;`
    ///
    #[parol_runtime::function_name::named]
    fn string_content(&mut self, _string_content_list: &ParseTreeType<'t>) -> Result<()> {
        let context = function_name!();
        trace!("{}", self.trace_item_stack(context));
        let string_content_list =
            pop_and_reverse_item!(self, string_content_list, StringContentList, context);
        let string_content_built = StringContent {
            string_content_list,
        };
        // Calling user action here
        self.user_grammar.string_content(&string_content_built)?;
        self.push(ASTType::StringContent(string_content_built), context);
        Ok(())
    }

    /// Semantic action for production 6:
    ///
    /// `StringContentList /* Vec<T>::Push */: StringElement StringContentList;`
    ///
    #[parol_runtime::function_name::named]
    fn string_content_list_0(
        &mut self,
        _string_element: &ParseTreeType<'t>,
        _string_content_list: &ParseTreeType<'t>,
    ) -> Result<()> {
        let context = function_name!();
        trace!("{}", self.trace_item_stack(context));
        let mut string_content_list =
            pop_item!(self, string_content_list, StringContentList, context);
        let string_element = pop_item!(self, string_element, StringElement, context);
        let string_content_list_0_built = StringContentList {
            string_element: Box::new(string_element),
        };
        // Add an element to the vector
        string_content_list.push(string_content_list_0_built);
        self.push(ASTType::StringContentList(string_content_list), context);
        Ok(())
    }

    /// Semantic action for production 7:
    ///
    /// `StringContentList /* Vec<T>::New */: ;`
    ///
    #[parol_runtime::function_name::named]
    fn string_content_list_1(&mut self) -> Result<()> {
        let context = function_name!();
        trace!("{}", self.trace_item_stack(context));
        let string_content_list_1_built = Vec::new();
        self.push(
            ASTType::StringContentList(string_content_list_1_built),
            context,
        );
        Ok(())
    }

    /// Semantic action for production 8:
    ///
    /// `StringElement: Escaped;`
    ///
    #[parol_runtime::function_name::named]
    fn string_element_0(&mut self, _escaped: &ParseTreeType<'t>) -> Result<()> {
        let context = function_name!();
        trace!("{}", self.trace_item_stack(context));
        let escaped = pop_item!(self, escaped, Escaped, context);
        let string_element_0_built = StringElementEscaped {
            escaped: Box::new(escaped),
        };
        let string_element_0_built = StringElement::Escaped(string_element_0_built);
        // Calling user action here
        self.user_grammar.string_element(&string_element_0_built)?;
        self.push(ASTType::StringElement(string_element_0_built), context);
        Ok(())
    }

    /// Semantic action for production 9:
    ///
    /// `StringElement: EscapedLineEnd;`
    ///
    #[parol_runtime::function_name::named]
    fn string_element_1(&mut self, _escaped_line_end: &ParseTreeType<'t>) -> Result<()> {
        let context = function_name!();
        trace!("{}", self.trace_item_stack(context));
        let escaped_line_end = pop_item!(self, escaped_line_end, EscapedLineEnd, context);
        let string_element_1_built = StringElementEscapedLineEnd {
            escaped_line_end: Box::new(escaped_line_end),
        };
        let string_element_1_built = StringElement::EscapedLineEnd(string_element_1_built);
        // Calling user action here
        self.user_grammar.string_element(&string_element_1_built)?;
        self.push(ASTType::StringElement(string_element_1_built), context);
        Ok(())
    }

    /// Semantic action for production 10:
    ///
    /// `StringElement: NoneQuote;`
    ///
    #[parol_runtime::function_name::named]
    fn string_element_2(&mut self, _none_quote: &ParseTreeType<'t>) -> Result<()> {
        let context = function_name!();
        trace!("{}", self.trace_item_stack(context));
        let none_quote = pop_item!(self, none_quote, NoneQuote, context);
        let string_element_2_built = StringElementNoneQuote {
            none_quote: Box::new(none_quote),
        };
        let string_element_2_built = StringElement::NoneQuote(string_element_2_built);
        // Calling user action here
        self.user_grammar.string_element(&string_element_2_built)?;
        self.push(ASTType::StringElement(string_element_2_built), context);
        Ok(())
    }

    /// Semantic action for production 11:
    ///
    /// `Identifier: /[a-zA-Z_]\w*/;`
    ///
    #[parol_runtime::function_name::named]
    fn identifier(&mut self, identifier: &ParseTreeType<'t>) -> Result<()> {
        let context = function_name!();
        trace!("{}", self.trace_item_stack(context));
        let identifier = identifier.token()?.clone();
        let identifier_built = Identifier { identifier };
        // Calling user action here
        self.user_grammar.identifier(&identifier_built)?;
        self.push(ASTType::Identifier(identifier_built), context);
        Ok(())
    }

    /// Semantic action for production 12:
    ///
    /// `Escaped: <String>/\\["\\bfnt]/;`
    ///
    #[parol_runtime::function_name::named]
    fn escaped(&mut self, escaped: &ParseTreeType<'t>) -> Result<()> {
        let context = function_name!();
        trace!("{}", self.trace_item_stack(context));
        let escaped = escaped.token()?.clone();
        let escaped_built = Escaped { escaped };
        // Calling user action here
        self.user_grammar.escaped(&escaped_built)?;
        self.push(ASTType::Escaped(escaped_built), context);
        Ok(())
    }

    /// Semantic action for production 13:
    ///
    /// `EscapedLineEnd: <String>/\\[\s--\n\r]*\r?\n/;`
    ///
    #[parol_runtime::function_name::named]
    fn escaped_line_end(&mut self, escaped_line_end: &ParseTreeType<'t>) -> Result<()> {
        let context = function_name!();
        trace!("{}", self.trace_item_stack(context));
        let escaped_line_end = escaped_line_end.token()?.clone();
        let escaped_line_end_built = EscapedLineEnd { escaped_line_end };
        // Calling user action here
        self.user_grammar
            .escaped_line_end(&escaped_line_end_built)?;
        self.push(ASTType::EscapedLineEnd(escaped_line_end_built), context);
        Ok(())
    }

    /// Semantic action for production 14:
    ///
    /// `NoneQuote: <String>/[^"\\]+/;`
    ///
    #[parol_runtime::function_name::named]
    fn none_quote(&mut self, none_quote: &ParseTreeType<'t>) -> Result<()> {
        let context = function_name!();
        trace!("{}", self.trace_item_stack(context));
        let none_quote = none_quote.token()?.clone();
        let none_quote_built = NoneQuote { none_quote };
        // Calling user action here
        self.user_grammar.none_quote(&none_quote_built)?;
        self.push(ASTType::NoneQuote(none_quote_built), context);
        Ok(())
    }

    /// Semantic action for production 15:
    ///
    /// `StringDelimiter: <INITIAL, String>/"/;`
    ///
    #[parol_runtime::function_name::named]
    fn string_delimiter(&mut self, string_delimiter: &ParseTreeType<'t>) -> Result<()> {
        let context = function_name!();
        trace!("{}", self.trace_item_stack(context));
        let string_delimiter = string_delimiter.token()?.clone();
        let string_delimiter_built = StringDelimiter { string_delimiter };
        // Calling user action here
        self.user_grammar
            .string_delimiter(&string_delimiter_built)?;
        self.push(ASTType::StringDelimiter(string_delimiter_built), context);
        Ok(())
    }
}

impl<'t> UserActionsTrait<'t> for ScannerStatesGrammarAuto<'t, '_> {
    ///
    /// This function is implemented automatically for the user's item ScannerStatesGrammar.
    ///
    fn call_semantic_action_for_production_number(
        &mut self,
        prod_num: usize,
        children: &[ParseTreeType<'t>],
    ) -> Result<()> {
        match prod_num {
            0 => self.start(&children[0]),
            1 => self.start_list_0(&children[0], &children[1]),
            2 => self.start_list_1(),
            3 => self.content_0(&children[0]),
            4 => self.content_1(&children[0], &children[1], &children[2]),
            5 => self.string_content(&children[0]),
            6 => self.string_content_list_0(&children[0], &children[1]),
            7 => self.string_content_list_1(),
            8 => self.string_element_0(&children[0]),
            9 => self.string_element_1(&children[0]),
            10 => self.string_element_2(&children[0]),
            11 => self.identifier(&children[0]),
            12 => self.escaped(&children[0]),
            13 => self.escaped_line_end(&children[0]),
            14 => self.none_quote(&children[0]),
            15 => self.string_delimiter(&children[0]),
            _ => Err(ParserError::InternalError(format!(
                "Unhandled production number: {}",
                prod_num
            ))
            .into()),
        }
    }

    fn on_comment(&mut self, token: Token<'t>) {
        self.user_grammar.on_comment(token)
    }
}
