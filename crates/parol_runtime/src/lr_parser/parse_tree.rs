use std::{convert::TryFrom, fmt::Display};

use crate::{
    ParseTreeType, Token,
    parser::{
        parse_tree_type::{SynTree, TreeConstruct},
        parser_types::{SynTreeFlavor, TreeBuilder},
    },
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

/// Custom `Drop` implementation that iteratively drops children to avoid stack overflow
/// when dropping deeply nested `LRParseTree` structures (e.g. from long lists).
impl Drop for LRParseTree<'_> {
    fn drop(&mut self) {
        // Take the children out of self, replacing with None (which is trivially droppable)
        if let LRParseTree::NonTerminal(_, children) = self
            && let Some(children_vec) = children.take()
        {
            // Use an explicit stack to iteratively drop all nested children
            let mut drop_stack: Vec<Vec<LRParseTree<'_>>> = vec![children_vec];
            while let Some(mut current_children) = drop_stack.pop() {
                for child in current_children.iter_mut() {
                    // Take grandchildren out of each child NonTerminal, preventing
                    // recursive drop. The child itself becomes NonTerminal(_, None)
                    // which drops trivially.
                    if let LRParseTree::NonTerminal(_, grandchildren) = child
                        && let Some(grandchildren_vec) = grandchildren.take()
                        && !grandchildren_vec.is_empty()
                    {
                        drop_stack.push(grandchildren_vec);
                    }
                }
                // current_children now only contains Terminal and NonTerminal(_, None),
                // both of which drop trivially without recursion
            }
        }
    }
}

impl Display for LRParseTree<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LRParseTree::Terminal(token) => write!(f, "{token}"),
            LRParseTree::NonTerminal(name, Some(children)) => {
                write!(f, "{name}")?;
                if !children.is_empty() {
                    write!(f, "(")?;
                    for (i, child) in children.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{child}")?;
                    }
                    write!(f, ")")?;
                }
                Ok(())
            }
            LRParseTree::NonTerminal(name, None) => write!(f, "{name}"),
        }
    }
}

// Build a tree from a parse tree in a depth-first manner. This is an iterative function that
// traverses the parse tree and builds the syntree tree using an explicit work stack.
// This avoids stack overflow for deeply nested parse trees (e.g. from long lists).
pub(crate) fn build_tree<'a, T: TreeConstruct<'a>>(
    builder: &mut T,
    parse_tree: LRParseTree<'a>,
) -> Result<(), T::Error> {
    // We use a Vec of mutable trees as our work stack. We also need to track when to
    // close non-terminals, so we use a sentinel: NonTerminal("", None) means "close".
    // This avoids a separate enum while respecting the Drop constraint.
    let mut stack: Vec<LRParseTree<'a>> = Vec::new();
    // Start by processing the root
    stack.push(parse_tree);

    while let Some(mut tree) = stack.pop() {
        match &mut tree {
            LRParseTree::Terminal(token) => {
                builder.add_token(token)?;
            }
            LRParseTree::NonTerminal(name, children) => {
                if name.is_empty() && children.is_none() {
                    // This is our sentinel for "close non-terminal"
                    builder.close_non_terminal()?;
                } else if let Some(children_vec) = children.take() {
                    let len = children_vec.len();
                    builder.open_non_terminal(name, Some(len))?;
                    // Push close marker first (will be processed last)
                    stack.push(LRParseTree::NonTerminal("", None));
                    // Push children in reverse order so they are processed left-to-right
                    for child in children_vec.into_iter().rev() {
                        stack.push(child);
                    }
                } else {
                    builder.open_non_terminal(name, Some(0))?;
                    builder.close_non_terminal()?;
                }
            }
        }
    }
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
