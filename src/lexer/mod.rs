#![forbid(missing_docs)]

use regex::Regex;

///
/// Type used as an identifier for token types that the user provides.
///
pub type TerminalIndex = usize;

///
/// Module that provides basic token implementation.
///
pub mod token;
pub use token::{Token, BLOCK_COMMENT, EOI, FIRST_USER_TOKEN, LINE_COMMENT, NEW_LINE, WHITESPACE};

///
/// The owned token type is a token with owning token text
///
pub mod owned_token;
pub use owned_token::OwnedToken;

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
    static ref RX_NEW_LINE: Regex =
        Regex::new(r"\r\n|\r|\n").expect("error parsing regex: RX_NEW_LINE");
}
