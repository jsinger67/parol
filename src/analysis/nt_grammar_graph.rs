//!
//! This module provides a special graph type that is mostly used for the
//! detection of left recursions.
//!

use crate::{Cfg, Pr, Symbol, Terminal};
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;
use std::fmt::{Debug, Display, Error, Formatter};
use std::hash::Hash;

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
///
/// Node type of the grammar graph
///
#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum NtNodeType {
    /// Non-terminal type
    Nt(String),
    /// Non-terminal instance
    N(String, usize, usize),
    /// Production
    P(Pr),
    /// Terminal instance
    T(Terminal, usize, usize),
    /// End of input, pseudo-terminal
    E(usize, usize),
}

impl Default for NtNodeType {
    fn default() -> Self {
        Self::E(usize::MAX, usize::MAX)
    }
}

impl Display for NtNodeType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Self::Nt(n) => write!(f, "{}", n),
            Self::N(n, p, s) => write!(f, "{}({},{})", n, p, s),
            Self::P(p) => write!(f, "P({})", p),
            Self::T(t, _, _) => write!(f, "'{}'", t),
            Self::E(_, _) => write!(f, "$"),
        }
    }
}

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
///
///  Edge type of the grammar graph
///
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum NtEdgeType {
    /// Edges from non-terminals to their productions
    /// carrying the number of the production headed to.
    NtToPr(usize),
    /// Edges within the right-hand-side of productions
    /// carrying the number of the associated production
    PrRhs(usize),
    /// Edges between non-terminal instances to their non-terminal types
    NtTypeInstance,
}

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
/// Grammar graph over non-terminals and productions of a Cfg
#[derive(Debug)]
pub struct NtGrammarGraph(pub DiGraph<NtNodeType, NtEdgeType>);

impl NtGrammarGraph {
    /// Creates a new graph item
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for NtGrammarGraph {
    fn default() -> Self {
        Self(DiGraph::<NtNodeType, NtEdgeType>::new())
    }
}

impl From<&Cfg> for NtGrammarGraph {
    fn from(g: &Cfg) -> Self {
        let mut gg = Self::new();

        let prod_node_indices =
            g.pr.iter()
                .map(|p| gg.0.add_node(NtNodeType::P(p.clone())))
                .collect::<Vec<NodeIndex<u32>>>();

        let non_terminals = g.get_non_terminal_ordering();
        let nt_node_indices = non_terminals
            .iter()
            .fold(HashMap::new(), |mut acc, (n, _)| {
                acc.insert(n.clone(), gg.0.add_node(NtNodeType::Nt(n.clone())));
                acc
            });
        let nt_positions = g.get_non_terminal_positions();
        let nt_positions = nt_positions.iter().fold(HashMap::new(), |mut acc, (p, n)| {
            let node_index =
                gg.0.add_node(NtNodeType::N(n.clone(), p.pr_index(), p.sy_index()));
            acc.insert((p.pr_index(), p.sy_index()), node_index);
            acc
        });
        let te_positions = g.get_terminal_positions();
        let te_positions = te_positions.iter().fold(HashMap::new(), |mut acc, (p, t)| {
            let node_index = match t {
                Symbol::T(trm) if matches!(trm, Terminal::Trm(_, _)) => {
                    gg.0.add_node(NtNodeType::T(trm.clone(), p.pr_index(), p.sy_index()))
                }
                Symbol::T(Terminal::End) => {
                    gg.0.add_node(NtNodeType::E(p.pr_index(), p.sy_index()))
                }
                _ => panic!("Invalid symbol type on RHS of production"),
            };
            acc.insert((p.pr_index(), p.sy_index()), node_index);
            acc
        });

        // Add edges to the graph
        g.pr.iter().enumerate().for_each(|(pi, p)| {
            let lhs_node_index = nt_positions.get(&(pi, 0)).unwrap();
            let pr_node_index = prod_node_indices[pi];
            let nt_type_index = nt_node_indices.get(p.get_n_str()).unwrap();

            // First add edges from LHS non-terminals to their productions
            gg.0.add_edge(*lhs_node_index, pr_node_index, NtEdgeType::NtToPr(pi));
            // Add edges from LHS non-terminal type to their instance
            gg.0.add_edge(*nt_type_index, *lhs_node_index, NtEdgeType::NtTypeInstance);

            // Second add edges within right-hand-sides of productions
            if !p.is_empty() {
                let mut from_node_index = pr_node_index;
                for (si, s) in p.get_r().iter().enumerate().filter(|(_, s)| !s.is_switch()) {
                    let to_node_index = match s {
                        Symbol::N(n) => {
                            // Add edge from from RHS non-terminal instances to their non-terminal types
                            let from_index = nt_positions.get(&(pi, si + 1)).unwrap();
                            let nt_type_index = nt_node_indices.get(n).unwrap();
                            gg.0.add_edge(*from_index, *nt_type_index, NtEdgeType::NtTypeInstance);
                            from_index
                        }
                        Symbol::T(Terminal::Trm(_, _)) | Symbol::T(Terminal::End) => {
                            te_positions.get(&(pi, si + 1)).unwrap()
                        }
                        _ => panic!(
                            "Unexpected symbol type on right-hand-side of production {}: '{}'",
                            pi, s
                        ),
                    };
                    gg.0.add_edge(from_node_index, *to_node_index, NtEdgeType::PrRhs(pi));
                    from_node_index = *to_node_index;
                }
            }
        });

        gg
    }
}
