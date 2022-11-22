//! Grammar flow analysis
//! Productivity of non-terminals

use crate::grammar::symbol_string::SymbolString;
use crate::utils::{short_cut_conjunction_combine, short_cut_disjunction_combine};
use crate::{Cfg, Pr, Symbol /*SymbolString*/};
use log::trace;

/// Result type for each non-terminal:
/// true: productive
/// false: non-productive
type DomainType = bool;

/// The result vector applied to each iteration step;
/// is also returned after each iteration step
type ResultVector = Vec<DomainType>;

/// The type of the function in the equation system
/// It is called for each non-terminal
type TransferFunction<'a> = Box<dyn Fn(&ResultVector) -> DomainType + 'a>;

type EquationSystem<'a> = Vec<TransferFunction<'a>>;

type StepFunction = Box<dyn Fn(&EquationSystem, &ResultVector) -> ResultVector>;

/// Returns the non-productive terminals as a vector
pub fn non_productive_non_terminals(cfg: &Cfg) -> Vec<String> {
    // The indices within this vector of non-terminals corresponds to the indices in the result-vector.
    let non_terminals = cfg
        .get_non_terminal_set()
        .iter()
        .cloned()
        .collect::<Vec<String>>();

    let non_terminal_index =
        |nt: &str| -> usize { non_terminals.iter().position(|n| n == nt).unwrap() };

    let equation_system: EquationSystem = non_terminals.iter().fold(
        EquationSystem::with_capacity(non_terminals.len()),
        |mut es, nt| {
            let matching_productions = cfg.matching_productions(nt);
            es.push(combine_production_equation(
                matching_productions,
                non_terminal_index,
            ));
            es
        },
    );

    let step_function: StepFunction =
        Box::new(|es: &EquationSystem, result_vector: &ResultVector| {
            result_vector.iter().enumerate().fold(
                ResultVector::with_capacity(result_vector.len()),
                |mut acc, (i, _)| {
                    acc.push(es[i](result_vector));
                    acc
                },
            )
        });

    let mut result_vector = vec![false; non_terminals.len()];
    loop {
        let new_result_vector = step_function(&equation_system, &result_vector);
        trace!(
            "{}",
            trace_result_vector(&new_result_vector, &non_terminals)
        );
        if new_result_vector == result_vector {
            break;
        }
        result_vector = new_result_vector;
    }

    result_vector
        .iter()
        .enumerate()
        .fold(Vec::new(), |mut acc, (i, r)| {
            if !*r {
                acc.push(non_terminals[i].clone());
            }
            acc
        })
}

fn trace_result_vector(result_vector: &[bool], non_terminals: &[String]) -> String {
    result_vector
        .iter()
        .enumerate()
        .map(|(i, b)| format!("{}({})", non_terminals[i], if *b { "1" } else { "0" }))
        .collect::<Vec<String>>()
        .join(", ")
}

fn create_production_transfer_function<'a>(
    symbol_string: SymbolString,
    non_terminal_index: impl Fn(&str) -> usize + Clone,
) -> TransferFunction<'a> {
    let mut result_function: TransferFunction<'a> = Box::new(|_| true);
    for s in symbol_string.0 {
        if let Symbol::N(n, _, _) = s {
            let index = non_terminal_index(&n);
            let f = Box::new(move |result_vector: &ResultVector| result_vector[index]);
            result_function = Box::new(short_cut_conjunction_combine(result_function, f));
        }
    }
    result_function
}

fn combine_production_equation<'a, 'c>(
    matching_productions: Vec<(usize, &'c Pr)>,
    non_terminal_index: impl Fn(&str) -> usize + Clone,
) -> TransferFunction<'a> {
    if matching_productions.is_empty() {
        Box::new(|_| false)
    } else if matching_productions
        .iter()
        .any(|(_, p)| p.get_r().iter().all(|s| matches!(s, Symbol::T(_))))
    {
        Box::new(|_| true)
    } else {
        let mut symbol_strings =
            matching_productions
                .iter()
                .fold(Vec::new(), |mut acc, (_, pr)| {
                    acc.push(SymbolString::from_production(pr));
                    acc
                });
        let mut prod_functions = symbol_strings
            .drain(..)
            .map(|symbol_string| {
                create_production_transfer_function(symbol_string, non_terminal_index.clone())
            })
            .collect::<Vec<TransferFunction<'a>>>();
        let mut result_function: TransferFunction<'a> = Box::new(|_| false);
        for f in prod_functions.drain(..) {
            result_function = Box::new(short_cut_disjunction_combine(result_function, f));
        }
        result_function
    }
}
