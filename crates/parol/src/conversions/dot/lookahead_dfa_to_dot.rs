use crate::analysis::LookaheadDFA;
use crate::StrVec;

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

    format!(
        r#"digraph G {{
    rankdir=LR;
    label="{title}";

    node [shape=point, style=invis]; ""
    node [shape=ellipse, color=cyan, style=solid];
    "" -> {start_state}

    node [shape=ellipse, color=cyan];
{states}

{transitions}
}}"#
    )
}

#[cfg(test)]
mod tests {
    use crate::analysis::lookahead_dfa::DFAState;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_render_dfa_dot_string() {
        let dfa = LookaheadDFA {
            states: vec![
                DFAState {
                    id: 0,
                    accepted: false,
                    prod_num: 63,
                },
                DFAState {
                    id: 1,
                    accepted: false,
                    prod_num: 64,
                },
                DFAState {
                    id: 2,
                    accepted: true,
                    prod_num: 65,
                },
            ],
            transitions: [
                (0, [(16, 2), (36, 1)].into_iter().collect()),
                (1, [(56, 3)].into_iter().collect()),
            ]
            .into_iter()
            .collect(),
            k: 1,
        };
        assert_eq!(
            render_dfa_dot_string(&dfa, "Test"),
            r#"digraph G {
    rankdir=LR;
    label="Test";

    node [shape=point, style=invis]; ""
    node [shape=ellipse, color=cyan, style=solid];
    "" -> 0 [label = "Id(0), Pr(63)"];

    node [shape=ellipse, color=cyan];
    1 [label = "Id(1), Pr(64)"];
    2 [label = "Id(2, accepting), Pr(65)"];


    0 -> 2 [label = "16"];
    0 -> 1 [label = "36"];
    1 -> 3 [label = "56"];

}"#
        );
    }
}
