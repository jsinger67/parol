use std::collections::BTreeMap;

use super::lookahead_dfa::{CompiledProductionIndex, StateIndex, INVALID_PROD};
use super::LookaheadDFA;

// use crate::group_by;

pub(crate) type TerminalIndex = usize;

mod adjacency_list {
    use std::collections::{HashMap, HashSet};
    use crate::analysis::lookahead_dfa::CompiledProductionIndex;
    use crate::analysis::compiled_la_dfa::TerminalIndex;

    use super::CompiledDFA;

    type NodeId = usize;

    #[derive(Debug, Eq, PartialEq, Hash)]
    struct AdjacentNode {
        id: usize,
        term: TerminalIndex,
    }

    #[derive(Debug)]
    struct AdjacencyListEntry {
        neighbors: HashSet<AdjacentNode>,
    }

    impl AdjacencyListEntry {
        fn new() -> Self {
            AdjacencyListEntry {
                neighbors: HashSet::new(),
            }
        }

        fn add_neighbor(&mut self, neighbor: AdjacentNode) {
            self.neighbors.insert(neighbor);
        }
    }

    #[derive(Debug)]
    struct AdjacencyList {
        list: HashMap<NodeId, AdjacencyListEntry>,
        productions: HashMap<NodeId, CompiledProductionIndex>,
    }

    impl From<&CompiledDFA> for AdjacencyList {
        fn from(value: &CompiledDFA) -> Self {
            let mut list = HashMap::new();
            let mut productions = HashMap::new();
            list.insert(0, AdjacencyListEntry::new());
            productions.insert(0, value.prod0);

            for t in &value.transitions {
                // We use the to-state and the resulting production number
                list.insert(t.2, AdjacencyListEntry::new());
                productions.insert(t.2, t.3);
            }

            for t in &value.transitions {
                // We add the neighbors (to-state) to each node (from-state)
                if let Some(f) = list.get_mut(&t.0) {
                    f.add_neighbor(AdjacentNode { id: t.2, term: t.1 });
                }
            }

            AdjacencyList { list, productions }
        }
    }
}

/// A Four-Tuple type consisting of
/// * Start state index (from-state)
/// * Terminal index occurred in start state that triggers the transition
/// * Result state index (to-state)
/// * Possible ProductionIndex
type CompiledTransition = (
    StateIndex,
    TerminalIndex,
    StateIndex,
    CompiledProductionIndex,
);

///
/// Internal data structure to represent a CompiledDFA which in turn is used to
/// generate the parsers source code
///
#[derive(Debug, Clone)]
pub(crate) struct CompiledDFA {
    /// Contains the production number in state 0, i.e. the state that the automaton is initially in
    /// without applying any transitions
    pub prod0: CompiledProductionIndex,
    /// Tuples from-state->terminal->to-state->Possible ProductionIndex
    pub transitions: Vec<CompiledTransition>,
    pub k: usize,
}

impl CompiledDFA {
    pub fn from_lookahead_dfa(dfa: &LookaheadDFA) -> CompiledDFA {
        let states = dfa
            .states
            .iter()
            .filter_map(|s| {
                if s.is_accepting() {
                    Some((s.id, s.prod_num))
                } else {
                    None
                }
            })
            .collect::<BTreeMap<StateIndex, CompiledProductionIndex>>();

        let prod0 = *states.get(&0).unwrap_or(&INVALID_PROD);

        let transitions =
            dfa.transitions
                .iter()
                .fold(Vec::<CompiledTransition>::new(), |mut acc, (ci, t)| {
                    let mut transitions =
                        t.iter()
                            .fold(Vec::<CompiledTransition>::new(), |mut acc, (trm, ns)| {
                                acc.push((
                                    *ci as StateIndex,
                                    *trm,
                                    *ns as StateIndex,
                                    *states.get(ns).unwrap_or(&INVALID_PROD),
                                ));
                                acc
                            });
                    transitions.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
                    acc.append(&mut transitions);
                    acc
                });

        let k = dfa.k;

        Self::optimize(Self {
            prod0,
            transitions,
            k,
        })
    }

    fn _minimize(self) -> CompiledDFA {
        let Self {
            prod0,
            mut transitions,
            k,
        } = self;

        fn minimize_step(
            _combined_states: Vec<usize>,
            _transitions: &[CompiledTransition],
        ) -> (Vec<usize>, Vec<CompiledTransition>) {
            //
            (vec![], vec![])
        }

        let _transitions_to_final_states: Vec<(usize, usize, usize, i32)> = transitions
            .iter()
            .filter(|t| t.3 != INVALID_PROD)
            .cloned()
            .collect();

        // let groups_of_combinable_states =
        //     group_by(&combined_states, ||);

        // let (new_combined_states, mut transitions) = minimize_step(combined_states, &transitions);
        // combined_states = new_combined_states;
        // while !combined_states.is_empty() {}

        transitions.sort_by_key(|s| (s.0, s.1));

        Self {
            prod0,
            transitions,
            k,
        }
    }

    // Accepting states that yield the same production index can be combined.
    // When we identify a duplicate state we remove it and let all references to it point to the
    // earlier found one. This is repeated until no changes can be made this way.
    fn optimize(self) -> CompiledDFA {
        let Self {
            prod0,
            mut transitions,
            k,
        } = self;

        // fn remove_duplicate_at_index(
        //     kept_index: usize,
        //     index_to_remove: usize,
        //     transitions: &mut [CompiledTransition],
        // ) {
        //     // debug_assert!(kept_index < index_to_remove);
        //     transitions.iter_mut().for_each(|t| {
        //         match t.0.cmp(&index_to_remove) {
        //             std::cmp::Ordering::Less => (),
        //             std::cmp::Ordering::Equal => t.0 = kept_index,
        //             std::cmp::Ordering::Greater => t.0 -= 1,
        //         }
        //         match t.2.cmp(&index_to_remove) {
        //             std::cmp::Ordering::Less => (),
        //             std::cmp::Ordering::Equal => t.2 = kept_index,
        //             std::cmp::Ordering::Greater => t.2 -= 1,
        //         }
        //     });
        // }

        // let mut changed = true;
        // 'NEXT: while changed {
        //     changed = false;
        //     for (index1, t1) in transitions.iter().enumerate() {
        //         for t2 in transitions.iter().skip(index1 + 1) {
        //             // Check for same result production number
        //             // Note that only accepting states carry a valid production number
        //             if t1.3 != INVALID_PROD && t1.3 == t2.3 && t1.2 != t2.2 {
        //                 remove_duplicate_at_index(t1.2, t2.2, &mut transitions);
        //                 changed = true;
        //                 continue 'NEXT;
        //             }
        //         }
        //     }
        // }

        transitions.sort_by_key(|s| (s.0, s.1));

        Self {
            prod0,
            transitions,
            k,
        }
    }
}
