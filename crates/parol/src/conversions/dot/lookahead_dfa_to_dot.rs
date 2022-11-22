use crate::analysis::LookaheadDFA;
use crate::StrVec;
use std::fmt::Debug;

#[derive(BartDisplay, Debug, Default)]
#[template = "templates/lookahead_dfa.dot"]
struct NtDotElements<'a> {
    title: &'a str,
    start_state: String,
    states: StrVec,
    transitions: StrVec,
}

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
///
/// Formats the given DFA in a special dot-format.
///
pub fn render_dfa_dot_string(dfa: &LookaheadDFA, title: &str) -> String {
    let start_state = format!("0 [label = \"{}\"];", dfa.states[0]);

    let states = dfa
        .states
        .iter()
        .enumerate()
        .skip(1)
        .fold(StrVec::new(4), |mut acc, (i, s)| {
            acc.push(format!("{} [label = \"{}\"];", i, s));
            acc
        });

    let transitions =
        dfa.transitions
            .iter()
            .fold(StrVec::new(4), |mut acc, (from_state, trans)| {
                for (term, to_state) in trans {
                    let terminal_string = format!("{}", term);
                    acc.push(
                        format!(
                            "{} -> {} [label = \"{}\"];",
                            from_state,
                            to_state,
                            terminal_string.escape_default()
                        )
                        .to_string(),
                    );
                }
                acc
            });

    let elements = NtDotElements {
        title,
        start_state,
        states,
        transitions,
    };
    format!("{}", elements)
}
