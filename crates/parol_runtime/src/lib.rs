///
/// Module that provides types for lexical analysis.
///
pub mod lexer;
pub use lexer::{
    FormatToken, Location, LocationBuilder, Span, TerminalIndex, ToSpan, Token, TokenIter,
    TokenStream, Tokenizer,
};

///
/// Module that provides types for syntactical analysis.
///
pub mod parser;
pub use parser::{
    DFAState, DFATransition, LLKParser, LookaheadDFA, NonTerminalIndex, ParseStack,
    ParseTreeStackEntry, ParseTreeType, ParseType, Production, ProductionIndex, ScannerIndex,
    StateIndex, UserActionsTrait,
};

///
/// error_chain's error module that auto-creates basic error types.
///
#[macro_use]
pub mod errors;
pub use errors::{
    FileSource, LexerError, ParolError, ParserError, Result, TokenVec, UnexpectedToken,
};

// re-export
#[cfg(feature = "auto_generation")]
pub use derive_builder;
pub use function_name;
pub use id_tree;
pub use id_tree_layout;
pub use log;
pub use once_cell;
#[cfg(feature = "auto_generation")]
pub use parol_macros;
