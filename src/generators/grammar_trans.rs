use crate::analysis::{non_productive_non_terminals, unreachable_non_terminals};
use crate::errors::*;
use crate::{detect_left_recursions, left_factor, Cfg};

pub fn check_and_transform_grammar(cfg: &Cfg) -> Result<Cfg> {
    let non_productive = non_productive_non_terminals(cfg);
    if !non_productive.is_empty() {
        let non_productive_string = non_productive
            .iter()
            .map(|nt| nt.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        return Err(format!(
            "Grammar contains non-productive non-terminals:\n{}",
            non_productive_string
        )
        .into());
    }
    let unreachable = unreachable_non_terminals(cfg);
    if !unreachable.is_empty() {
        let unreachable_string = unreachable
            .iter()
            .map(|nt| nt.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        return Err(format!(
            "Grammar contains unreachable non-terminals:\n{}",
            unreachable_string
        )
        .into());
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

        return Err(format!(
            "Grammar contains left_recursions:\n{}",
            left_recursions_string
        )
        .into());
    }

    Ok(left_factor(cfg))
}
