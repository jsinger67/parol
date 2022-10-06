//!
//! This module provides a special graph type that is mostly used for the
//! detection of left recursions.
//!

use crate::{Cfg, Pr, Symbol, Terminal};
use petgraph::graph::{DiGraph, NodeIndex};
use std::borrow::Cow;
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
    Nt(Cow<'static, str>),
    /// LHS non-terminal
    L(Cow<'static, str>, usize),
    /// Non-terminal instance
    N(Cow<'static, str>, usize, usize),
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
            Self::L(n, p) => write!(f, "{}({})", n, p),
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
    /// Edges from LHS non-terminals to their productions
    /// carrying the number of the production headed to.
    LhsToPr(usize),
    /// Edges from non-terminal types to their productions
    /// carrying the number of the production headed to.
    NtToPr(usize),
    /// Edges from  production ends to the follower of the nt instance
    /// carrying the number of the production headed to.
    PrToFo(usize),
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

        // ---------------------------------------------------------------------
        // Add nodes to the graph
        // ---------------------------------------------------------------------

        // Production node indices
        let prod_node_indices =
            g.pr.iter()
                .map(|p| gg.0.add_node(NtNodeType::P(p.clone())))
                .collect::<Vec<NodeIndex<u32>>>();

        // Indices of non-terminal types
        let nt_node_indices = g
            .get_non_terminal_set()
            .iter()
            .fold(HashMap::new(), |mut acc, n| {
                acc.insert(
                    n.clone(),
                    gg.0.add_node(NtNodeType::Nt(Cow::Owned(n.clone()))),
                );
                acc
            });

        // Indices of non-terminal instances positions
        let nt_positions =
            g.get_non_terminal_positions()
                .iter()
                .fold(HashMap::new(), |mut acc, (p, n)| {
                    let node = if p.sy_index() == 0 {
                        NtNodeType::L(Cow::Owned(n.clone()), p.pr_index())
                    } else {
                        NtNodeType::N(Cow::Owned(n.clone()), p.pr_index(), p.sy_index())
                    };
                    let node_index = gg.0.add_node(node);
                    acc.insert((p.pr_index(), p.sy_index()), node_index);
                    acc
                });

        // Indices of terminals
        let te_positions =
            g.get_terminal_positions()
                .iter()
                .fold(HashMap::new(), |mut acc, (p, t)| {
                    let node_index = match t {
                        Symbol::T(trm) if matches!(trm, Terminal::Trm(..)) => {
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

        // ---------------------------------------------------------------------
        // Add edges to the graph
        // ---------------------------------------------------------------------
        g.pr.iter().enumerate().for_each(|(pi, p)| {
            let lhs_node_index = nt_positions.get(&(pi, 0)).unwrap();
            let pr_node_index = prod_node_indices[pi];
            let nt_type_index = nt_node_indices.get(p.get_n_str()).unwrap();

            // First add edges from LHS non-terminals to their productions
            gg.0.add_edge(*lhs_node_index, pr_node_index, NtEdgeType::LhsToPr(pi));
            // Add edges from non-terminal types to their productions
            gg.0.add_edge(*nt_type_index, pr_node_index, NtEdgeType::NtToPr(pi));

            // Add edges from LHS non-terminal type to their instance
            // gg.0.add_edge(*nt_type_index, *lhs_node_index, NtEdgeType::NtTypeInstance);

            // Second add edges within right-hand-sides of productions
            if !p.is_empty() {
                let mut from_node_index = pr_node_index;
                for (si, s) in p.get_r().iter().enumerate().filter(|(_, s)| !s.is_switch()) {
                    let to_node_index = match s {
                        Symbol::N(n, _, _) => {
                            // Add edge from from RHS non-terminal instances to their non-terminal
                            // types
                            let from_index = nt_positions.get(&(pi, si + 1)).unwrap();
                            let nt_type_index = nt_node_indices.get(n).unwrap();
                            gg.0.add_edge(*from_index, *nt_type_index, NtEdgeType::NtTypeInstance);

                            // Add edges from the ends of all productions belonging to the current
                            // non-terminal to the possible follower of this non-terminal
                            if p.len() > si + 1 {
                                // Find the follower node index
                                if let Some(follower_node_index) = p
                                    .get_r()
                                    .iter()
                                    .enumerate()
                                    .skip(si)
                                    .filter(|(_, s)| !s.is_switch())
                                    .next()
                                    .and_then(|(si, s)| {
                                        match s {
                                            Symbol::N(_, _, _) => nt_positions.get(&(pi, si + 1)),
                                            Symbol::T(_) => te_positions.get(&(pi, si + 1)),
                                            _ => panic!(
                                                "Unexpected symbol type on right-hand-side of production {}: '{}'",
                                                pi, s
                                            ),
                                        }
                                    }) {
                                        let matching_productions = g.matching_productions(n);
                                        for (pi, pr) in &matching_productions {
                                            // Find the index of the last symbol of the production
                                            if let Some(last_production_symbol_node) = pr
                                                .get_r()
                                                .iter()
                                                .enumerate()
                                                .filter(|(_, s)| !s.is_switch())
                                                .last()
                                                .and_then(|(si, s)| {
                                                    match s {
                                                        Symbol::N(_, _, _) => nt_positions.get(&(*pi, si + 1)),
                                                        Symbol::T(_) => te_positions.get(&(*pi, si + 1)),
                                                        _ => panic!(
                                                            "Unexpected symbol type on right-hand-side of production {}: '{}'",
                                                            pi, s
                                                        ),
                                                    }
                                                }) {
                                                    gg.0.add_edge(*last_production_symbol_node, *follower_node_index, NtEdgeType::PrToFo(*pi));
                                                }
                                        }
                                    }
                            }
                            Some(from_index)
                        }
                        Symbol::T(Terminal::Trm(..)) | Symbol::T(Terminal::End) => {
                            te_positions.get(&(pi, si + 1))
                        }
                        _ => panic!(
                            "Unexpected symbol type on right-hand-side of production {}: '{}'",
                            pi, s
                        ),
                    };
                    if let Some(to_node_index) = to_node_index {
                        gg.0.add_edge(from_node_index, *to_node_index, NtEdgeType::PrRhs(pi));
                        from_node_index = *to_node_index;
                    }
                }
            }
        });

        gg
    }
}
