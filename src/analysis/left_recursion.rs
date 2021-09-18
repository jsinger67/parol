use crate::{Cfg, NtGrammarGraph, NtNodeType};
use petgraph::algo::all_simple_paths;
use petgraph::graph::NodeIndex;
use std::collections::HashSet;

pub fn detect_left_recursions(grammar: &Cfg) -> HashSet<Vec<NtNodeType>> {
    let nt_graph: NtGrammarGraph = grammar.into();
    let graph = &nt_graph.0;
    let nullables = grammar.calculate_nullable_non_terminals();

    let mut recursion_paths: HashSet<Vec<NtNodeType>> = HashSet::new();

    let mut already_found_recursion_paths: HashSet<Vec<NodeIndex>> = HashSet::new();

    let mut find_recursion = |node_index| -> HashSet<Vec<NtNodeType>> {
        let nr = if let NtNodeType::Nt(n) = &graph[node_index] {
            n
        } else {
            panic!("Should not happen!")
        };
        all_simple_paths(graph, node_index, node_index, 1, None)
            .filter_map(|p: Vec<NodeIndex>| {
                let produces_no_terminals = p.iter().all(|ni| {
                    let n = &graph[*ni];
                    match n {
                        NtNodeType::P(_) => true,
                        NtNodeType::Nt(n) | NtNodeType::N(n, _, _) => {
                            nr == n || nullables.contains(n)
                        }
                        _ => false,
                    }
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
