#![forbid(missing_docs)]

///
/// Type used for token types the user provides.
///
pub type TerminalIndex = u16;

///
/// Type used for token numbers in tokens.
///
pub type TokenNumber = u32;

///
/// Invalid token number
/// These tokens have been inserted by the parser during error recovery.
/// They have no valid index within the original token stream.
/// ATTENTION: This could lead to invalid array index access.
/// TODO:
/// Maybe create a special token type that is used in error recovery scenarios and that can't
/// be confused with normal tokens.
/// Or ensure that such tokens never leave the parser.
///
pub const INVALID_TOKEN_NUMBER: TokenNumber = TokenNumber::MAX;

///
/// Module with common formatting trait
///
pub mod format_token;
pub use format_token::FormatToken;

///
/// Module with a location type
///
pub mod location;
pub use location::{Location, LocationBuilder};

///
/// Module to support handling of std::ops::Range
///
pub mod rng;
pub use rng::{Span, ToSpan};

///
/// Module that provides the type used for scanner configurations.
///
pub mod scanner_config;
pub use scanner_config::ScannerConfig;

///
/// Module that provides basic token implementation.
///
pub mod token;
pub use token::{Token, BLOCK_COMMENT, EOI, FIRST_USER_TOKEN, LINE_COMMENT, NEW_LINE, WHITESPACE};

mod token_buffer;
pub(crate) use token_buffer::TokenBuffer;

///
/// Module that provides the Tokenizer type.
///
pub mod tokenizer;
pub use tokenizer::Tokenizer;

///
/// Module that provides the TokenIter type.
///
pub mod token_iter;
pub use token_iter::TokenIter;

///
/// Module that provides the TokenStream type.
///
pub mod token_stream;
pub use token_stream::TokenStream;
