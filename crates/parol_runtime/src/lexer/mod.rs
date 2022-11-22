#![forbid(missing_docs)]

use regex::Regex;

///
/// Type used for token types the user provides.
///
pub type TerminalIndex = usize;

///
/// Module with common formatting trait
///
pub mod format_token;
pub use format_token::FormatToken;

///
/// Module with a location type
///
pub mod location;
pub use location::Location;
///
/// Module that provides basic token implementation.
///
pub mod token;
pub use token::{Token, BLOCK_COMMENT, EOI, FIRST_USER_TOKEN, LINE_COMMENT, NEW_LINE, WHITESPACE};

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

lazy_static! {
    static ref RX_NEW_LINE: Regex = Regex::new(r"\r?\n").expect("error parsing regex: RX_NEW_LINE");
}
