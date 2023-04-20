use crate::analysis::compiled_la_dfa::TerminalIndex;
use crate::KTuples;
use anyhow::{bail, Result};
use parol_runtime::log::trace;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt::{Display, Error, Formatter};

/// Index type of DFA states
pub type StateIndex = usize;
/// Index type of Productions
pub type ProductionIndex = usize;
/// Index type of Productions in generated automata
pub type CompiledProductionIndex = i32;
/// Invalid production number
/// It usually denotes the absence of a valid production number after applying a transition
pub const INVALID_PROD: CompiledProductionIndex = -1;

///
/// Data structure to represent a DFA state
///
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct DFAState {
    ///
    /// A unique state number, actually the index into the array of states.
    ///
    pub id: StateIndex,

    ///
    /// Used to detect conflicts.
    /// A conflict can occur in union operations.
    /// When combining two states that are both accepted and have different
    /// production numbers a conflict is detected.
    ///
    pub prod_num: CompiledProductionIndex,
}

impl DFAState {
    pub(crate) fn is_accepting(&self) -> bool {
        self.prod_num >= 0
    }
}

impl Display for DFAState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        let prod_num = if self.prod_num == INVALID_PROD {
            "".to_owned()
        } else {
            format!(", Pr({})", self.prod_num)
        };
        let accepted = if self.is_accepting() {
            ", accepting"
        } else {
            ""
        };
        write!(f, "Id({}{}){}", self.id, accepted, prod_num)
    }
}

///
/// The lookahead DFA. Used to calculate a certain production number from a
/// sequence of terminals.
///
/// The start state is per definition always the state with index 0.
///
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct LookaheadDFA {
    /// DFA states
    pub states: Vec<DFAState>,

    /// The transitions data is the relation: "from-state -> terminal -> to-state"
    /// Actually a map of from-states to terminal transitions, which in turn are
    /// maps from terminals to to-states.
    pub transitions: BTreeMap<StateIndex, BTreeMap<TerminalIndex, StateIndex>>,

    /// Maximum number of tokens needed to reach an accepting state
    /// It is equivalent to the maximum length over all contributing k-tuples.
    pub k: usize,
}

///
/// Internal type
/// Information about a possible transition from a given state via a given
/// terminal
///
#[derive(Debug, Clone, Eq, PartialEq)]
enum TransitionInfo {
    /// No transition from a given state via a given terminal exists
    NoTransition,
    /// There are other transitions from the given state but not via the
    /// given terminal
    /// Means the hash exists
    OtherTransitions,
    /// A transition from a given state via a given terminal exists
    /// Contains the index of the to_state
    TransitionExists(StateIndex),
}

impl LookaheadDFA {
    /// Create a DFA from KTuples
    /// The idea is to convert lists of terminals into a trie data structure
    pub fn from_k_tuples(k_tuples: &KTuples, prod_num: usize) -> Self {
        let mut dfa = Self {
            states: vec![DFAState {
                id: 0,
                prod_num: if k_tuples.is_empty() {
                    prod_num as i32
                } else {
                    INVALID_PROD
                },
            }],
            transitions: BTreeMap::new(),
            k: 0,
        };
        trace!("KTuples for production {prod_num}");
        for k_tuple in &k_tuples.sorted() {
            trace!("{k_tuple}");
            let mut current_state = 0;
            let tuple = k_tuple.terminals.inner();
            for i in 0..tuple.i {
                current_state = dfa.add_transition(current_state, tuple.t[i].0);
            }
            // The last created state is always accepting and needs to have a
            // valid production number
            dfa.states[current_state].prod_num = prod_num as i32;
            dfa.k = std::cmp::max(dfa.k, k_tuple.len());
        }
        dfa
    }

    fn add_transition(&mut self, from_state: StateIndex, terminal: TerminalIndex) -> StateIndex {
        match self.transition_info(from_state, terminal) {
            TransitionInfo::TransitionExists(to_state) => to_state,
            TransitionInfo::OtherTransitions => {
                // The transition hash in the from_state already exists,
                // but we have to add a new transition via the given terminal
                // to a newly created state
                let to_state = self.new_state();
                let transitions_from_state = self.transitions.get_mut(&from_state).unwrap();
                transitions_from_state.insert(terminal, to_state);
                to_state
            }
            TransitionInfo::NoTransition => {
                // No transitions exist for the current state yet.
                let to_state = self.new_state();
                let mut transitions_from_state = BTreeMap::new();
                transitions_from_state.insert(terminal, to_state);
                self.transitions.insert(from_state, transitions_from_state);
                to_state
            }
        }
    }

    ///
    /// Returns the union of self and other without changing self.
    /// If there exists a conflict in the accepting states production numbers
    /// an error is returned.
    ///
    pub fn union(&self, other: &Self) -> Result<Self> {
        self.clone().unite(other)
    }

    ///
    /// Returns the union of self and other while consuming self.
    /// If there exists a conflict in the accepting state's production numbers
    /// an error is returned.
    ///
    pub fn unite(self, other: &Self) -> Result<Self> {
        // Helper map for other's states: state in other -> state in union
        let state_mapping: RefCell<BTreeMap<StateIndex, StateIndex>> =
            RefCell::new(BTreeMap::new());
        // Starting state is per definition always 0!
        state_mapping.borrow_mut().insert(0, 0);
        let result_union = RefCell::new(self);

        loop {
            let mut changed = false;
            for tr in &other.transitions {
                if state_mapping.borrow().contains_key(tr.0) {
                    let result_state_index = *state_mapping.borrow().get(tr.0).unwrap();
                    for (terminal, to_state) in tr.1 {
                        let result_state = result_union
                            .borrow_mut()
                            .add_transition(result_state_index, *terminal);
                        if state_mapping
                            .borrow_mut()
                            .insert(*to_state, result_state)
                            .is_none()
                        {
                            changed = true;
                            let other_state_accepted = other.states[*to_state].is_accepting();
                            let other_to_state_prod_num = other.states[*to_state].prod_num;

                            let result_state_accepted =
                                result_union.borrow().states[result_state].is_accepting();
                            let result_state_prod_num =
                                result_union.borrow().states[result_state].prod_num;

                            if other_state_accepted
                                && result_state_accepted
                                && (other_to_state_prod_num != result_state_prod_num)
                            {
                                let message = format!(
                                    r#"Conflict in union operation detected:
Ambiguous production number prediction
{} <--> {}"#,
                                    result_state_prod_num, other_to_state_prod_num
                                );
                                bail!(message);
                            }
                            result_union
                                .borrow_mut()
                                .coin_state(result_state, other_to_state_prod_num);
                        }
                    }
                }
            }
            if !changed {
                break;
            }
        }
        Ok(result_union.into_inner())
    }

    fn new_state(&mut self) -> StateIndex {
        let id = self.states.len();
        self.states.push(DFAState {
            id,
            prod_num: INVALID_PROD,
        });
        id
    }

    fn coin_state(&mut self, id: StateIndex, prod_num: CompiledProductionIndex) {
        self.states[id].prod_num = prod_num;
    }

    fn transition_info(&self, from_state: StateIndex, terminal: TerminalIndex) -> TransitionInfo {
        let transitions_from_state_exist = self.transitions.contains_key(&from_state);
        let transition_from_state_via_terminal_exists = if transitions_from_state_exist {
            self.transitions
                .get(&from_state)
                .unwrap()
                .contains_key(&terminal)
        } else {
            false
        };
        match (
            transitions_from_state_exist,
            transition_from_state_via_terminal_exists,
        ) {
            (true, true) => TransitionInfo::TransitionExists(
                *self
                    .transitions
                    .get(&from_state)
                    .unwrap()
                    .get(&terminal)
                    .unwrap(),
            ),
            (true, false) => TransitionInfo::OtherTransitions,
            _ => TransitionInfo::NoTransition,
        }
    }
}

impl Display for LookaheadDFA {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        let states = self
            .states
            .iter()
            .map(|s| format!("  {}", s))
            .collect::<Vec<String>>()
            .join("\n");
        let transitions = self
            .transitions
            .iter()
            .map(|(s, t)| {
                let ts = t
                    .iter()
                    .map(|(terminal, to_state)| format!("  => {} => {}", terminal, to_state))
                    .collect::<Vec<String>>()
                    .join("\n");
                format!("  {}\n{}", s, ts)
            })
            .collect::<Vec<String>>()
            .join("\n");
        write!(f, "States\n{}\nTransitions:\n{}\n", states, transitions)
    }
}

// #[cfg(test)]
// mod test {
//     use crate::analysis::LookaheadDFA;
//     use crate::{KTuple, KTuples, Terminal};
//     use std::collections::HashSet;

//     #[test]
//     fn test_from_k_tuples() {
//         let k_tuples = KTuples(
//             vec![
//                 KTuple::of(vec![Terminal::End]),
//                 KTuple::of(vec![Terminal::t("a"), Terminal::End]),
//                 KTuple::of(vec![Terminal::t("a"), Terminal::t("a"), Terminal::End]),
//                 KTuple::of(vec![
//                     Terminal::t("a"),
//                     Terminal::t("a"),
//                     Terminal::t("a"),
//                     Terminal::End,
//                 ]),
//                 KTuple::of(vec![
//                     Terminal::t("a"),
//                     Terminal::t("a"),
//                     Terminal::t("a"),
//                     Terminal::t("a"),
//                 ]),
//             ]
//             .iter()
//             .cloned()
//             .collect::<HashSet<KTuple<Terminal>>>(),
//         );

//         let la_dfa = LookaheadDFA::from_k_tuples(&k_tuples, 1);

//         println!("LA_DFA\n{}", la_dfa);
//         assert_eq!(9, la_dfa.states.len());
//     }

//     #[test]
//     fn test_union() {
//         let k_tuples1 = KTuples(
//             vec![
//                 KTuple::of(vec![Terminal::End]),
//                 KTuple::of(vec![Terminal::t("a"), Terminal::End]),
//                 KTuple::of(vec![Terminal::t("a"), Terminal::t("a"), Terminal::End]),
//                 KTuple::of(vec![
//                     Terminal::t("a"),
//                     Terminal::t("a"),
//                     Terminal::t("a"),
//                     Terminal::End,
//                 ]),
//                 KTuple::of(vec![
//                     Terminal::t("a"),
//                     Terminal::t("a"),
//                     Terminal::t("a"),
//                     Terminal::t("a"),
//                 ]),
//             ]
//             .iter()
//             .cloned()
//             .collect::<HashSet<KTuple<Terminal>>>(),
//         );

//         let k_tuples2 = KTuples(
//             vec![
//                 KTuple::of(vec![Terminal::t("a"), Terminal::t("b"), Terminal::End]),
//                 KTuple::of(vec![
//                     Terminal::t("a"),
//                     Terminal::t("b"),
//                     Terminal::t("b"),
//                     Terminal::End,
//                 ]),
//                 KTuple::of(vec![
//                     Terminal::t("a"),
//                     Terminal::t("a"),
//                     Terminal::t("b"),
//                     Terminal::t("b"),
//                 ]),
//             ]
//             .iter()
//             .cloned()
//             .collect::<HashSet<KTuple<Terminal>>>(),
//         );

//         let la_dfa1 = LookaheadDFA::from_k_tuples(&k_tuples1, 1);
//         let la_dfa2 = LookaheadDFA::from_k_tuples(&k_tuples2, 2);

//         println!("LA_DFA1\n{}", la_dfa1);
//         assert_eq!(9, la_dfa1.states.len());
//         assert_eq!(4, la_dfa1.k);

//         println!("LA_DFA2\n{}", la_dfa2);
//         assert_eq!(9, la_dfa2.states.len());
//         assert_eq!(4, la_dfa2.k);

//         let la_union = la_dfa1.union(&la_dfa2).unwrap();
//         println!("UNION\n{}", la_union);
//         assert_eq!(15, la_union.states.len());
//         assert_eq!(4, la_union.k);
//     }

//     #[test]
//     #[should_panic(expected = "Conflict in union operation detected")]
//     fn test_union_finds_ambiguity() {
//         let k_tuples1 = KTuples(
//             vec![
//                 KTuple::of(vec![Terminal::End]),
//                 KTuple::of(vec![Terminal::t("a"), Terminal::End]),
//                 KTuple::of(vec![Terminal::t("a"), Terminal::t("a"), Terminal::End]),
//                 KTuple::of(vec![
//                     Terminal::t("a"),
//                     Terminal::t("a"),
//                     Terminal::t("a"),
//                     Terminal::End,
//                 ]),
//                 KTuple::of(vec![
//                     Terminal::t("a"),
//                     Terminal::t("a"),
//                     Terminal::t("a"),
//                     Terminal::t("a"),
//                 ]),
//             ]
//             .iter()
//             .cloned()
//             .collect::<HashSet<KTuple<Terminal>>>(),
//         );

//         let k_tuples2 = KTuples(
//             vec![
//                 KTuple::of(vec![Terminal::t("a"), Terminal::t("b"), Terminal::End]),
//                 KTuple::of(vec![
//                     Terminal::t("a"),
//                     Terminal::t("b"),
//                     Terminal::t("b"),
//                     Terminal::End,
//                 ]),
//                 KTuple::of(vec![
//                     Terminal::t("a"),
//                     Terminal::t("a"),
//                     Terminal::t("a"),
//                     Terminal::t("a"),
//                 ]),
//             ]
//             .iter()
//             .cloned()
//             .collect::<HashSet<KTuple<Terminal>>>(),
//         );

//         let la_dfa1 = LookaheadDFA::from_k_tuples(&k_tuples1, 1);
//         let la_dfa2 = LookaheadDFA::from_k_tuples(&k_tuples2, 2);

//         // This unwrap should panic here!
//         let _ = la_dfa1.union(&la_dfa2).unwrap();
//     }
// }
