use std::{convert::TryFrom, fmt::Display};

use crate::{
    parser::{
        parse_tree_type::{SynTree, TreeConstruct},
        parser_types::{SynTreeFlavor, TreeBuilder},
    },
    ParseTreeType, Token,
};
use syntree::{Builder, Tree};

/// Parse tree representation.
/// The lifetime `'t` is the lifetime of the input text.
#[derive(Debug, Clone)]
pub enum LRParseTree<'t> {
    Terminal(Token<'t>),
    NonTerminal(&'static str, Option<Vec<LRParseTree<'t>>>),
}

impl LRParseTree<'_> {
    pub(crate) fn is_skip_token(&self) -> bool {
        match self {
            LRParseTree::Terminal(token) => token.is_skip_token(),
            LRParseTree::NonTerminal(_, _) => false,
        }
    }
}

impl Display for LRParseTree<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LRParseTree::Terminal(token) => write!(f, "{}", token),
            LRParseTree::NonTerminal(name, Some(children)) => {
                write!(f, "{}", name)?;
                if !children.is_empty() {
                    write!(f, "(")?;
                    for (i, child) in children.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", child)?;
                    }
                    write!(f, ")")?;
                }
                Ok(())
            }
            LRParseTree::NonTerminal(name, None) => write!(f, "{}", name),
        }
    }
}

// Build a tree from a parse tree in a depth-first manner. This is a recursive function that
// traverses the parse tree and builds the syntree tree.
// This can possibly result in a stack overflow if the parse tree is too deep. However, since the
// parse tree is built during parsing, it is unlikely that the parse tree is too deep.
// Otherwise, we need to convert this function to an iterative function.
pub(crate) fn build_tree<'a, T: TreeConstruct<'a>>(
    builder: &mut T,
    parse_tree: LRParseTree<'a>,
) -> Result<(), T::Error> {
    match parse_tree {
        LRParseTree::Terminal(token) => {
            builder.add_token(&token)?;
        }
        LRParseTree::NonTerminal(name, children) => {
            if let Some(children) = children {
                builder.open_non_terminal(name, Some(children.len()))?;
                for child in children {
                    build_tree::<T>(builder, child)?;
                }
            } else {
                builder.open_non_terminal(name, Some(0))?;
            }
            builder.close_non_terminal()?;
        }
    };
    Ok(())
}

// Convert a parse tree to a syntree tree.
// Since syntree must be built from the root, we use the LRParseTree during parsing and convert it
// to a syntree tree afterwards.
impl<'t> TryFrom<LRParseTree<'t>> for Tree<SynTree, SynTreeFlavor> {
    type Error = syntree::Error;

    fn try_from(parse_tree: LRParseTree<'t>) -> Result<Self, Self::Error> {
        let mut builder = Builder::new_with();
        build_tree::<TreeBuilder<SynTree>>(&mut builder, parse_tree)?;
        builder.build()
    }
}

impl<'t> From<&LRParseTree<'t>> for ParseTreeType<'t> {
    fn from(parse_tree: &LRParseTree<'t>) -> Self {
        match parse_tree {
            LRParseTree::Terminal(token) => ParseTreeType::T(token.clone()),
            LRParseTree::NonTerminal(name, _) => ParseTreeType::N(name),
        }
    }
}
