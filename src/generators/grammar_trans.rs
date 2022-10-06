use crate::analysis::errors::{RecursiveNonTerminal, RelatedHint};
use crate::analysis::{
    non_productive_non_terminals, unreachable_non_terminals, GrammarAnalysisError,
};
use crate::{detect_left_recursive_non_terminals, left_factor, Cfg};
use miette::{bail, Result};

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
        let non_terminals = non_productive
            .iter()
            .map(|nt| RelatedHint {
                topic: "Non-terminal".to_string(),
                hint: nt.to_string(),
            })
            .collect::<Vec<RelatedHint>>();
        bail!(GrammarAnalysisError::NonProductiveNonTerminals { non_terminals });
    }
    let unreachable = unreachable_non_terminals(cfg);
    if !unreachable.is_empty() {
        let non_terminals = unreachable
            .iter()
            .map(|nt| RelatedHint {
                topic: "Non-terminal".to_string(),
                hint: nt.to_string(),
            })
            .collect::<Vec<RelatedHint>>();
        bail!(GrammarAnalysisError::UnreachableNonTerminals { non_terminals });
    }

    let left_recursions = detect_left_recursive_non_terminals(cfg);
    if !left_recursions.is_empty() {
        let recursions = left_recursions
            .iter()
            .enumerate()
            .map(|(number, name)| RecursiveNonTerminal {
                number,
                name: name.to_string(),
            })
            .collect::<Vec<RecursiveNonTerminal>>();

        bail!(GrammarAnalysisError::LeftRecursion { recursions });
    }

    Ok(left_factor(cfg))
}
