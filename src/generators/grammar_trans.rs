use crate::analysis::errors::{Recursion, RelatedHint};
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
                hint: format!("{}", nt),
            })
            .collect::<Vec<RelatedHint>>();
        bail!(GrammarAnalysisError::UnreachableNonTerminals { non_terminals });
    }

    let left_recursions = detect_left_recursions(cfg);
    if !left_recursions.is_empty() {
        let recursions = left_recursions
            .iter()
            .map(|n| Recursion {
                hints: n
                    .iter()
                    .map(|s| RelatedHint {
                        hint: format!("{}", s),
                    })
                    .collect::<Vec<RelatedHint>>(),
            })
            .collect::<Vec<Recursion>>();

        bail!(GrammarAnalysisError::LeftRecursion { recursions });
    }

    Ok(left_factor(cfg))
}
