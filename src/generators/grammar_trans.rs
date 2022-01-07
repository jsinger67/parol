use crate::analysis::{non_productive_non_terminals, unreachable_non_terminals};
use crate::{detect_left_recursions, left_factor, Cfg};
use miette::{miette, Result};

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
///
///  Apply all grammar transformation necessary to be able to use the given grammar.
///
pub fn check_and_transform_grammar(cfg: &Cfg) -> Result<Cfg> {
    let non_productive = non_productive_non_terminals(cfg);
    if !non_productive.is_empty() {
        let non_productive_string = non_productive
            .iter()
            .map(|nt| nt.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        return Err(miette!(
            "Grammar contains non-productive non-terminals:\n{}",
            non_productive_string
        ));
    }
    let unreachable = unreachable_non_terminals(cfg);
    if !unreachable.is_empty() {
        let unreachable_string = unreachable
            .iter()
            .map(|nt| nt.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        return Err(miette!(
            "Grammar contains unreachable non-terminals:\n{}",
            unreachable_string
        ));
    }

    let left_recursions = detect_left_recursions(cfg);
    if !left_recursions.is_empty() {
        let left_recursions_string = left_recursions
            .iter()
            .map(|n| {
                n.iter()
                    .map(|s| format!("{}", s))
                    .collect::<Vec<String>>()
                    .join(" => ")
            })
            .collect::<Vec<String>>()
            .join(", ");

        return Err(miette!(
            "Grammar contains left_recursions:\n{}",
            left_recursions_string
        ));
    }

    Ok(left_factor(cfg))
}
