use crate::{FormatToken, Span, TerminalIndex, ToSpan};
use std::borrow::Cow;
use std::convert::From;
use std::fmt::{Debug, Display, Error, Formatter};

use super::{Location, TokenNumber};

//
// Special token constants the lexer has to deal with regularly.
// There are some special fix values used for common token types.
//

/// End of input constant
pub const EOI: TerminalIndex = 0;
/// New line token
pub const NEW_LINE: TerminalIndex = 1;
/// Whitespace token
pub const WHITESPACE: TerminalIndex = 2;
/// Line comment token
pub const LINE_COMMENT: TerminalIndex = 3;
/// Block comment token
pub const BLOCK_COMMENT: TerminalIndex = 4;
/// Index of the first user token.
pub const FIRST_USER_TOKEN: TerminalIndex = 5;

/// End of input token text
const EOI_TOKEN: &str = "$";

///
/// The Token<'t> type represents a scanned token.
/// It has a reference to the scanned text in the text member.
///
/// The lifetime parameter `'t` refers to the lifetime of the scanned text.
///
#[derive(Debug, Clone, Default, Eq, PartialEq, PartialOrd, Ord)]
pub struct Token<'t> {
    /// The matched string
    pub(crate) text: Cow<'t, str>,

    /// The index of the terminal in the augmented terminal list
    pub token_type: TerminalIndex,

    /// Position information
    pub location: Location,

    /// Unique token number that allows ordering of tokens from different contexts, e.g. comment
    /// tokens can be intermingled with normal tokens.
    pub token_number: TokenNumber,
}

impl<'t> Token<'t> {
    ///
    /// Creates an End-Of-Input token
    ///
    pub fn eoi(token_number: TokenNumber) -> Self {
        Self {
            text: EOI_TOKEN.into(),
            token_type: EOI,
            location: Location::default(),
            token_number,
        }
    }

    ///
    /// Creates a token with given values.
    ///
    pub fn with<T>(
        text: T,
        token_type: TerminalIndex,
        location: Location,
        token_number: TokenNumber,
    ) -> Self
    where
        T: Into<Cow<'t, str>>,
    {
        Self {
            text: text.into(),
            token_type,
            location,
            token_number,
        }
    }

    /// Change the location of the token after it's creation
    pub fn with_location(mut self, location: Location) -> Self {
        self.location = location;
        self
    }

    /// Change the location of the token after it's creation
    pub fn with_type(mut self, token_type: TerminalIndex) -> Self {
        self.token_type = token_type;
        self
    }

    ///
    /// Indicates whether the token is normally skipped by the TokenStream.
    /// The behavior is independent from the language.
    ///
    #[inline]
    pub fn is_skip_token(&self) -> bool {
        self.token_type > EOI && self.token_type < FIRST_USER_TOKEN
    }

    ///
    /// Indicates whether the token is a comment token.
    ///
    #[inline]
    pub fn is_comment_token(&self) -> bool {
        self.token_type == LINE_COMMENT || self.token_type == BLOCK_COMMENT
    }

    ///
    /// Accesses the token's scanned text
    ///
    pub fn text(&self) -> &str {
        self.text.as_ref()
    }

    ///
    /// Creates an owned instance of the token from a shared reference
    ///
    pub fn to_owned(&self) -> Token<'static> {
        Token {
            text: Cow::Owned(self.text.clone().into_owned()),
            token_type: self.token_type,
            location: self.location.clone(),
            token_number: self.token_number,
        }
    }

    ///
    /// Creates an owned instance of the token and consumes self
    ///
    pub fn into_owned(self) -> Token<'static> {
        Token {
            text: Cow::Owned(self.text.into_owned()),
            token_type: self.token_type,
            location: self.location,
            token_number: self.token_number,
        }
    }
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        let (c1, c2) = if self.text.starts_with('\'') {
            ('<', '>')
        } else {
            ('\'', '\'')
        };
        write!(
            f,
            "{}{}{}, Ty:{}, at {}..{}[{}]",
            c1,
            self.text,
            c2,
            self.token_type,
            self.location.start,
            self.location.end,
            self.location
        )
    }
}

impl FormatToken for Token<'_> {
    fn format(&self, terminal_names: &'static [&'static str]) -> String {
        let name = terminal_names[self.token_type as usize];
        format!(
            "{} ({}) at {}[{}]",
            self.text, name, self.location, self.token_number,
        )
    }
}

impl From<&Token<'_>> for std::ops::Range<usize> {
    fn from(token: &Token<'_>) -> Self {
        (&token.location).into()
    }
}

impl From<&Token<'_>> for Location {
    fn from(token: &Token<'_>) -> Self {
        token.location.clone()
    }
}

impl From<&Token<'_>> for Span {
    fn from(token: &Token<'_>) -> Self {
        (Into::<std::ops::Range<usize>>::into(&token.location)).into()
    }
}

impl ToSpan for Token<'_> {
    fn span(&self) -> Span {
        self.into()
    }
}

/// Lightweight version of Token that is Copy for use in parse trees
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub struct PTToken {
    /// The index of the terminal in the augmented terminal list
    pub token_type: TerminalIndex,

    /// Range start information
    pub start: usize,
    /// Range end information
    pub end: usize,

    /// Unique token number that allows ordering of tokens from different contexts, e.g. comment
    /// tokens can be intermingled with normal tokens.
    pub token_number: TokenNumber,
}

impl PTToken {
    /// Calculate the length of the token
    #[inline]
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// Returns true if the token is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }
}

impl From<&Token<'_>> for PTToken {
    fn from(token: &Token<'_>) -> Self {
        Self {
            token_type: token.token_type,
            start: token.location.start(),
            end: token.location.end(),
            token_number: token.token_number,
        }
    }
}

impl Display for PTToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(
            f,
            "Ty:{}, at {}..{}[{}]",
            self.token_type, self.start, self.end, self.token_number
        )
    }
}
