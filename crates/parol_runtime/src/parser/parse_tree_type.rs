use crate::{lexer::token::PTToken, ParserError, Token};

use std::fmt::{Display, Formatter};
use syntree_layout::Visualize;

use super::{
    parser_types::{SynTreeFlavor, TreeBuilder},
    ParseTree,
};

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

/// The type of a node when the typed syntree option is enabled.
pub type Node<'a, T, Nt> = syntree::Node<'a, SynTree2<T, Nt>, SynTreeFlavor>;

/// An extension trait that provides methods for nodes.
pub trait NodeExt<T: Copy, Nt: Copy> {
    /// Finds a child node in the node from the cursor position (with skipping invalid children), and returns the new cursor position and the child node if found.
    fn find_child(&self, cursor: usize, child: NodeKind<T, Nt>) -> Option<(usize, Node<T, Nt>)>;
}

impl<'a, T, Nt> NodeExt<T, Nt> for Node<'a, T, Nt>
where
    T: Copy + TerminalEnum + PartialEq,
    Nt: Copy + PartialEq,
{
    fn find_child(
        &self,
        cursor: usize,
        child: NodeKind<T, Nt>,
    ) -> Option<(usize, Node<'a, T, Nt>)> {
        for (i, node) in self.children().enumerate().skip(cursor) {
            if node.value().kind() == child {
                return Some((i + 1, node));
            }
            match node.value().kind() {
                NodeKind::Terminal(t) => {
                    if t.is_builtin_whitespace() {
                        continue;
                    }
                    if t.is_builtin_new_line() {
                        continue;
                    }
                }
                NodeKind::NonTerminal(_) => {}
            }
        }
        None
    }
}

/// What kinds of children are expected for a node kind.
#[derive(Debug, Clone, Copy)]
pub enum ExpectedChildrenKinds<T, Nt>
where
    T: 'static,
    Nt: 'static,
{
    /// A node kind that expects one of the given child kinds. Corresponds to enum ast types.
    OneOf(&'static [NodeKind<T, Nt>]),
    /// A node kind that expects a sequence of child kinds. Corresponds to struct ast types.
    Sequence(&'static [NodeKind<T, Nt>]),
}

impl<T, Nt> ExpectedChildrenKinds<T, Nt> {
    /// Asserts that the node is a valid with this expected children.
    pub fn assert_node_syntax(&self, node: Node<T, Nt>) -> bool
    where
        T: Copy + PartialEq + TerminalEnum,
        Nt: Copy + PartialEq,
    {
        match self {
            ExpectedChildrenKinds::OneOf(children) => {
                for child in *children {
                    if *child == node.value().kind() {
                        return true;
                    }
                }
                false
            }
            ExpectedChildrenKinds::Sequence(children) => {
                let mut cursor = 0;
                for child in *children {
                    if let Some((new_cursor, _)) = node.find_child(cursor, *child) {
                        cursor = new_cursor;
                    } else {
                        return false;
                    }
                }
                true
            }
        }
    }
}

/// A trait that a node kind must implement that returns the expected child nodes for this node kind for grammar assertions.
pub trait ExpectedChildren<T, Nt> {
    /// Expected child nodes for this node kind.
    fn expected_children(&self) -> ExpectedChildrenKinds<T, Nt>;
}

/// The type of a child node.
pub enum ChildKind<T, Nt> {
    /// A terminal node.
    Terminal(T),
    /// A non-terminal node.
    NonTerminal(Nt),
    /// A vector of non-terminal nodes.
    Vec(Nt),
    /// An optional non-terminal node.
    Optional(Nt),
}

/// A trait that a terminal enum must implement.
pub trait TerminalEnum: Copy + std::fmt::Debug {
    /// Creates a terminal from an index.
    fn from_terminal_index(index: u16) -> Self;

    /// Returns true if the terminal is a parol's built-in (not user defined) new line token.
    fn is_builtin_new_line(&self) -> bool;

    /// Returns true if the terminal is a parol's built-in (not user defined) whitespace token.
    fn is_builtin_whitespace(&self) -> bool;
}

/// A trait that a non-terminal enum must implement.
pub trait NonTerminalEnum: Copy + std::fmt::Debug {
    /// Creates a non-terminal from a name.
    fn from_non_terminal_name(name: &str) -> Self;
}

#[derive(Debug, Clone, Copy)]
/// A parse tree that is associated with a grammar.
pub enum SynTree2<T, Nt> {
    /// A terminal node.
    Terminal(SynTreeTerminal<T>),
    /// A non-terminal node.
    NonTerminal(SynTreeNonTerminal<Nt>),
}

impl<T: Copy, Nt: Copy> SynTree2<T, Nt> {
    /// The kind of the node.
    pub fn kind(&self) -> NodeKind<T, Nt> {
        match self {
            SynTree2::Terminal(data) => NodeKind::Terminal(data.kind),
            SynTree2::NonTerminal(data) => NodeKind::NonTerminal(data.kind),
        }
    }
}

impl<T, Nt> std::fmt::Display for SynTree2<T, Nt>
where
    T: std::fmt::Display,
    Nt: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SynTree2::Terminal(t) => write!(f, "{}", t),
            SynTree2::NonTerminal(n) => write!(f, "{}", n),
        }
    }
}

#[derive(Debug, Clone, Copy)]
/// A terminal node.
pub struct SynTreeTerminal<T> {
    /// The kind of the terminal.
    pub kind: T,
    /// The data of the terminal.
    pub data: TerminalData,
}

impl<T> std::fmt::Display for SynTreeTerminal<T>
where
    T: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)
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

#[derive(Debug, Clone, Copy)]
/// A non-terminal node.
pub struct SynTreeNonTerminal<Nt> {
    /// The kind of the non-terminal.
    pub kind: Nt,
}

impl<Nt> std::fmt::Display for SynTreeNonTerminal<Nt>
where
    Nt: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)
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

impl<'t, T, Nt> AstNode<'t> for SynTree2<T, Nt>
where
    T: TerminalEnum,
    Nt: NonTerminalEnum,
{
    fn from_token(token: &Token<'t>) -> Self {
        SynTree2::Terminal(SynTreeTerminal {
            kind: T::from_terminal_index(token.token_type),
            data: TerminalData::Input(InputSpan {
                start: token.location.start,
                end: token.location.end,
            }),
        })
    }
    fn from_non_terminal(name: &'static str) -> Self {
        SynTree2::NonTerminal(SynTreeNonTerminal {
            kind: Nt::from_non_terminal_name(name),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// The kind of a node.
pub enum NodeKind<T, Nt> {
    /// A terminal node.
    Terminal(T),
    /// A non-terminal node.
    NonTerminal(Nt),
}

impl<T, Nt> ExpectedChildren<T, Nt> for NodeKind<T, Nt>
where
    Nt: ExpectedChildren<T, Nt>,
{
    fn expected_children(&self) -> ExpectedChildrenKinds<T, Nt> {
        match self {
            NodeKind::Terminal(_) => ExpectedChildrenKinds::Sequence(&[]),
            NodeKind::NonTerminal(nt) => nt.expected_children(),
        }
    }
}

impl<T, Nt> ExpectedChildren<T, Nt> for SynTree2<T, Nt>
where
    Nt: ExpectedChildren<T, Nt>,
{
    fn expected_children(&self) -> ExpectedChildrenKinds<T, Nt> {
        match self {
            SynTree2::Terminal(_) => ExpectedChildrenKinds::Sequence(&[]),
            SynTree2::NonTerminal(nt) => nt.kind.expected_children(),
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
