/*!
This crate provides the runtime library used by parsers that have been generated by the `parol`
parser generator.

In most cases you don't need to understand the inner details of this crate because `parol` generates
all necessary scaffolding and plumping for the typical user automatically.

The most likely used parts are those returned to the user including the error types defined in the
module [errors].

If you use parsers generated in [vanilla mode](https://jsinger67.github.io/VanillaMode.html) you
should understand the types that are handed over to your semantic actions. You will find them in the
module [parser::parse_tree_type].
 */

///
/// Module that provides types for lexical analysis.
///
pub mod lexer;
pub use lexer::{
    FormatToken, Location, LocationBuilder, Span, TerminalIndex, ToSpan, Token, TokenIter,
    TokenNumber, TokenStream, Tokenizer,
};

///
/// Module that provides types for syntactical analysis.
///
pub mod parser;
pub use parser::{
    LLKParser, LookaheadDFA, NonTerminalIndex, ParseStack, ParseTree, ParseTreeType, ParseType,
    Production, ProductionIndex, ScannerIndex, StateIndex, Trans, UserActionsTrait,
};

///
/// Module that provides types for the LR parser.
///
pub mod lr_parser;

///
/// Module with error types reported from this crate.
///
pub mod errors;
pub use errors::{
    FileSource, LexerError, ParolError, ParserError, Report, Result, SyntaxError, TokenVec,
    UnexpectedToken,
};

// re-export
pub use codespan_reporting;
#[cfg(feature = "auto_generation")]
pub use derive_builder;
pub use function_name;
pub use log;
pub use once_cell;
#[cfg(feature = "auto_generation")]
pub use parol_macros;
pub use syntree;
pub use syntree_layout;
