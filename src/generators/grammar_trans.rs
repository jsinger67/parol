use crate::analysis::errors::{RecursionPath, RelatedHint};
use crate::analysis::{
    non_productive_non_terminals, unreachable_non_terminals, GrammarAnalysisError,
};
use crate::{detect_left_recursions, left_factor, Cfg};
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
                hint: format!("{}", nt),
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
                hint: format!("{}", nt),
            })
            .collect::<Vec<RelatedHint>>();
        bail!(GrammarAnalysisError::UnreachableNonTerminals { non_terminals });
    }

    let left_recursions = detect_left_recursions(cfg);
    if !left_recursions.is_empty() {
        let recursions = left_recursions
            .iter()
            .enumerate()
            .map(|(number, path_elements)| RecursionPath {
                number,
                hints: path_elements
                    .iter()
                    .map(|s| RelatedHint {
                        topic: "Recursion path element".to_string(),
                        hint: format!("{}", s),
                    })
                    .collect::<Vec<RelatedHint>>(),
            })
            .collect::<Vec<RecursionPath>>();

        bail!(GrammarAnalysisError::LeftRecursion { recursions });
    }

    Ok(left_factor(cfg))
}
