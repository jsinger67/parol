use crate::lexer::OwnedToken;
use crate::parser::ParseTreeType;
use anyhow::Result;
use id_tree::{Node, NodeId, Tree};

///
/// The type of elements in the parser's parse tree stack.
/// * 'Nd' references nodes not yet inserted into the parse tree.
/// * 'Id' holds node ids to nodes that are already part of the parse tree.
///
#[derive(Debug)]
pub enum ParseTreeStackEntry {
    /// The node is not inserted into the parse tree yet.
    /// Thus we can access it directly.
    Nd(Node<ParseTreeType>),

    /// The node is already inserted into the parse tree.
    /// Wee need to lookup the node in the parse tree via the NodeId.
    Id(NodeId),
}

impl ParseTreeStackEntry {
    ///
    /// Abstracts from the actual place where the node exists and returns the
    /// inner ParseTreeType.
    ///
    pub fn get_parse_tree_type<'t>(
        &'t self,
        parse_tree: &'t Tree<ParseTreeType>,
    ) -> &'t ParseTreeType {
        match self {
            Self::Nd(n) => n.data(),
            Self::Id(i) => parse_tree.get(i).unwrap().data(),
        }
    }

    ///
    /// Tries to access the OwnedToken of the ParseTreeStackEntry.
    /// Can fail if the entry is no terminal (i.e. a non-terminal).
    ///
    pub fn token<'t>(&'t self, parse_tree: &'t Tree<ParseTreeType>) -> Result<&'t OwnedToken> {
        match self {
            Self::Nd(n) => n.data().token(),
            Self::Id(i) => parse_tree.get(i).unwrap().data().token(),
        }
    }

    ///
    /// Tries to access the text of the ParseTreeStackEntry.
    /// Can fail if the entry is no terminal (i.e. a non-terminal).
    ///
    pub fn symbol<'t>(&'t self, parse_tree: &'t Tree<ParseTreeType>) -> Result<&'t String> {
        match self {
            Self::Nd(node) => {
                let token = node.data().token()?;
                Ok(&token.symbol)
            }
            Self::Id(i) => {
                let node = parse_tree.get(i)?;
                let token = node.data().token()?;
                Ok(&token.symbol)
            }
        }
    }
}
