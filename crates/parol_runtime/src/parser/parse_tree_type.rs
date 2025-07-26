use crate::{ParserError, Token, lexer::token::PTToken};

use std::fmt::{Display, Formatter};
use syntree_layout::Visualize;

use super::{ParseTree, parser_types::TreeBuilder};

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
            _ => Err(ParserError::InternalError(format!("{self} is no token!"))),
        }
    }

    ///
    /// Tries to access the scanned text of the ParseTreeType.
    /// Can fail if the entry is no terminal (i.e. a non-terminal).
    ///
    pub fn text(&self) -> Result<&str, ParserError> {
        match self {
            Self::T(t) => Ok(t.text()),
            _ => Err(ParserError::InternalError(format!("{self} is no token!"))),
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
            ParseTreeType::T(t) => write!(f, "{t}"),
            ParseTreeType::N(n) => write!(f, "{n}"),
        }
    }
    fn emphasize(&self) -> bool {
        matches!(self, Self::T(_))
    }
}

impl Display for ParseTreeType<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Self::T(t) => write!(f, "T({t})"),
            Self::N(n) => write!(f, "N({n})"),
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
            SynTree::Terminal(t) => write!(f, "{t}"),
            SynTree::NonTerminal(n) => write!(f, "{n}"),
        }
    }
}

/// A trait that a tree builder must implement.
pub trait TreeConstruct<'t> {
    /// The error type of the tree builder.
    type Error;
    /// The type of the tree.
    type Tree;

    /// Creates a node from a non-terminal name.
    fn open_non_terminal(
        &mut self,
        name: &'static str,
        size_hint: Option<usize>,
    ) -> Result<(), Self::Error>;

    /// Closes a non-terminal node.
    fn close_non_terminal(&mut self) -> Result<(), Self::Error>;

    /// Creates a token node.
    fn add_token(&mut self, token: &Token<'t>) -> Result<(), Self::Error>;

    /// Returns the tree.
    fn build(self) -> Result<Self::Tree, Self::Error>;
}

impl<'t, T: AstNode<'t>> TreeConstruct<'t> for TreeBuilder<T> {
    type Error = syntree::Error;
    type Tree = ParseTree<T>;

    fn open_non_terminal(
        &mut self,
        name: &'static str,
        _size_hint: Option<usize>,
    ) -> Result<(), Self::Error> {
        self.open(T::from_non_terminal(name))?;
        Ok(())
    }

    fn close_non_terminal(&mut self) -> Result<(), Self::Error> {
        self.close()?;
        Ok(())
    }

    fn add_token(&mut self, token: &Token<'t>) -> Result<(), Self::Error> {
        self.token(T::from_token(token), token.location.len())?;
        Ok(())
    }

    fn build(self) -> Result<Self::Tree, Self::Error> {
        self.build()
    }
}

/// Factory trait for creating custom syntree nodes.
pub trait AstNode<'t>: Copy {
    /// Creates a syntree node from a non-terminal name.
    fn from_non_terminal(name: &'static str) -> Self;
    /// Creates a syntree node from a token.
    fn from_token(token: &Token<'t>) -> Self;
}

impl<'t> AstNode<'t> for SynTree {
    fn from_token(token: &Token<'t>) -> Self {
        SynTree::Terminal(token.into())
    }
    fn from_non_terminal(name: &'static str) -> Self {
        SynTree::NonTerminal(name)
    }
}
