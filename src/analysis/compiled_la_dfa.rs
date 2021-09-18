use super::lookahead_dfa::ProductionIndex;
use super::lookahead_dfa::StateIndex;
use super::LookaheadDFA;

pub(crate) type TerminalIndex = usize;

///
/// Internal data structure to represent a CompiledDFA which in turn is used to
/// generate the parsers source code
///
#[derive(Debug, Clone)]
pub(crate) struct CompiledDFA {
    pub states: Vec<Option<ProductionIndex>>,
    pub transitions: Vec<(StateIndex, TerminalIndex, StateIndex)>,
    pub k: usize,
}

// Only used to circumvent the fact, that destructuring assignment is unstable
struct X(
    Vec<Option<ProductionIndex>>,
    Vec<(StateIndex, TerminalIndex, StateIndex)>,
);

impl CompiledDFA {
    pub fn from_lookahead_dfa(dfa: &LookaheadDFA) -> Self {
        let states = dfa
            .states
            .iter()
            .map(|s| if s.accepted { Some(s.prod_num) } else { None })
            .collect::<Vec<Option<ProductionIndex>>>();

        let transitions = dfa.transitions.iter().fold(
            Vec::<(StateIndex, TerminalIndex, StateIndex)>::new(),
            |mut acc, (ci, t)| {
                let mut transitions = t.iter().fold(
                    Vec::<(StateIndex, TerminalIndex, StateIndex)>::new(),
                    |mut acc, (trm, ns)| {
                        acc.push((*ci as StateIndex, *trm, *ns as StateIndex));
                        acc
                    },
                );
                transitions.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
                acc.append(&mut transitions);
                acc
            },
        );

        let k = dfa.k;

        let x = Self::optimize(states, transitions);

        Self {
            states: x.0,
            transitions: x.1,
            k,
        }
    }

    fn optimize(
        mut states: Vec<Option<ProductionIndex>>,
        mut transitions: Vec<(StateIndex, TerminalIndex, StateIndex)>,
    ) -> X {
        fn remove_duplicate_at_index(
            kept_index: usize,
            index_to_remove: usize,
            mut states: Vec<Option<ProductionIndex>>,
            mut transitions: Vec<(StateIndex, TerminalIndex, StateIndex)>,
        ) -> X {
            states.remove(index_to_remove);
            transitions = transitions
                .iter_mut()
                .map(|mut t| {
                    if t.0 == index_to_remove {
                        t.0 = kept_index;
                    }
                    if t.2 == index_to_remove {
                        t.2 = kept_index;
                    }
                    if t.0 > index_to_remove {
                        t.0 -= 1;
                    }
                    if t.2 > index_to_remove {
                        t.2 -= 1;
                    }
                    *t
                })
                .collect();
            X(states, transitions)
        }

        let mut changed = true;
        'NEXT: while changed {
            changed = false;
            for (index1, _state1) in states.iter().enumerate() {
                for (index2, _state2) in states.iter().enumerate().skip(index1 + 1) {
                    if let Some(s1) = &states[index1] {
                        if let Some(s2) = &states[index2] {
                            if s1 == s2 {
                                let x =
                                    remove_duplicate_at_index(index1, index2, states, transitions);
                                states = x.0;
                                transitions = x.1;
                                changed = true;
                                continue 'NEXT;
                            }
                        }
                    }
                }
            }
        }

        transitions.sort_by_key(|s| (s.0, s.1));

        X(states, transitions)
    }
}
