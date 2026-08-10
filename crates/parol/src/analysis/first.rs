//!
//! Grammar flow analysis
//! FIRSTk of productions and non-terminals
//!

use std::rc::Rc;

use crate::analysis::FirstCache;
use crate::grammar::cfg::{NonTerminalIndexFn, TerminalIndexFn};
use crate::grammar::symbol_string::SymbolString;
use crate::{CompiledTerminal, GrammarConfig, KTuples, Pr, Symbol};
use parol_runtime::TerminalIndex;
use parol_runtime::lexer::FIRST_USER_TOKEN;
use parol_runtime::log::trace;

use super::k_tuples::KTuplesBuilder;

/// A struct to hold the FIRST k sets for all symbols of a grammar.
#[derive(Debug, Clone, Default)]
pub struct FirstSet {
    /// FIRST sets, i.e. KTuples for productions in production-index order.
    ///
    /// They are intermediate results such that each production for a non-terminal contributes to
    /// the FIRST set of the non-terminal which are combined in the non_terminal part of the result.
    pub productions: Vec<KTuples>,
    /// FIRST sets, i.e. KTuples for non-terminals in non-terminal-index (alphabetical) order
    pub non_terminals: Vec<KTuples>,
}

impl FirstSet {
    /// If this method returns true, the first set is empty.
    /// This is used for the first cache to indicate that the first set is not yet calculated.
    pub fn is_empty(&self) -> bool {
        self.productions.is_empty() && self.non_terminals.is_empty()
    }
}

/// Result type for each production:
/// The set of the first k terminals
type DomainType = KTuples;
type DomainTypeBuilder<'a> = KTuplesBuilder<'a>;

/// The result vector applied to each iteration step;
/// is also returned after each iteration step
/// The first indices correspond to the production number
/// After the Tuples for each production the Tuples for non-terminals are following.
type ResultVector = Vec<DomainType>;

#[derive(Clone)]
enum ProductionPart {
    TerminalSet(DomainType),
    NonTerminal(usize),
}

/// The equation system for the FIRST(k) calculation
type EquationSystem = Vec<Vec<ProductionPart>>;

/// The step function for the iteration
type StepFunction = Box<dyn Fn(Rc<ResultVector>) -> ResultVector>;

///
/// Calculates the FIRST(k) sets for all productions of the given grammar.
/// The indices in the returned vector correspond to the production number.
///
pub fn first_k(grammar_config: &GrammarConfig, k: usize, first_cache: &FirstCache) -> FirstSet {
    let cfg = &grammar_config.cfg;

    let pr_count = cfg.pr.len();
    let nt_count = cfg.get_non_terminal_set().len();

    // The indices returned from this function corresponds to the indices in the result-vector.
    let nti = Rc::new(grammar_config.cfg.get_non_terminal_index_function());
    // The indices returned from this function are used to create CompiledTerminals.
    let ti = Rc::new(grammar_config.cfg.get_terminal_index_function());

    let max_terminal_index = cfg.get_ordered_terminals().len() + FIRST_USER_TOKEN as usize;

    let nt_for_production: Vec<usize> =
        cfg.get_non_terminal_set()
            .iter()
            .fold(vec![0; pr_count], |mut acc, nt| {
                let non_terminal_index = nti.non_terminal_index(nt);
                for (pi, _) in cfg.matching_productions(nt) {
                    acc[pi] = non_terminal_index;
                }
                acc
            });

    let equation_system: EquationSystem =
        cfg.pr
            .iter()
            .fold(Vec::with_capacity(pr_count), |mut es, pr| {
                es.push(compile_production_equation(
                    pr,
                    Rc::clone(&ti),
                    Rc::clone(&nti),
                    k,
                    max_terminal_index,
                ));
                es
            });

    trace!(
        "Number of equations in equation system for FIRST(k) is {}",
        equation_system.len()
    );

    // Single threaded variant
    let step_function: StepFunction = {
        let empty_set = DomainTypeBuilder::new()
            .k(k)
            .max_terminal_index(max_terminal_index)
            .build()
            .unwrap();
        let epsilon_set = DomainTypeBuilder::new()
            .k(k)
            .max_terminal_index(max_terminal_index)
            .eps()
            .unwrap();

        Box::new(move |result_vector: Rc<ResultVector>| {
            let mut new_result_vector: ResultVector = vec![empty_set.clone(); result_vector.len()];
            for (pr_i, equation) in equation_system.iter().enumerate() {
                let mut r = epsilon_set.clone();
                for part in equation {
                    r = match part {
                        ProductionPart::TerminalSet(terminal_set) => r.k_concat(terminal_set, k),
                        ProductionPart::NonTerminal(nt_index) => {
                            r.k_concat(&result_vector[pr_count + nt_index], k)
                        }
                    };
                }
                new_result_vector[pr_count + nt_for_production[pr_i]].append(r.clone());
                new_result_vector[pr_i] = r;
            }
            new_result_vector
        })
    };

    // Create the initial result vector:
    // The first indices correspond to the production number.
    // After the Tuples for each production the Tuples for non-terminals are following.
    let mut result_vector = Rc::new(if k == 0 {
        // For k=0 the k-1 first set does not exist, we create an empty set
        (0..pr_count + nt_count).fold(Vec::with_capacity(pr_count + nt_count), |mut acc, i| {
            if i < pr_count {
                acc.push(
                    DomainTypeBuilder::new()
                        .k(k)
                        .max_terminal_index(max_terminal_index)
                        .build()
                        .unwrap(),
                );
            } else {
                acc.push(
                    DomainTypeBuilder::new()
                        .k(k)
                        .max_terminal_index(max_terminal_index)
                        .eps()
                        .unwrap(),
                );
            }
            acc
        })
    } else {
        // For k>0 the k-1 first set is the last first set calculated
        // We clone and modify the last first set and set the k value
        let last_first_set = first_cache.get(k - 1, grammar_config).borrow().clone();
        let mut result_vector = Vec::with_capacity(pr_count + nt_count);
        // The part for the productions
        for t in last_first_set.productions.iter() {
            result_vector.push(t.clone().set_k(k));
        }
        // The part for the non-terminals
        debug_assert_eq!(last_first_set.non_terminals.len(), nt_count);
        for t in last_first_set.non_terminals.iter() {
            result_vector.push(t.clone().set_k(k));
        }
        result_vector
    });

    let mut iterations = 0usize;
    loop {
        let new_result_vector = Rc::new(step_function(result_vector.clone()));
        if new_result_vector == result_vector {
            break;
        }
        result_vector = new_result_vector;
        iterations += 1;
        trace!("Iteration number {iterations} completed");
    }

    let (r, k_tuples_of_nt) = result_vector.split_at(pr_count);

    FirstSet {
        productions: r.to_vec(),
        non_terminals: k_tuples_of_nt.to_vec(),
    }
}

///
/// Compiles a production equation into reusable parts for fast iterative evaluation.
///
fn compile_production_equation<N, T>(
    pr: &Pr,
    ti_fn: Rc<T>,
    nti_fn: Rc<N>,
    k: usize,
    max_terminal_index: usize,
) -> Vec<ProductionPart>
where
    T: TerminalIndexFn,
    N: NonTerminalIndexFn,
{
    let parts = pr
        .get_r()
        .iter()
        .fold(Vec::<SymbolString>::new(), |mut acc, s| {
            match s {
                // For each non-terminal create a separate SymbolString
                Symbol::N(..) => acc.push(SymbolString(vec![s.clone()])),
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
                _ => {
                    unreachable!(
                        "Scanner switching directives have been removed from the grammar syntax."
                    );
                }
            }
            acc
        });

    let mut equation = Vec::with_capacity(parts.len());
    for symbol_string in parts {
        match &symbol_string.0[0] {
            Symbol::T(_) => {
                let terminal_indices: Vec<TerminalIndex> = symbol_string
                    .0
                    .iter()
                    .map(|s| CompiledTerminal::create(s, Rc::clone(&ti_fn)).0)
                    .collect();
                let terminal_set = DomainTypeBuilder::new()
                    .k(k)
                    .max_terminal_index(max_terminal_index)
                    .terminal_indices(&[&terminal_indices])
                    .build()
                    .unwrap();
                equation.push(ProductionPart::TerminalSet(terminal_set));
            }
            Symbol::N(nt, _, _, _) => {
                equation.push(ProductionPart::NonTerminal(nti_fn.non_terminal_index(nt)));
            }
            _ => {
                unreachable!(
                    "Scanner switching directives have been removed from the grammar syntax."
                );
            }
        }
    }
    equation
}
