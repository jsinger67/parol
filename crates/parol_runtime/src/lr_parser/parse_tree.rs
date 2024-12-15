use std::{convert::TryFrom, fmt::Display};

use crate::{parser::parse_tree_type::SynTree, ParseTreeType, Token};
use syntree::{Builder, Tree};

/// Parse tree representation.
/// The lifetime `'t` is the lifetime of the input text.
#[derive(Debug, Clone)]
pub enum LRParseTree<'t> {
    Terminal(Token<'t>),
    NonTerminal(&'static str, Option<Vec<LRParseTree<'t>>>),
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
fn build_tree<'t>(
    builder: &mut Builder<SynTree, u32, usize>,
    parse_tree: LRParseTree<'t>,
) -> Result<(), syntree::Error> {
    match parse_tree {
        LRParseTree::Terminal(token) => {
            let len = token.location.len();
            builder.token(SynTree::Terminal((&token).into()), len)?;
        }
        LRParseTree::NonTerminal(name, children) => {
            builder.open(SynTree::NonTerminal(name))?;
            if let Some(children) = children {
                for child in children {
                    build_tree(builder, child)?;
                }
            }
            builder.close()?;
        }
    };
    Ok(())
}

// Convert a parse tree to a syntree tree.
// Since syntree must be built from the root, we use the LRParseTree during parsing and convert it
// to a syntree tree afterwards.
impl<'t> TryFrom<LRParseTree<'t>> for Tree<SynTree, u32, usize> {
    type Error = syntree::Error;

    fn try_from(parse_tree: LRParseTree<'t>) -> Result<Self, Self::Error> {
        let mut builder = Builder::new();
        build_tree(&mut builder, parse_tree)?;
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
