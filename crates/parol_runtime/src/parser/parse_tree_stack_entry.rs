use crate::{ParseTreeType, ParserError, Token};
use anyhow::{anyhow, Result};
use id_tree::{Node, NodeId, Tree};

///
/// The type of elements in the parser's parse tree stack.
/// * 'Nd' references nodes not yet inserted into the parse tree.
/// * 'Id' holds node ids to nodes that are already part of the parse tree.
///
#[derive(Debug)]
pub enum ParseTreeStackEntry<'t> {
    /// The node is not inserted into the parse tree yet.
    /// Thus we can access it directly.
    Nd(Node<ParseTreeType<'t>>),

    /// The node is already inserted into the parse tree.
    /// Wee need to lookup the node in the parse tree via the NodeId.
    Id(NodeId),
}

impl<'t> ParseTreeStackEntry<'t> {
    ///
    /// Abstracts from the actual place where the node exists and returns the
    /// inner ParseTreeType.
    ///
    /// `'a` refers to the lifetime of self.
    /// `'b` refers to the lifetime of the parse tree.
    ///
    pub fn get_parse_tree_type<'a, 'b>(
        &'a self,
        parse_tree: &'b Tree<ParseTreeType<'t>>,
    ) -> &'a ParseTreeType
    where
        'b: 'a,
    {
        match self {
            Self::Nd(n) => n.data(),
            Self::Id(i) => parse_tree.get(i).unwrap().data(),
        }
    }

    ///
    /// Tries to access the Token of the ParseTreeStackEntry.
    /// Can fail if the entry is no terminal (i.e. a non-terminal).
    ///
    /// `'a` refers to the lifetime of self.
    /// `'b` refers to the lifetime of the parse tree.
    ///
    /// `'t` refers to the lifetime of the scanned text.
    ///
    pub fn token<'a, 'b>(&'a self, parse_tree: &'b Tree<ParseTreeType<'t>>) -> Result<&'a Token<'t>>
    where
        'b: 'a,
    {
        match self {
            Self::Nd(n) => n.data().token(),
            Self::Id(i) => parse_tree.get(i).unwrap().data().token(),
        }
    }

    ///
    /// Tries to access the text of the ParseTreeStackEntry.
    /// Can fail if the entry is no terminal (i.e. a non-terminal).
    ///
    /// `'a` refers to the lifetime of self.
    /// `'b` refers to the lifetime of the parse tree.
    ///
    pub fn text<'a, 'b>(&'a self, parse_tree: &'b Tree<ParseTreeType>) -> Result<&'a str>
    where
        'b: 'a,
    {
        match self {
            Self::Nd(node) => {
                let token = node.data().token()?;
                Ok(&token.text)
            }
            Self::Id(i) => {
                let node = parse_tree
                    .get(i)
                    .map_err(|e| anyhow!(ParserError::IdTreeError { source: e }))?;
                let token = node.data().token()?;
                Ok(&token.text)
            }
        }
    }
}
