use std::collections::{BTreeMap, BTreeSet};

use log::trace;
use petgraph::{algo::all_simple_paths, prelude::DiGraph, visit::IntoNodeReferences};

use crate::{TerminalIndex, Trans, lexer::EOI};

use super::{CompiledProductionIndex, INVALID_PROD};

#[derive(Debug, Clone, PartialEq)]
pub enum EditOp {
    Keep,
    Insert,
    Delete,
    Replace,
}

pub(crate) struct Recovery;

impl Recovery {
    /// Calculates the Levenshtein distance between two token sequences.
    ///
    /// The Levenshtein distance is the minimum number of single-character
    /// edits (insertions, deletions, or substitutions) required to change
    /// one sequence into the other.
    ///
    /// # Arguments
    ///
    /// * `act` - The actual token sequence.
    /// * `exp` - The expected token sequence.
    ///
    /// # Returns
    ///
    /// A tuple containing the Levenshtein distance and the sequence of
    /// edit operations required to transform the actual sequence into the
    /// expected sequence.
    pub(crate) fn levenshtein_distance(
        act: &[TerminalIndex],
        exp: &[TerminalIndex],
    ) -> (usize, Vec<EditOp>) {
        if act.is_empty() && exp.is_empty() {
            return (0, Vec::new());
        }

        if act.is_empty() {
            return (exp.len(), vec![EditOp::Insert; exp.len()]);
        }

        if exp.is_empty() {
            return (act.len(), vec![EditOp::Delete; act.len()]);
        }

        let mut d = vec![vec![0; exp.len() + 1]; act.len() + 1];
        let mut ops = vec![vec![EditOp::Keep; exp.len() + 1]; act.len() + 1];

        for i in 0..=act.len() {
            d[i][0] = i;
            ops[i][0] = EditOp::Delete;
        }

        for j in 0..=exp.len() {
            d[0][j] = j;
            ops[0][j] = EditOp::Insert;
        }

        for i in 1..=act.len() {
            for j in 1..=exp.len() {
                if act[i - 1] == exp[j - 1] {
                    d[i][j] = d[i - 1][j - 1];
                    ops[i][j] = EditOp::Keep;
                } else {
                    let mut min = d[i - 1][j] + 1;
                    let mut op = EditOp::Delete;

                    if d[i][j - 1] + 1 < min {
                        min = d[i][j - 1] + 1;
                        op = EditOp::Insert;
                    }

                    if d[i - 1][j - 1] + 1 < min {
                        min = d[i - 1][j - 1] + 1;
                        op = EditOp::Replace;
                    }

                    d[i][j] = min;
                    ops[i][j] = op;
                }
            }
        }

        // Backtrack to get operations
        let mut result_ops = Vec::new();
        let mut i = act.len();
        let mut j = exp.len();

        while i > 0 || j > 0 {
            let op = &ops[i][j];
            match op {
                EditOp::Keep => {
                    result_ops.push(EditOp::Keep);
                    i -= 1;
                    j -= 1;
                }
                EditOp::Insert => {
                    result_ops.push(EditOp::Insert);
                    j -= 1;
                }
                EditOp::Delete => {
                    result_ops.push(EditOp::Delete);
                    i -= 1;
                }
                EditOp::Replace => {
                    result_ops.push(EditOp::Replace);
                    i -= 1;
                    j -= 1;
                }
            }
        }

        result_ops.reverse();
        (d[act.len()][exp.len()], result_ops)
    }

    // This function returns the first terminal string that matches the current scanned token types
    // with a maximum range.
    pub(crate) fn minimal_token_difference(
        scanned_token_types: &[TerminalIndex],
        possible_terminal_strings: &mut BTreeSet<Vec<TerminalIndex>>,
    ) -> Option<Vec<TerminalIndex>> {
        trace!("scanned_token_types: {scanned_token_types:?}");
        trace!("possible_terminal_strings: {possible_terminal_strings:?}");
        let mut min_distance = usize::MAX;
        let mut min_distance_string = None;
        for terminal_string in possible_terminal_strings.iter() {
            let (dist, _) = Self::levenshtein_distance(scanned_token_types, terminal_string);
            if dist < min_distance {
                min_distance = dist;
                min_distance_string = Some(terminal_string);
            } else if dist == min_distance
                && let Some(min_s) = min_distance_string
                && min_s.contains(&EOI)
                && !terminal_string.contains(&EOI)
            {
                min_distance_string = Some(terminal_string);
            }
        }
        min_distance_string.map(|i| possible_terminal_strings.get(i).unwrap().clone())
    }

    // This function uses the lookahead DFA (transitions and production number in state 0) of a
    // certain non-terminal to recalculate the possible token strings that lead to an end state.
    // Although this is an expensive calculation it is only done in case of error recovery.
    // For the sake of simplicity I use `petgraph` here for now.
    pub(crate) fn restore_terminal_strings(
        transitions: &[Trans],
        prod0: CompiledProductionIndex,
    ) -> BTreeSet<Vec<TerminalIndex>> {
        let mut result = BTreeSet::new();
        let mut nodes = BTreeSet::<(usize, bool)>::new();
        let root_key = (0, prod0 != INVALID_PROD);
        nodes.insert(root_key);
        for t in transitions {
            nodes.insert((t.2, t.3 != INVALID_PROD));
        }
        let mut node_indices = BTreeMap::new();
        let mut graph = DiGraph::<(usize, bool), TerminalIndex>::new();
        for n in &nodes {
            let idx = graph.add_node(*n);
            node_indices.insert(n, idx);
        }
        for t in transitions {
            let k0 = nodes
                .get(&(t.0, true))
                .or_else(|| nodes.get(&(t.0, false)))
                .unwrap();
            let k1 = nodes
                .get(&(t.2, true))
                .or_else(|| nodes.get(&(t.2, false)))
                .unwrap();
            graph.add_edge(
                *node_indices.get(&k0).unwrap(),
                *node_indices.get(&k1).unwrap(),
                t.1,
            );
        }

        let root_node_index = *node_indices.get(&root_key).unwrap();
        for end_node in graph.node_references().filter(|n| n.1.1) {
            for path in all_simple_paths::<Vec<_>, _, std::hash::RandomState>(
                &graph,
                root_node_index,
                end_node.0,
                0,
                None,
            ) {
                let mut terminal_string = Vec::new();
                let mut prev_node_index = root_node_index;
                for node_index in path.iter().skip(1) {
                    let edge_index = graph.find_edge(prev_node_index, *node_index).unwrap();
                    let edge = graph[edge_index];
                    terminal_string.push(edge);
                    prev_node_index = *node_index;
                }
                result.insert(terminal_string);
            }
        }

        result
    }
}

// To see trace output during tests use this call
// cargo test -p parol_runtime --tests recovery -- --nocapture
#[cfg(test)]
mod test {
    use super::*;

    // Type alias to simplify the complex test data type
    type TestCase = (
        (&'static [TerminalIndex], &'static [TerminalIndex]),
        usize,
        Vec<EditOp>,
    );

    #[test]
    fn test_levenshtein_distance() {
        let _ = env_logger::builder().is_test(true).try_init();
        let test_data: &[TestCase] = &[
            ((&[1, 2], &[0, 2]), 1, vec![EditOp::Replace, EditOp::Keep]),
            (
                (&[7, 8, 9, 10], &[8, 8, 9, 10]),
                1,
                vec![EditOp::Delete, EditOp::Keep, EditOp::Keep, EditOp::Keep],
            ),
            (
                (&[7, 8, 9, 10], &[8, 8, 9]),
                2,
                vec![EditOp::Delete, EditOp::Delete, EditOp::Keep, EditOp::Insert],
            ),
            (
                (&[8, 2, 8, 2, 8], &[1, 8, 2, 8]),
                2,
                vec![
                    EditOp::Replace,
                    EditOp::Keep,
                    EditOp::Keep,
                    EditOp::Keep,
                    EditOp::Keep,
                ],
            ),
        ];

        for (i, d) in test_data.iter().enumerate() {
            let (dist, _ops) = Recovery::levenshtein_distance(d.0.0, d.0.1);
            assert_eq!(
                d.1, dist,
                "test case {i} distance failed for input {:?}",
                d.0
            );
        }
    }

    #[test]
    fn test_restore_terminal_strings() {
        let _ = env_logger::builder().is_test(true).try_init();
        // k(2) example taken from examples\basic_interpreter\src\basic_parser.rs
        /* 4 - "BasicList" */
        let transitions = &[
            Trans(0, 0, 3, 2),
            Trans(0, 9, 1, -1),
            Trans(1, 0, 3, 2),
            Trans(1, 6, 2, 1),
        ];
        let terminal_strings = Recovery::restore_terminal_strings(transitions, -1);
        trace!("Terminal strings: {terminal_strings:?}");
        assert_eq!(3, terminal_strings.len());
        assert!(terminal_strings.contains(&vec![0]));
        assert!(terminal_strings.contains(&vec![9, 0]));
        assert!(terminal_strings.contains(&vec![9, 6]));
    }
}
