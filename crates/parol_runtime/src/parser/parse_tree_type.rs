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

/// A trait that associates kind types with the grammar.
pub trait GrammarEnums {
    /// The kind of the terminal.
    type TerminalEnum: TerminalEnum;
    /// The kind of the non-terminal.
    type NonTerminalEnum: NonTerminalEnum;
}

/// A trait that a terminal enum must implement.
pub trait TerminalEnum: Copy + std::fmt::Debug {
    /// Creates a terminal from an index.
    fn from_terminal_index(index: u16) -> Self;
}

/// A trait that a non-terminal enum must implement.
pub trait NonTerminalEnum: Copy + std::fmt::Debug {
    /// Creates a non-terminal from a name.
    fn from_non_terminal_name(name: &str) -> Self;
}

#[derive(Debug)]
/// A parse tree that is associated with a grammar.
pub enum SynTree2<G: GrammarEnums> {
    /// A terminal node.
    Terminal(SynTreeTerminal<G>),
    /// A non-terminal node.
    NonTerminal(SynTreeNonTerminal<G>),
}

impl<G: GrammarEnums> Copy for SynTree2<G> {}

impl<G: GrammarEnums> Clone for SynTree2<G> {
    fn clone(&self) -> Self {
        *self
    }
}

#[derive(Debug)]
/// A terminal node.
pub struct SynTreeTerminal<G>
where
    G: GrammarEnums,
    G: ?Sized,
{
    /// The kind of the terminal.
    pub kind: G::TerminalEnum,
    /// The data of the terminal.
    pub data: TerminalData,
}

impl<G: GrammarEnums> Copy for SynTreeTerminal<G> {}

impl<G: GrammarEnums> Clone for SynTreeTerminal<G> {
    fn clone(&self) -> Self {
        *self
    }
}

#[derive(Debug, Clone, Copy)]
/// A span that is only valid within the context of the input text.
pub struct InputSpan {
    /// The start of the span.
    pub start: u32,
    /// The end of the span.
    pub end: u32,
}

#[derive(Debug, Clone, Copy)]
/// A dynamic token id that provided by the user land.
pub struct DynamicTokenId(pub u32);

#[derive(Debug, Clone, Copy)]
/// The data of the terminal.
pub enum TerminalData {
    /// A terminal that is associated with an input span.
    Input(InputSpan),
    /// A terminal that is associated with a dynamic token id.
    Dynamic(DynamicTokenId),
}

#[derive(Debug)]
/// A non-terminal node.
pub struct SynTreeNonTerminal<G>
where
    G: GrammarEnums,
    G: ?Sized,
{
    /// The kind of the non-terminal.
    pub kind: G::NonTerminalEnum,
}

impl<G: GrammarEnums> Copy for SynTreeNonTerminal<G> {}

impl<G: GrammarEnums> Clone for SynTreeNonTerminal<G> {
    fn clone(&self) -> Self {
        *self
    }
}

/// Factory trait for creating custom syntree nodes.
pub trait SynTreeNode<'t>: Copy {
    /// Creates a syntree node from a non-terminal name.
    fn from_non_terminal(name: &'static str) -> Self;
    /// Creates a syntree node from a token.
    fn from_token(token: &Token<'t>) -> Self;
}

impl<'t> SynTreeNode<'t> for SynTree {
    fn from_token(token: &Token<'t>) -> Self {
        SynTree::Terminal(token.into())
    }
    fn from_non_terminal(name: &'static str) -> Self {
        SynTree::NonTerminal(name)
    }
}

impl<'t, G: GrammarEnums> SynTreeNode<'t> for SynTree2<G> {
    fn from_token(token: &Token<'t>) -> Self {
        SynTree2::Terminal(SynTreeTerminal {
            kind: G::TerminalEnum::from_terminal_index(token.token_type),
            data: TerminalData::Input(InputSpan {
                start: token.location.start,
                end: token.location.end,
            }),
        })
    }
    fn from_non_terminal(name: &'static str) -> Self {
        SynTree2::NonTerminal(SynTreeNonTerminal {
            kind: G::NonTerminalEnum::from_non_terminal_name(name),
        })
    }
}
