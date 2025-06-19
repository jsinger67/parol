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
/// Module that provides basic token implementation.
///
pub mod token;
pub use token::{BLOCK_COMMENT, EOI, FIRST_USER_TOKEN, LINE_COMMENT, NEW_LINE, Token, WHITESPACE};

mod token_buffer;
pub(crate) use token_buffer::TokenBuffer;

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

///
/// This is an  unmatchable regular expression.
/// It is normally not included in the generated Regex's source but stands for
/// tokens that should be skipped, i.e. if a language doesn't support block
/// comments you could mark the regex on index token::BLOCK_COMMENT as
/// unmatchable.
///
pub const UNMATCHABLE_TOKEN: &str = r"\w\b\w";

///
/// Regular expression for new lines
///
pub const NEW_LINE_TOKEN: &str = r"\r\n|\r|\n";

///
/// Regular expression for any whitespace except newline characters
///
pub const WHITESPACE_TOKEN: &str = r"[\s--\r\n]+";

///
/// Regular expression that matches any other token. With this you can detect
/// so far unmatched tokens. It is only used for error detection during lexical
/// analysis.
///
pub const ERROR_TOKEN: &str = r###"."###;
