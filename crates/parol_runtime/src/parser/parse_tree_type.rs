use crate::{lexer::token::PTToken, ParserError, Token};

use std::fmt::{Display, Formatter};
use syntree_layout::Visualize;

///
/// The type of the elements in the parse tree.
///
/// The lifetime parameter `'t` refers to the lifetime of the scanned text.
///
#[derive(Debug, Clone)]
pub enum ParseTreeType<'t> {
    ///
    /// A scanned token.
    ///
    T(Token<'t>),

    ///
    /// A non-terminal name.
    /// All names are of static lifetime (see NON_TERMINALS slice of non-terminal names).
    ///
    N(&'static str),
}

impl<'t> ParseTreeType<'t> {
    ///
    /// Tries to access the Token of the ParseTreeType.
    /// Can fail if the entry is no terminal (i.e. a non-terminal).
    ///
    pub fn token(&self) -> Result<&Token<'t>, ParserError> {
        match self {
            Self::T(t) => Ok(t),
            _ => Err(ParserError::InternalError(format!("{} is no token!", self))),
        }
    }

    ///
    /// Tries to access the scanned text of the ParseTreeType.
    /// Can fail if the entry is no terminal (i.e. a non-terminal).
    ///
    pub fn text(&self) -> Result<&str, ParserError> {
        match self {
            Self::T(t) => Ok(t.text()),
            _ => Err(ParserError::InternalError(format!("{} is no token!", self))),
        }
    }
}

///
/// Implementation of the Visualize trait to support the visualization of the
/// ParseTreeType in a tree layout.
///
impl Visualize for ParseTreeType<'_> {
    fn visualize(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseTreeType::T(t) => write!(f, "{}", t),
            ParseTreeType::N(n) => write!(f, "{}", n),
        }
    }
    fn emphasize(&self) -> bool {
        matches!(self, Self::T(_))
    }
}

impl Display for ParseTreeType<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Self::T(t) => write!(f, "T({})", t),
            Self::N(n) => write!(f, "N({})", n),
        }
    }
}

/// Parse tree representation.
/// It is uses to build the syntree tree. The syntree tree expects the tree type to be Copy.
#[derive(Debug, Clone, Copy)]
pub enum SynTree {
    /// A terminal node
    Terminal(PTToken),
    /// A non-terminal node
    NonTerminal(&'static str),
}

impl Display for SynTree {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            SynTree::Terminal(t) => write!(f, "{}", t),
            SynTree::NonTerminal(n) => write!(f, "{}", n),
        }
    }
}
