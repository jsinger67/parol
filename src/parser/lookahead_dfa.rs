use crate::errors::{FileSource, LookaheadError, TokenVec, UnexpectedToken};
use crate::lexer::{FormatToken, TerminalIndex, Token, TokenStream};
use crate::parser::{ProductionIndex, StateIndex};
use log::trace;
use miette::{miette, Result, WrapErr};
use std::cmp::Ordering;
use std::fmt::Debug;
use std::path::Path;

///
/// Data structure to represent a DFA state
///
/// If the state is accepting the production index is valid and denotes
/// the resulting production index.
///
pub type DFAState = Option<ProductionIndex>;

///
/// The transitions contain tuples: "from-state -> terminal-index -> to-state"
///
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct DFATransition(pub StateIndex, pub TerminalIndex, pub StateIndex);

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
    ///
    /// States are identified by their index within the slice.
    /// It corresponds to the state's number
    ///
    pub states: &'static [DFAState],

    ///
    /// Transitions are sorted
    ///     * firstly by the index of the from-state,
    ///     * secondly by the terminal index
    /// This way it is easy to detect the case where no match exists and to be
    /// able to quickly terminate the search for an applicable transition.
    ///
    pub transitions: &'static [DFATransition],

    ///
    /// Maximum number of tokens needed to reach an accepting state
    ///
    pub k: usize,
}

impl LookaheadDFA {
    ///
    /// Creates a new instance with the given parameters.
    ///
    pub fn new(
        states: &'static [DFAState],
        transitions: &'static [DFATransition],
        k: usize,
    ) -> Self {
        Self {
            states,
            transitions,
            k,
        }
    }

    ///
    /// Calculates the next production to use for the given non-terminal.
    /// Retrieves the lookahead tokens from the TokenStream object without
    /// consuming any of them.
    ///
    pub fn eval<'t>(&self, token_stream: &mut TokenStream<'t>) -> Result<ProductionIndex> {
        let mut state: StateIndex = 0;
        if self.k > token_stream.k {
            return Err(miette!(LookaheadError::DataError(
                "Lookahead size mismatch between token stream and Lookahead DFA"
            )));
        }
        let mut last_accepting_state: Option<StateIndex> = None;
        for i in 0..self.k {
            // Read the current lookahead token and extract it's type
            let current_lookahead_token = token_stream
                .lookahead_token_type(i)
                .wrap_err("Error accessing lookahead token from token stream!")?;

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
                            state,
                            current_lookahead_token,
                            current_transition.2
                        );
                        state = current_transition.2;
                        if self.states[state].is_some() {
                            // In case we step too far, we can retrieve the last
                            // accepting state.
                            // Indeed we can step too far because the self.k is the
                            // maximum depth of all subtrees.
                            last_accepting_state = Some(state);
                            trace!("State {} accepts", state);
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
        if let Some(prod_num) = self.states[state] {
            // The state is accepting, we can return the associated production number
            trace!("Predict production {} at state {}", prod_num, state);
            Ok(prod_num)
        } else if let Some(last_state) = last_accepting_state {
            trace!(
                "Predict production {:?} from last accepting state {}",
                self.states[last_state],
                state
            );
            Ok(self.states[last_state].unwrap())
        } else {
            trace!(
                "Production prediction failed at state {} with token {:?}",
                state,
                token_stream.lookahead(0)
            );
            Err(miette!(LookaheadError::PredictionError {
                cause: format!("Production prediction failed at state {}", state),
            }))
        }
    }

    ///
    /// Returns all terminals that lead from state 0 to a valid next state
    ///
    pub fn build_error<T>(
        &self,
        terminal_names: &'static [&'static str],
        tokens: &[Token],
        file_name: T,
    ) -> Result<(String, Vec<UnexpectedToken>, TokenVec)>
    where
        T: AsRef<Path> + Debug,
    {
        let mut state = 0;
        let mut diag_msg = String::new();
        let mut unexpected_tokens = Vec::new();
        let mut expected_tokens = TokenVec::default();
        for (lookahead, token) in tokens.iter().enumerate() {
            let token_type = token.token_type;
            if let Some(transition) = self
                .transitions
                .iter()
                .position(|t| t.0 == state && t.1 == token_type)
            {
                diag_msg.push_str(
                    format!(
                        "LA({}): {} ",
                        lookahead + 1,
                        token.format(&file_name, terminal_names)
                    )
                    .as_str(),
                );
                unexpected_tokens.push(UnexpectedToken::new(
                    format!("LA({})", lookahead + 1),
                    terminal_names[token_type].to_owned(),
                    FileSource::try_new(file_name.as_ref().to_path_buf())?.into(),
                    token,
                ));
                state = self.transitions[transition].2;
            } else {
                diag_msg.push_str(
                    format!(
                        "LA({}): {}.",
                        lookahead + 1,
                        token.format(&file_name, terminal_names),
                    )
                    .as_str(),
                );
                unexpected_tokens.push(UnexpectedToken::new(
                    format!("LA({})", lookahead + 1),
                    terminal_names[token_type].to_owned(),
                    FileSource::try_new(file_name.as_ref().to_path_buf())?.into(),
                    token,
                ));
                expected_tokens = self.transitions.iter().filter(|t| t.0 == state).fold(
                    expected_tokens,
                    |mut acc, t| {
                        acc.push(format!(r#""{}""#, terminal_names[t.1]));
                        acc
                    },
                );
                break;
            }
        }
        Ok((diag_msg, unexpected_tokens, expected_tokens))
    }
}
