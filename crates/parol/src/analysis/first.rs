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

#[derive(Clone)]
enum ProductionPart {
    TerminalSet(DomainType),
    NonTerminal(usize),
}

/// The equation system for the FIRST(k) calculation
type EquationSystem = Vec<Vec<ProductionPart>>;

#[inline]
fn evaluate_equation(
    equation: &[ProductionPart],
    non_terminals: &[DomainType],
    epsilon_set: &DomainType,
    k: usize,
) -> DomainType {
    let mut parts = equation.iter();
    let mut r = match parts.next() {
        None => epsilon_set.clone(),
        Some(ProductionPart::TerminalSet(ts)) => ts.clone(),
        Some(ProductionPart::NonTerminal(src_nt)) => {
            debug_assert!(*src_nt < non_terminals.len());
            non_terminals[*src_nt].clone()
        }
    };

    if !r.is_k_complete() {
        for part in parts {
            r = match part {
                ProductionPart::TerminalSet(ts) => r.k_concat(ts, k),
                ProductionPart::NonTerminal(src_nt) => {
                    debug_assert!(*src_nt < non_terminals.len());
                    r.k_concat(&non_terminals[*src_nt], k)
                }
            };
            if r.is_k_complete() {
                break;
            }
        }
    }

    r
}

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

    let mut current_non_terminals: Vec<DomainType> = if k == 0 {
        (0..nt_count).map(|_| epsilon_set.clone()).collect()
    } else {
        let last_first_set = first_cache.get(k - 1, grammar_config);
        let borrowed = last_first_set.borrow();
        debug_assert_eq!(borrowed.non_terminals.len(), nt_count);
        borrowed
            .non_terminals
            .iter()
            .map(|t| t.clone().set_k(k))
            .collect()
    };

    let mut next_non_terminals: Vec<DomainType> = vec![empty_set.clone(); nt_count];

    let mut iterations = 0usize;
    loop {
        for (equation, nt_index) in equation_system.iter().zip(nt_for_production.iter()) {
            let r = evaluate_equation(equation, &current_non_terminals, &epsilon_set, k);
            debug_assert!(*nt_index < next_non_terminals.len());
            next_non_terminals[*nt_index].union_in_place(&r);
        }

        iterations += 1;
        trace!("Iteration number {iterations} completed");

        if next_non_terminals == current_non_terminals {
            break;
        }

        std::mem::swap(&mut current_non_terminals, &mut next_non_terminals);
        for nt in &mut next_non_terminals {
            nt.clear();
        }
    }

    // Single final pass to construct productions
    let mut productions = Vec::with_capacity(pr_count);
    for equation in equation_system.iter() {
        productions.push(evaluate_equation(equation, &next_non_terminals, &epsilon_set, k));
    }

    FirstSet {
        productions,
        non_terminals: next_non_terminals,
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
                    } else if let Some(last_part) = acc.last_mut() {
                        if matches!(last_part.0.last(), Some(Symbol::T(_))) {
                            // Only add to terminals
                            last_part.0.push(s.clone());
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
