use crate::{Cfg, NtGrammarGraph, NtNodeType};
use petgraph::algo::all_simple_paths;
use petgraph::graph::NodeIndex;
use std::collections::{BTreeMap, HashSet};

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
///
/// Detects left recursions.
/// The result is a collection of vectors of [NtNodeType].
/// The vectors of nodes are cycles in the [NtGrammarGraph].
///
pub fn detect_left_recursions(grammar: &Cfg) -> HashSet<Vec<NtNodeType>> {
    let nt_graph: NtGrammarGraph = grammar.into();
    let graph = &nt_graph.0;

    let mut recursion_paths: HashSet<Vec<NtNodeType>> = HashSet::new();

    let mut already_found_recursion_paths: HashSet<Vec<NodeIndex>> = HashSet::new();

    let mut find_recursion = |node_index| -> HashSet<Vec<NtNodeType>> {
        all_simple_paths(graph, node_index, node_index, 1, None)
            .filter_map(|p: Vec<NodeIndex>| {
                let produces_no_terminals = p.iter().all(|ni| {
                    let n = &graph[*ni];
                    !matches!(n, NtNodeType::T(..))
                });

                if produces_no_terminals && matches!(graph[p[2]], NtNodeType::P(_)) {
                    let mut node_set = p.to_vec();
                    node_set.sort();
                    node_set.dedup();
                    if already_found_recursion_paths.contains(&node_set) {
                        None
                    } else {
                        already_found_recursion_paths.insert(node_set);
                        Some(
                            p.iter()
                                .map(|i| graph[*i].clone())
                                .collect::<Vec<NtNodeType>>(),
                        )
                    }
                } else {
                    None
                }
            })
            .collect::<HashSet<Vec<NtNodeType>>>()
    };

    graph
        .node_indices()
        .filter(|n| matches!(graph[*n], NtNodeType::Nt(_)))
        .for_each(|n| {
            find_recursion(n)
                .drain()
                .filter(|p| !p.is_empty())
                .for_each(|p| {
                    let _ = recursion_paths.insert(p);
                });
        });

    recursion_paths
}

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
///
/// Detects left recursions.
/// The result is a collection of vectors of [NtNodeType].
/// The vectors of nodes are cycles in the [NtGrammarGraph].
///
pub fn detect_left_recursive_non_terminals(cfg: &Cfg) -> Vec<String> {
    let nullables = cfg.calculate_nullable_non_terminals();
    let mut can_start_with = cfg
        .get_non_terminal_set()
        .iter()
        .map(|nt| (nt.clone(), HashSet::<String>::new()))
        .collect::<BTreeMap<String, HashSet<String>>>();

    // For all non-terminals A, B calculate the relation 'A can-start-with B'
    let mut changed = true;
    while changed {
        changed = false;
        for p in &cfg.pr {
            let lhs = p.0.get_n_ref().unwrap();
            let lhs_entry = can_start_with.get_mut(lhs).unwrap();
            for s in &p.1 {
                match s {
                    crate::Symbol::N(ref n, _, _) => {
                        changed |= lhs_entry.insert(n.clone());
                        if !nullables.contains(n) {
                            break;
                        }
                    }
                    crate::Symbol::T(_) => break,
                    crate::Symbol::S(_) | crate::Symbol::Push(_) | crate::Symbol::Pop => (),
                }
            }
        }
    }

    // Calculate transitive closure of the relation 'A can-start-with B'
    // Ex.: A->B, B->C => A->{B, C}
    changed = true;
    while changed {
        changed = false;
        for nt in can_start_with
            .keys()
            .cloned()
            .collect::<Vec<String>>()
            .iter()
        {
            let v = can_start_with
                .get(nt)
                .map_or(HashSet::default(), |s| s.clone());
            for e in &v {
                let t = can_start_with
                    .get(e)
                    .map_or(HashSet::default(), |s| s.clone());
                can_start_with
                    .get_mut(nt)
                    .map(|f| f.extend(t.iter().cloned()));
            }
            changed |= v.len() < can_start_with.get(nt).map_or(0, |v| v.len());
        }
    }

    // Return all non-terminals that have themselves in the 'can-start-with relation'
    can_start_with.iter().fold(Vec::new(), |mut acc, (k, v)| {
        if v.contains(k) {
            acc.push(k.to_owned());
        }
        acc
    })
}

#[cfg(test)]
mod test {
    use crate::{detect_left_recursions, obtain_grammar_config_from_string};

    use super::detect_left_recursive_non_terminals;

    #[derive(Debug)]
    struct TestData {
        input: &'static str,
        recursive_non_terminals: &'static [&'static str],
        recursive_nodes: &'static str,
    }

    const TESTS: &[TestData] = &[
        TestData {
            input: r#"%start A %% A: B "r"; B: C "d"; C: A "t";"#,
            recursive_non_terminals: &["A", "B", "C"],
            recursive_nodes: r#"{[Nt("A"), N("A", 0, 0), P(Pr(N("A", None, None), [N("B", None, None), T(Trm("r", [0], None, None))], None)), N("B", 0, 1), Nt("B"), N("B", 1, 0), P(Pr(N("B", None, None), [N("C", None, None), T(Trm("d", [0], None, None))], None)), N("C", 1, 1), Nt("C"), N("C", 2, 0), P(Pr(N("C", None, None), [N("A", None, None), T(Trm("t", [0], None, None))], None)), N("A", 2, 1), Nt("A")]}"#,
        },
        TestData {
            input: r#"%start S %% S: S "a"; S: ;"#,
            recursive_non_terminals: &["S"],
            recursive_nodes: r#"{[Nt("S"), N("S", 0, 0), P(Pr(N("S", None, None), [N("S", None, None), T(Trm("a", [0], None, None))], None)), N("S", 0, 1), Nt("S")]}"#,
        },
        TestData {
            input: r#"%start A %% A: A B "d"; A: A "a"; A: "a"; B: B "e"; B: "b";"#,
            recursive_non_terminals: &["A", "B"],
            recursive_nodes: r#""#,
        },
        TestData {
            input: r#"%start E %% E: E "+" E; E: E "*" E; E: "a";"#,
            recursive_non_terminals: &["E"],
            recursive_nodes: r#""#,
        },
        TestData {
            input: r#"%start E %% E: E "+" T; E: T; T: T "*" F; T: F; F: "id";"#,
            recursive_non_terminals: &["E", "T"],
            recursive_nodes: r#""#,
        },
        TestData {
            input: r#"%start S %% S: "(" L ")"; S: "a"; L: L "," S; L: S;"#,
            recursive_non_terminals: &["L"],
            recursive_nodes: r#""#,
        },
        TestData {
            input: r#"%start S %% S: S "0" S "1" S; S: "0" "1";"#,
            recursive_nodes: r#""#,
            recursive_non_terminals: &["S"],
        },
        TestData {
            input: r#"%start S %% S: A; A: A "d"; A: A "e"; A: "a" B; A: "a" "c"; B: "b" B "c"; B: "f";"#,
            recursive_non_terminals: &["A"],
            recursive_nodes: r#""#,
        },
        TestData {
            input: r#"%start A %% A: A A "a"; A: "b";"#,
            recursive_non_terminals: &["A"],
            recursive_nodes: r#""#,
        },
        TestData {
            input: r#"%start A %% A: B "a"; A: A "a"; A: "c"; B: B "b"; B: A "b"; B: "d";"#,
            recursive_non_terminals: &["A", "B"],
            recursive_nodes: r#""#,
        },
        TestData {
            input: r#"%start X %% X: X S "b"; X: S "a"; X: "b"; S: S "b"; S: X "a"; S: "a";"#,
            recursive_non_terminals: &["S", "X"],
            recursive_nodes: r#""#,
        },
        TestData {
            input: r#"%start S %% S: A "a"; S: "b"; A: A "c"; A: S "d"; A: ;"#,
            recursive_non_terminals: &["A", "S"],
            recursive_nodes: r#""#,
        },
    ];

    #[test]
    #[ignore = "incomplete"]
    fn check_detect_left_recursions() {
        for (i, test) in TESTS.iter().enumerate() {
            let grammar_config = obtain_grammar_config_from_string(test.input, false).unwrap();
            let recursions = detect_left_recursions(&grammar_config.cfg);
            // dbg!(&recursions);
            assert_eq!(
                test.recursive_nodes,
                format!("{recursions:?}"),
                "Error at test #{i}"
            );
        }
    }

    #[test]
    fn check_detect_left_recursive_non_terminals() {
        for (i, test) in TESTS.iter().enumerate() {
            let grammar_config = obtain_grammar_config_from_string(test.input, false).unwrap();
            let recursions = detect_left_recursive_non_terminals(&grammar_config.cfg);
            assert_eq!(
                test.recursive_non_terminals, recursions,
                "Error at test #{i}"
            );
        }
    }
}
