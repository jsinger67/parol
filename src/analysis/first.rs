//!
//! Grammar flow analysis
//! FIRSTk of productions and non-terminals
//!

use crate::analysis::compiled_la_dfa::TerminalIndex;
use crate::analysis::FirstCache;
use crate::{CompiledTerminal, GrammarConfig, KTuple, KTuples, Pr, Symbol, SymbolString};
use log::trace;
use std::collections::HashMap;

pub type FirstSet = (Vec<KTuples>, HashMap<String, KTuples>);

/// Result type for each production:
/// The set of the first k terminals
type DomainType = KTuples;

/// The result vector applied to each iteration step;
/// is also returned after each iteration step
/// The first indices correspond to the production number
/// After the Tuples for each production the Tuples for non-terminals are
/// following.
type ResultVector = Vec<DomainType>;

/// The type of the function in the equation system
/// It is called for each non-terminal
type TransferFunction<'a> = Box<dyn Fn(&ResultVector) -> DomainType + 'a>;

type EquationSystem<'a> = Vec<TransferFunction<'a>>;

type StepFunction = Box<dyn Fn(&EquationSystem, &ResultVector) -> ResultVector>;

///
/// Calculates the FIRST k sets for all productions of the given grammar.
/// The indices in the returned vector correspond to the production number.
///
pub fn first_k(grammar_config: &GrammarConfig, k: usize, first_cache: &FirstCache) -> FirstSet {
    let cfg = &grammar_config.cfg;

    let pr_count = cfg.pr.len();

    // The indices within this vector of non-terminals corresponds to the
    // indices in the result-vector.
    let non_terminals = cfg
        .get_non_terminal_set()
        .iter()
        .cloned()
        .collect::<Vec<String>>();
    let nt_count = non_terminals.len();

    let non_terminal_index =
        |nt: &str| -> usize { non_terminals.iter().position(|n| n == nt).unwrap() };

    let augmented_terminals = grammar_config.generate_augmented_terminals();
    let terminals = augmented_terminals.to_vec();

    let terminal_index = |nt: &str| -> usize { terminals.iter().position(|n| *n == nt).unwrap() };

    let nt_for_production: Vec<usize> =
        non_terminals.iter().fold(vec![0; pr_count], |mut acc, nt| {
            let non_terminal_index = non_terminal_index(nt);
            for (pi, _) in cfg.matching_productions(nt) {
                acc[pi] = non_terminal_index;
            }
            acc
        });

    // trace!("nt_for_production: {:?}", nt_for_production);

    let equation_system: EquationSystem =
        cfg.pr
            .iter()
            .fold(EquationSystem::with_capacity(pr_count), |mut es, pr| {
                // trace!("{}:", pr);
                es.push(combine_production_equation(
                    pr,
                    pr_count,
                    &terminal_index,
                    &non_terminal_index,
                    k,
                ));
                es
            });

    let step_function: StepFunction = {
        // let terminals = terminals.clone();
        Box::new(move |es: &EquationSystem, result_vector: &ResultVector| {
            //let mut new_result_vector: ResultVector = result_vector.clone();
            let mut new_result_vector: ResultVector = vec![DomainType::new(k); result_vector.len()];
            for pr_i in 0..pr_count {
                let mut r = es[pr_i](result_vector);
                // trace!(
                //     "Result for production {} is {}",
                //     pr_i,
                //     r.to_string(&terminals)
                // );
                new_result_vector[pr_i] = r.clone();
                // trace!(
                //     "Nt index for production {} is {}",
                //     pr_i,
                //     pr_count + nt_for_production[pr_i]
                // );
                new_result_vector[pr_count + nt_for_production[pr_i]].append(&mut r);
            }
            new_result_vector
        })
    };

    let mut result_vector = if k <= 1 {
        (0..pr_count + nt_count).fold(Vec::with_capacity(pr_count + nt_count), |mut acc, i| {
            if i < pr_count {
                acc.push(DomainType::new(k));
            } else {
                acc.push(DomainType::eps(k));
            }
            acc
        })
    } else {
        let (mut p, n) = first_cache.get(k - 1, grammar_config);
        for _ in 0..nt_count {
            p.push(DomainType::new(k));
        }
        for (nt, t) in n {
            p[pr_count + non_terminal_index(&nt)] = t;
        }
        p.drain(..).map(|t| t.set_k(k)).collect()
    };

    let mut iterations = 0usize;
    loop {
        // trace!(
        //     "\nInput\n{}",
        //     result_vector_to_string(&result_vector, pr_count, &terminals, &non_terminals)
        // );
        let new_result_vector = step_function(&equation_system, &result_vector);
        // trace!(
        //     "\nOutput\n{}",
        //     result_vector_to_string(&new_result_vector, pr_count, &terminals, &non_terminals)
        // );
        if new_result_vector == result_vector {
            // trace!("Stopping iteration");
            break;
        }
        result_vector = new_result_vector;
        iterations += 1;
        trace!("Iteration number {} completed", iterations);
    }

    let (r, n) = result_vector.split_at(pr_count);

    let k_tuples_of_nt = non_terminals.iter().enumerate().fold(
        HashMap::<String, DomainType>::new(),
        |mut acc, (ni, nt)| {
            acc.insert(nt.to_string(), n[ni].clone());
            acc
        },
    );

    (r.to_vec(), k_tuples_of_nt)
}

///
/// Creates a function that calculates the FIRST k set for the given production.
///
fn combine_production_equation<'a, 'c: 'a>(
    pr: &'c Pr,
    pr_count: usize,
    terminal_index: &'a (impl Fn(&str) -> TerminalIndex + Clone),
    non_terminal_index: &'a (impl Fn(&str) -> usize + Clone),
    k: usize,
) -> TransferFunction<'a> {
    let parts = pr
        .get_r()
        .iter()
        .fold(Vec::<SymbolString>::new(), |mut acc, s| {
            match s {
                // For each non-terminal create a separate SymbolString
                Symbol::N(_) => acc.push(SymbolString(vec![s.clone()])),
                // Stack terminals as long as possible
                Symbol::T(_) => {
                    if acc.is_empty() {
                        acc.push(SymbolString(vec![s.clone()]));
                    } else {
                        let last = acc.len() - 1;
                        let last_len = acc[last].0.len();
                        let last_terminal = &acc[last].0[last_len - 1];
                        if matches!(last_terminal, Symbol::T(_)) {
                            // Only add to terminals
                            acc[last].0.push(s.clone());
                        } else {
                            // Create a new start of terminal list
                            acc.push(SymbolString(vec![s.clone()]));
                        }
                    }
                }
            }
            acc
        });
    // trace!(
    //     "Parts: {}",
    //     parts
    //         .iter()
    //         .map(|s| format!("{}", s))
    //         .collect::<Vec<String>>()
    //         .join(", ")
    // );
    let mut result_function: TransferFunction = Box::new(move |_| DomainType::eps(k));
    // trace!(" ε");
    // For each part of the production (separated into strings of terminals and
    // single non-terminals) we have to provide a part of the equation like this:
    // Fir_k_(p) = p1 + p2 + ... + pn | + is k-concatenation; n number of parts
    // This function is build like this:
    // f(v).k_concat(fp1(v)).k_concat(fp2(v))...k_concat(fpn(v)) | v is result_vector

    for symbol_string in parts {
        // trace!(" + {}", symbol_string);
        match &symbol_string.0[0] {
            Symbol::T(_) => {
                result_function = Box::new(move |result_vector: &ResultVector| {
                    let mapper = |s| CompiledTerminal::create(s, terminal_index.clone());
                    result_function(result_vector).k_concat(
                        &DomainType::of(&[KTuple::from_slice(&symbol_string.0, mapper, k)], k),
                        k,
                    )
                });
            }
            Symbol::N(nt) => {
                let f = create_union_access_function(nt, pr_count, non_terminal_index);
                result_function = Box::new(move |result_vector: &ResultVector| {
                    result_function(result_vector).k_concat(&f(result_vector), k)
                });
            }
        }
    }
    result_function
}

///
/// Creates a function that returns the KTuple at position p from the
/// result_vector which is given as a parameter at call time.
///
/// Is used to calculate the union of KTuples of productions that belong to
/// certain non-terminal.
///
fn create_union_access_function<'a>(
    nt: &str,
    pr_count: usize,
    non_terminal_index: &'a (impl Fn(&str) -> usize + Clone),
) -> TransferFunction<'a> {
    let nt = nt.to_owned();
    let index = non_terminal_index(&nt);
    Box::new(move |result_vector: &ResultVector| {
        result_vector[pr_count + index].clone()
        // trace!(
        //     "Accessing non-terminal union of {}({}): {} (v: {:?})",
        //     nt,
        //     index,
        //     result,
        //     result_vector,
        // );
    })
}

#[allow(dead_code)]
fn result_vector_to_string(
    result_vector: &[DomainType],
    pr_count: usize,
    terminals: &[String],
    non_terminals: &[String],
) -> String {
    format!(
        "Productions:\n{}\nNon-terminals:\n{}",
        result_vector
            .iter()
            .take(pr_count)
            .enumerate()
            .map(|(i, f)| format!("{}({})", i, f.to_string(terminals)))
            .collect::<Vec<String>>()
            .join("\n"),
        result_vector
            .iter()
            .skip(pr_count)
            .enumerate()
            .map(|(i, f)| format!("{}({})", non_terminals[i], f.to_string(terminals)))
            .collect::<Vec<String>>()
            .join("\n")
    )
}
