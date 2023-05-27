use std::collections::BTreeMap;

use super::lookahead_dfa::{CompiledProductionIndex, StateIndex, INVALID_PROD};
use super::LookaheadDFA;

pub(crate) type TerminalIndex = usize;

mod adjacency_list {
    use crate::analysis::compiled_la_dfa::TerminalIndex;
    use crate::analysis::lookahead_dfa::{CompiledProductionIndex, INVALID_PROD};
    use crate::group_by;
    use std::collections::{HashMap, VecDeque};

    use super::{CompiledDFA, CompiledTransition};

    type StateId = usize;

    #[derive(Debug, Clone)]
    struct AdjacencyListEntry {
        neighbors: HashMap<StateId, TerminalIndex>,
    }

    impl AdjacencyListEntry {
        fn new() -> Self {
            AdjacencyListEntry {
                neighbors: HashMap::new(),
            }
        }

        fn add_neighbor(&mut self, neighbor_id: StateId, neighbor_term: TerminalIndex) {
            self.neighbors.insert(neighbor_id, neighbor_term);
        }

        fn rename_neighbor(&mut self, id: StateId, new_id: StateId) {
            if let Some(term) = self.neighbors.remove(&id) {
                self.neighbors.insert(new_id, term);
            }
        }
    }

    #[derive(Debug)]
    pub(super) struct AdjacencyList {
        list: HashMap<StateId, AdjacencyListEntry>,
        productions: HashMap<StateId, CompiledProductionIndex>,
        prod0: CompiledProductionIndex,
        k: usize,
    }

    impl From<CompiledDFA> for AdjacencyList {
        /// Conversion from CompiledDFA
        fn from(value: CompiledDFA) -> Self {
            let mut list = HashMap::new();
            let mut productions = HashMap::new();
            list.insert(0, AdjacencyListEntry::new());
            productions.insert(0, value.prod0);

            for t in &value.transitions {
                // We use the to-state and the resulting production number
                list.insert(t.to_state, AdjacencyListEntry::new());
                productions.insert(t.to_state, t.prod_num);
            }

            for t in &value.transitions {
                // We add the neighbors (to-state) to each node (from-state)
                if let Some(f) = list.get_mut(&t.from_state) {
                    f.add_neighbor(t.to_state, t.term);
                }
            }

            AdjacencyList {
                list,
                productions,
                prod0: value.prod0,
                k: value.k,
            }
        }
    }

    impl AdjacencyList {
        #[inline]
        fn remove_state(&mut self, id: StateId) {
            self.list.remove(&id);
            self.productions.remove(&id);
        }

        fn rename_state(&mut self, id: StateId, new_id: StateId) {
            for (_, e) in &mut self.list {
                e.rename_neighbor(id, new_id);
            }
        }

        fn combine_two_states(&mut self, state_to_keep: StateId, state_to_merge: StateId) {
            debug_assert_ne!(state_to_keep, state_to_merge);
            if let (
                Some(_),
                Some(adj_list_of_merge_state),
                Some(prod_num_of_keep_state),
                Some(prod_num_of_merge_state),
            ) = (
                self.list.get(&state_to_keep),
                self.list.get(&state_to_merge).map(|l| l.clone()),
                self.productions.get(&state_to_keep),
                self.productions.get(&state_to_merge),
            ) {
                debug_assert_eq!(prod_num_of_keep_state, prod_num_of_merge_state);
                // Combine the adjacency list of two states in the state to keep
                self.list
                    .get_mut(&state_to_keep)
                    .map(|l| l.neighbors.extend(adj_list_of_merge_state.neighbors));
                // Remove the state to merge
                self.remove_state(state_to_merge);
                // Rename the state_to_merge into state_to_keep in all adjacency lists
                self.rename_state(state_to_merge, state_to_keep);
            } else {
                debug_assert!(false, "Internal error in combine_states");
            }
        }

        fn combine_states(&mut self, mut states: VecDeque<StateId>) {
            if let Some(state_to_keep) = states.pop_front() {
                while let Some(state_to_merge) = states.pop_front() {
                    self.combine_two_states(state_to_keep, state_to_merge);
                }
            }
        }

        pub(super) fn minimize(&mut self) {
            let final_states: Vec<(StateId, CompiledProductionIndex)> = self
                .productions
                .iter()
                .filter(|t| *t.1 != INVALID_PROD)
                .map(|t| (*t.0, *t.1))
                .collect();
            let combinable_groups = group_by(&final_states, |t| t.1);
            for g in combinable_groups {
                let states = g.1.iter().map(|s| s.0).collect();
                self.combine_states(states);
            }
        }

        pub(super) fn to_compiled_dfa(self) -> CompiledDFA {
            let mut transitions: Vec<CompiledTransition> = vec![];
            for (s, l) in self.list {
                for t in l.neighbors {
                    let p = self.productions.get(&t.0).unwrap();
                    transitions.push(CompiledTransition {
                        from_state: s,
                        term: t.1,
                        to_state: t.0,
                        prod_num: *p,
                    })
                }
            }
            transitions.sort_by_key(|s| (s.from_state, s.term));
            CompiledDFA {
                prod0: self.prod0,
                transitions,
                k: self.k,
            }
        }
    }

    #[cfg(test)]
    mod test {
        use crate::analysis::compiled_la_dfa::{CompiledDFA, CompiledTransition};

        use super::AdjacencyList;

        macro_rules! trans {
            ($from:literal, $term:literal, $to:literal, $prod:literal) => {
                CompiledTransition {
                    from_state: $from,
                    term: $term,
                    to_state: $to,
                    prod_num: $prod,
                }
            };
        }

        // Taken from example 'list_auto', Non-terminal "ItemsList"
        //   Id(0)
        //   Id(1)
        //   Id(2, accepting), Pr(4)
        //   Id(3, accepting), Pr(5)
        //   Id(4, accepting), Pr(5)
        // Transitions:
        //   0
        //   => 0 => 3
        //   => 5 => 1
        //   1
        //   => 0 => 4
        //   => 6 => 2
        // k:2
        #[test]
        fn test_conversion() {
            let mut transitions = vec![];
            transitions.push(trans!(0, 0, 3, 5));
            transitions.push(trans!(0, 5, 1, -1));
            transitions.push(trans!(1, 0, 4, 5));
            transitions.push(trans!(1, 6, 2, 4));
            let dfa = CompiledDFA {
                prod0: -1,
                transitions,
                k: 2,
            };
            let adjacency_list: AdjacencyList = dfa.into();
            assert_eq!(2, adjacency_list.k);
            assert_eq!(-1, adjacency_list.prod0);
            assert_eq!(5, adjacency_list.list.len());
            assert_eq!(5, adjacency_list.productions.len());
            for s in [0, 1, 2, 3, 4] {
                assert!(adjacency_list.list.contains_key(&s));
                assert!(adjacency_list.productions.contains_key(&s));
            }
            for (i, (s, p)) in [(0, -1), (1, -1), (2, 4), (3, 5), (4, 5)]
                .iter()
                .enumerate()
            {
                assert_eq!(
                    p,
                    adjacency_list.productions.get(&s).unwrap(),
                    "at index {}",
                    i
                );
            }
            assert_eq!(2, adjacency_list.list.get(&0).unwrap().neighbors.len());
            assert_eq!(2, adjacency_list.list.get(&1).unwrap().neighbors.len());
            assert_eq!(0, adjacency_list.list.get(&2).unwrap().neighbors.len());
            assert_eq!(0, adjacency_list.list.get(&3).unwrap().neighbors.len());
            assert_eq!(0, adjacency_list.list.get(&4).unwrap().neighbors.len());

            assert!(adjacency_list
                .list
                .get(&0)
                .unwrap()
                .neighbors
                .contains_key(&1));
            assert!(adjacency_list
                .list
                .get(&0)
                .unwrap()
                .neighbors
                .contains_key(&3));

            assert!(adjacency_list
                .list
                .get(&1)
                .unwrap()
                .neighbors
                .contains_key(&2));
            assert!(adjacency_list
                .list
                .get(&1)
                .unwrap()
                .neighbors
                .contains_key(&4));
        }
        #[test]
        fn test_minimize() {
            let mut transitions = vec![];
            transitions.push(trans!(0, 0, 3, 5));
            transitions.push(trans!(0, 5, 1, -1));
            transitions.push(trans!(1, 0, 4, 5));
            transitions.push(trans!(1, 6, 2, 4));
            let dfa = CompiledDFA {
                prod0: -1,
                transitions,
                k: 2,
            };
            let mut adjacency_list: AdjacencyList = dfa.into();
            assert_eq!(5, adjacency_list.list.len());
            assert_eq!(5, adjacency_list.productions.len());
            adjacency_list.minimize();
            assert_eq!(4, adjacency_list.list.len());
            assert_eq!(4, adjacency_list.productions.len());

            let dfa = adjacency_list.to_compiled_dfa();
            assert_eq!(-1, dfa.prod0);
            assert_eq!(2, dfa.k);
            assert_eq!(4, dfa.transitions.len());

            assert!(dfa
                .transitions
                .iter()
                .find(|t| { *t == &trans!(0, 0, 3, 5) })
                .is_some());
            assert!(dfa
                .transitions
                .iter()
                .find(|t| { *t == &trans!(0, 5, 1, -1) })
                .is_some());
            assert!(dfa
                .transitions
                .iter()
                .find(|t| { *t == &trans!(1, 0, 3, 5) })
                .is_some());
            assert!(dfa
                .transitions
                .iter()
                .find(|t| { *t == &trans!(1, 6, 2, 4) })
                .is_some());
        }
    }
}

/// A Four-Tuple type consisting of
/// * Start state index (from-state)
/// * Terminal index occurred in start state that triggers the transition
/// * Result state index (to-state)
/// * Possible ProductionIndex
#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct CompiledTransition {
    pub(crate) from_state: StateIndex,
    pub(crate) term: TerminalIndex,
    pub(crate) to_state: StateIndex,
    pub(crate) prod_num: CompiledProductionIndex,
}

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
                                acc.push(CompiledTransition {
                                    from_state: *ci as StateIndex,
                                    term: *trm,
                                    to_state: *ns as StateIndex,
                                    prod_num: *states.get(ns).unwrap_or(&INVALID_PROD),
                                });
                                acc
                            });
                    transitions.sort_by(|a, b| a.term.partial_cmp(&b.term).unwrap());
                    acc.append(&mut transitions);
                    acc
                });

        let k = dfa.k;

        Self::minimize(Self {
            prod0,
            transitions,
            k,
        })
    }

    fn minimize(self) -> CompiledDFA {
        let mut adjacency_list: adjacency_list::AdjacencyList = self.into();
        adjacency_list.minimize();
        adjacency_list.to_compiled_dfa()
    }

    // Accepting states that yield the same production index can be combined.
    // When we identify a duplicate state we remove it and let all references to it point to the
    // earlier found one. This is repeated until no changes can be made this way.
    fn _optimize(self) -> CompiledDFA {
        let Self {
            prod0,
            mut transitions,
            k,
        } = self;

        fn remove_duplicate_at_index(
            kept_index: usize,
            index_to_remove: usize,
            transitions: &mut [CompiledTransition],
        ) {
            // debug_assert!(kept_index < index_to_remove);
            transitions.iter_mut().for_each(|t| {
                match t.from_state.cmp(&index_to_remove) {
                    std::cmp::Ordering::Less => (),
                    std::cmp::Ordering::Equal => t.from_state = kept_index,
                    std::cmp::Ordering::Greater => t.from_state -= 1,
                }
                match t.to_state.cmp(&index_to_remove) {
                    std::cmp::Ordering::Less => (),
                    std::cmp::Ordering::Equal => t.to_state = kept_index,
                    std::cmp::Ordering::Greater => t.to_state -= 1,
                }
            });
        }

        let mut changed = true;
        'NEXT: while changed {
            changed = false;
            for (index1, t1) in transitions.iter().enumerate() {
                for t2 in transitions.iter().skip(index1 + 1) {
                    // Check for same result production number
                    // Note that only accepting states carry a valid production number
                    if t1.prod_num != INVALID_PROD
                        && t1.prod_num == t2.prod_num
                        && t1.to_state != t2.to_state
                    {
                        remove_duplicate_at_index(t1.to_state, t2.to_state, &mut transitions);
                        changed = true;
                        continue 'NEXT;
                    }
                }
            }
        }

        transitions.sort_by_key(|s| (s.from_state, s.term));

        Self {
            prod0,
            transitions,
            k,
        }
    }
}
