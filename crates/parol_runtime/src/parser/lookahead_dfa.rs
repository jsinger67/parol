use crate::parser::INVALID_PROD;
use crate::{
    FormatToken, LexerError, NonTerminalIndex, ParolError, ParserError, ProductionIndex,
    StateIndex, TerminalIndex, TokenStream, TokenVec, UnexpectedToken,
};
use log::trace;
use std::cmp::Ordering;
use std::fmt::Debug;

use super::CompiledProductionIndex;

///
/// The transitions contain tuples: "from-state -> terminal-index -> to-state -> production index"
///
/// The CompiledProductionIndex is the production number that can be chosen when the to-state is
/// reached and the value is valid. The value is valid if it's not INVALID_PROD which denotes
/// that the to-state is not an accepting one.
///
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Trans(
    pub StateIndex,
    pub TerminalIndex,
    pub StateIndex,
    pub CompiledProductionIndex,
);

///
/// The lookahead DFA. Used to calculate a certain production number from a
/// sequence of terminals.
///
/// The start state is per definition always the state with index 0.
///
/// In the generated parsers there always exists exactly one LookaheadDFA for
/// each non-terminal.
///
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct LookaheadDFA {
    /// Contains the production number in initial state 0. If the automaton has not transitions this
    /// number will be returned.
    pub prod0: CompiledProductionIndex,

    ///
    /// Transitions are sorted
    ///     * firstly by the index of the from-state,
    ///     * secondly by the terminal index
    /// This way it is easy to detect the case where no match exists and to be
    /// able to quickly terminate the search for an applicable transition.
    ///
    pub transitions: &'static [Trans],

    ///
    /// Maximum number of tokens needed to reach an accepting state
    ///
    pub k: usize,
}

impl LookaheadDFA {
    ///
    /// Creates a new instance with the given parameters.
    ///
    pub fn new(prod0: CompiledProductionIndex, transitions: &'static [Trans], k: usize) -> Self {
        Self {
            prod0,
            transitions,
            k,
        }
    }

    ///
    /// Calculates the next production to use for the given non-terminal.
    /// Retrieves the lookahead tokens from the TokenStream object without
    /// consuming any of them.
    ///
    #[inline(always)]
    pub fn eval<F: Fn(char) -> Option<usize> + Clone>(
        &self,
        token_stream: &mut TokenStream<'_, F>,
        non_terminal: NonTerminalIndex,
    ) -> Result<ProductionIndex, ParolError> {
        let mut state: StateIndex = 0;
        let mut prod_num: CompiledProductionIndex = self.prod0;
        let mut last_prod_num: CompiledProductionIndex = INVALID_PROD;

        if self.k > token_stream.k {
            return Err(ParserError::DataError(
                "Lookahead size mismatch between token stream and Lookahead DFA",
            )
            .into());
        }
        let mut last_accepting_state: Option<StateIndex> = if prod_num > INVALID_PROD {
            Some(state)
        } else {
            None
        };
        for i in 0..self.k {
            // Read the current lookahead token and extract it's type
            let current_lookahead_token = token_stream.lookahead_token_type(i)?;

            // Filter the transitions with the matching from-state
            let mut any_matching_found = false;
            for i in 0..self.transitions.len() {
                let current_transition = &self.transitions[i];

                if current_transition.0 != state {
                    if any_matching_found {
                        // Since the transitions are sorted by from-state we
                        // have moved beyond the possible transitions and
                        // can safely stop the search here.
                        break;
                    } else {
                        // Try the next matching transition.
                        continue;
                    }
                }

                any_matching_found = true;

                // Test if there exists a transition from the from-state
                // via the current lookahead token
                match current_transition.1.cmp(&current_lookahead_token) {
                    Ordering::Equal => {
                        // Set the to_state and break into the outer for loop
                        // to finish if we found an accepting state or
                        // to read the next lookahead token if available.
                        trace!(
                            "{}, {} => {}",
                            state, current_lookahead_token, current_transition.2
                        );
                        // Set the state to the to-state
                        state = current_transition.2;
                        prod_num = current_transition.3;
                        // Test if the production in this transition is a valid one
                        // In this case the to-state is an accepting one.
                        if current_transition.3 > INVALID_PROD {
                            // In case we step too far, we can retrieve the last
                            // accepting state.
                            // Indeed we can step too far because the self.k is the
                            // maximum depth of all subtrees.
                            last_accepting_state = Some(current_transition.2);
                            last_prod_num = current_transition.3;
                            trace!("State {} accepts", current_transition.2);
                        }
                        break;
                    }
                    Ordering::Greater => {
                        // The token type is not found
                        break;
                    }
                    _ => (),
                }
            }
        }
        if prod_num > INVALID_PROD {
            // The state is accepting, we can return the associated production number
            trace!("Predict production {} at state {}", prod_num, state);
            Ok(prod_num as ProductionIndex)
        } else if let Some(last_state) = last_accepting_state {
            debug_assert!(last_prod_num > INVALID_PROD);
            trace!(
                "Predict production {:?} from last accepting state {}",
                last_prod_num, last_state
            );
            Ok(last_prod_num as ProductionIndex)
        } else {
            trace!(
                "Production prediction failed at state {} with tokens {:?}",
                state,
                token_stream.token_types()
            );
            return Err(ParserError::PredictionError {
                cause: format!(
                    "Production prediction failed for non-terminal {}",
                    non_terminal,
                ),
            }
            .into());
        }
    }

    ///
    /// Returns all terminals that lead from state 0 to a valid next state
    ///
    pub fn build_error<F: Fn(char) -> Option<usize> + Clone>(
        &self,
        terminal_names: &'static [&'static str],
        token_stream: &TokenStream<'_, F>,
    ) -> Result<(String, Vec<UnexpectedToken>, TokenVec), LexerError> {
        let mut state = 0;
        let mut diag_msg = String::new();
        let mut unexpected_tokens = Vec::new();
        let mut expected_tokens = TokenVec::default();
        for (lookahead, token) in token_stream.tokens.non_skip_tokens().enumerate() {
            let token_type = token.token_type;
            if let Some(transition) = self
                .transitions
                .iter()
                .position(|t| t.0 == state && t.1 == token_type)
            {
                diag_msg.push_str(
                    format!("LA({}): {} ", lookahead + 1, token.format(terminal_names)).as_str(),
                );
                unexpected_tokens.push(UnexpectedToken::new(
                    format!("LA({})", lookahead + 1),
                    terminal_names[token_type as usize].to_owned(),
                    token,
                ));
                state = self.transitions[transition].2;
            } else {
                diag_msg.push_str(
                    format!("LA({}): {}.", lookahead + 1, token.format(terminal_names),).as_str(),
                );
                unexpected_tokens.push(UnexpectedToken::new(
                    format!("LA({})", lookahead + 1),
                    terminal_names[token_type as usize].to_owned(),
                    token,
                ));
                expected_tokens = self.transitions.iter().filter(|t| t.0 == state).fold(
                    expected_tokens,
                    |mut acc, t| {
                        acc.push(terminal_names[t.1 as usize].to_owned());
                        acc
                    },
                );
                break;
            }
        }
        Ok((diag_msg, unexpected_tokens, expected_tokens))
    }
}
