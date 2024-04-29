use crate::analysis::{non_productive_non_terminals, unreachable_non_terminals};
use crate::parser::parol_grammar::GrammarType;
use crate::{augment_grammar, detect_left_recursive_non_terminals, left_factor, Cfg};
use crate::{GrammarAnalysisError, RecursiveNonTerminal, RelatedHint};
use parol_macros::bail;
use parol_runtime::Result;

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
///
///  Apply all grammar transformation necessary to be able to use the given grammar.
///
pub fn check_and_transform_grammar(cfg: &Cfg, grammar_type: GrammarType) -> Result<Cfg> {
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

    match grammar_type {
        GrammarType::LLK => check_and_transform_ll(cfg),
        GrammarType::LALR1 => check_and_transform_lr(cfg),
    }
}

fn check_and_transform_ll(cfg: &Cfg) -> Result<Cfg> {
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

fn check_and_transform_lr(cfg: &Cfg) -> Result<Cfg> {
    // let right_recursions = detect_right_recursive_non_terminals(cfg);
    // if !right_recursions.is_empty() {
    //     let recursions = right_recursions
    //         .iter()
    //         .enumerate()
    //         .map(|(number, name)| RecursiveNonTerminal {
    //             number,
    //             name: name.to_string(),
    //         })
    //         .collect::<Vec<RecursiveNonTerminal>>();

    //     bail!(GrammarAnalysisError::RightRecursion { recursions });
    // }
    Ok(augment_grammar(cfg))
}
