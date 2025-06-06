use std::{
    cmp::max,
    collections::{BTreeMap, BTreeSet},
    ops::Range,
};

use log::trace;
use petgraph::{algo::all_simple_paths, prelude::DiGraph, visit::IntoNodeReferences};

use crate::{TerminalIndex, Trans};

use super::{CompiledProductionIndex, INVALID_PROD};

pub(crate) struct Recovery;

impl Recovery {
    ///
    /// Calculates valid match ranges for actually scanned terminals and expected terminals.
    /// The Strategy is to match a (sub) range of the scanned terminals completely with a sub range
    /// of the expected terminals. Not matching prefixes can exist and are corrected later by the
    /// parser during recovery.
    /// To be successful the sub match must either reach until the end of the input tokens or
    /// until the end of the expected tokens.
    /// This is checked in the inner loop by evaluating the result of the inner `try_expand`.
    ///
    pub(crate) fn calculate_match_ranges(
        act: &[TerminalIndex],
        exp: &[TerminalIndex],
    ) -> Option<(Range<usize>, Range<usize>)> {
        if act.is_empty() || exp.is_empty() {
            return None;
        }

        // Create and fill the match matrix.
        // The matrix should be very small, i.e. significantly smaller than 10x10.
        let mut m: Vec<Vec<bool>> = vec![vec![false; exp.len()]; act.len()];
        for (ia, a) in act.iter().enumerate() {
            for (ie, e) in exp.iter().enumerate() {
                if *a == *e {
                    m[ia][ie] = true;
                }
            }
        }

        // Tries to follow a diagonal line in the match matrix.
        let try_expand = |mut ia: usize, mut ie: usize| -> Option<(usize, usize)> {
            let mut result = None;
            while m[ia][ie] {
                result = Some((ia, ie));
                ia += 1;
                ie += 1;
                if ia >= act.len() || ie >= exp.len() {
                    break;
                }
            }
            result
        };

        // trace!("exp: {exp:?}");
        // for (i, c) in m.iter().enumerate() {
        //     trace!("{}: {c:?}", act[i]);
        // }

        let mut result = None;
        let mut a_start = None;
        let mut a_end = None;
        let mut e_start = None;
        let mut e_end = None;
        let ia_max = act.len() - 1;
        let ie_max = exp.len() - 1;

        'OUTER: for (ia, a) in m.iter().enumerate() {
            for (ie, eq) in a.iter().enumerate() {
                if a_start.is_none() && *eq {
                    a_start = Some(ia);
                    e_start = Some(ie);
                    if let Some((ia_end, ie_end)) = try_expand(ia, ie) {
                        if ie_end == ie_max || ia_end == ia_max {
                            a_end = Some(ia_end);
                            e_end = Some(ie_end);
                            break 'OUTER;
                        } else {
                            a_start = None;
                            e_start = None;
                            a_end = None;
                            e_end = None;
                        }
                    } else {
                        a_start = None;
                        e_start = None;
                        a_end = None;
                        e_end = None;
                    }
                }
            }
        }
        trace!("{:?}", (a_start, a_end, e_start, e_end));
        if let (Some(a0), Some(a1), Some(e0), Some(e1)) = (a_start, a_end, e_start, e_end) {
            // Ranges are excluding, thus we increment the upper limits.
            result = Some((a0..a1 + 1, e0..e1 + 1));
        }
        trace!("calculate_match_ranges result: {result:?}");
        result
    }

    // This function returns the first terminal string that matches the current scanned token types
    // with a maximum range.
    pub(crate) fn minimal_token_difference(
        scanned_token_types: &[TerminalIndex],
        possible_terminal_strings: &mut BTreeSet<Vec<TerminalIndex>>,
    ) -> Option<Vec<TerminalIndex>> {
        trace!("scanned_token_types: {scanned_token_types:?}");
        trace!("possible_terminal_strings: {possible_terminal_strings:?}");
        let mut max_match_length = 0;
        let mut max_match_length_string = None;
        for terminal_string in possible_terminal_strings.iter() {
            if let Some((r, _)) = Self::calculate_match_ranges(scanned_token_types, terminal_string)
            {
                let match_length = max(0, r.end - r.start);
                if match_length > max_match_length {
                    max_match_length = match_length;
                    max_match_length_string = Some(terminal_string);
                }
            }
        }
        max_match_length_string.map(|i| possible_terminal_strings.get(i).unwrap().clone())
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
            for path in all_simple_paths::<Vec<_>, _>(&graph, root_node_index, end_node.0, 0, None)
            {
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
    use quickcheck_macros::quickcheck;

    use super::*;
    use std::cmp::max;

    type TestData = &'static [(
        (&'static [TerminalIndex], &'static [TerminalIndex]),
        Option<(Range<usize>, Range<usize>)>,
    )];
    #[test]
    fn test_calculate_match_ranges() {
        let _ = env_logger::builder().is_test(true).try_init();
        let test_data: TestData = &[
            ((&[1, 2], &[0, 2]), Some((1..2, 1..2))),
            ((&[7, 8, 9, 10], &[8, 8, 9, 10]), Some((1..4, 1..4))),
            ((&[7, 8, 9, 10], &[8, 8, 9]), Some((1..3, 1..3))),
            ((&[8, 2, 8, 2, 8], &[1, 8, 2, 8]), Some((0..3, 1..4))),
            ((&[7, 8, 9, 10], &[8, 8]), Some((1..2, 1..2))),
            ((&[7, 8, 9, 10], &[8]), Some((1..2, 0..1))),
            ((&[7, 8, 9, 10], &[]), None),
            ((&[6, 7, 8, 9], &[8, 8, 9, 10]), Some((2..4, 1..3))),
            ((&[6, 7, 8, 9, 11], &[8, 8, 9, 10]), None),
            ((&[6, 7, 8, 9, 10], &[8, 8, 9, 10, 11]), Some((2..5, 1..4))),
        ];
        for (i, d) in test_data.iter().enumerate() {
            assert_eq!(
                d.1,
                Recovery::calculate_match_ranges(d.0.0, d.0.1),
                "test case {i} failed for input {:?}",
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

    #[quickcheck]
    fn length_of_match_ranges_are_equal(t1: Vec<TerminalIndex>, t2: Vec<TerminalIndex>) -> bool {
        if let Some((r1, r2)) = Recovery::calculate_match_ranges(&t1, &t2) {
            let l1 = max(0, r1.end - r1.start);
            let l2 = max(0, r2.end - r2.start);
            l1 == l2
        } else {
            true
        }
    }
}
