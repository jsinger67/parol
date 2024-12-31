//!
//! Grammar flow analysis
//! FOLLOW k of productions and non-terminals
//!

use super::k_tuples::KTuplesBuilder;
use super::{FirstSet, FollowCache};
use crate::analysis::compiled_terminal::CompiledTerminal;
use crate::analysis::FirstCache;
use crate::grammar::cfg::{NonTerminalIndexFn, TerminalIndexFn};
use crate::grammar::symbol_string::SymbolString;
use crate::{GrammarConfig, KTuples, Pos, Pr, Symbol};
use parol_runtime::lexer::FIRST_USER_TOKEN;
use parol_runtime::log::trace;
use parol_runtime::TerminalIndex;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Result type for each non-terminal:
/// The set of the follow k terminals
type DomainType = KTuples;
type DomainTypeBuilder<'a> = KTuplesBuilder<'a>;

/// A struct to hold the FOLLOW sets for non-terminals in non-terminal-index (alphabetical) order
#[derive(Debug, Clone, Default)]
pub struct FollowSet {
    /// The FOLLOW sets, i.e. KTuples for each non-terminal
    pub non_terminals: Vec<DomainType>,
}

impl FollowSet {
    /// Creates a new instance of the FollowSet struct from a vector of DomainType
    pub fn new(non_terminals: Vec<DomainType>) -> Self {
        FollowSet { non_terminals }
    }

    /// If this method returns true, the follow set is empty.
    /// This is used for the follow cache to indicate that the follow set is not yet calculated.
    pub fn is_empty(&self) -> bool {
        self.non_terminals.is_empty()
    }
}

/// The result map is applied to each iteration step.
/// It is also returned after each iteration step.
/// It maps non-terminal positions to follow sets.
pub(crate) type ResultMap = HashMap<Pos, DomainType>;

/// The type of the function in the equation system
/// It is called for each non-terminal
type TransferFunction = Box<dyn Fn(Rc<ResultMap>, Rc<RefCell<FollowSet>>) -> DomainType>;

type EquationSystem = HashMap<Pos, TransferFunction>;

/// # [`StepFunction`] Documentation
/// The StepFunction type is a type alias for an `Rc` of a `dyn Fn`  that takes four parameters and
/// returns a `ResultMap'.
/// This function is called in each step of the iteration process until the results (the result map
/// and the non-terminal vector) don't change anymore.
///
/// ## Parameters
///   * `result_map: Rc<ResultMap>` - An `Rc` of a `ResultMap` struct.
///     This is the actual input for each iteration generated by the previous iteration.
///     It is wrapped in an `Rc` to make it read accessible from multiple threads.
///   * `non_terminal_positions: Rc<HashMap<Pos, usize>>` - An `Rc` of a `HashMap` of `Pos` and `usize`.
///     This is the association of non-terminal positions to non-terminal indices in the
///     non-terminal vector (the fourth parameter of this function) and is used to find the correct
///     place where the non-terminal result has to be accumulated.
///   * `non_terminal_results: Rc<RefCell<FollowSet>>` - An `Rc` of a `RefCell` of a `Vec` of `DomainType`.
///     This is the actual value returned by the [follow_k] function and is amended in each
///     iteration step by combining all results for all position of a certain non-terminal into a
///     single result (a k-tuple, i.e. a trie of terminal strings).
/// ## Return Value
/// The `StepFunction` returns a `ResultMap` struct that was extended in each iteration step.
type StepFunction =
    Rc<dyn Fn(Rc<ResultMap>, Rc<HashMap<Pos, usize>>, Rc<RefCell<FollowSet>>) -> ResultMap>;

///
/// Calculates the FOLLOW k sets for all non-terminals of the given grammar.
///
pub fn follow_k(
    grammar_config: &GrammarConfig,
    k: usize,
    first_cache: &FirstCache,
    follow_cache: &FollowCache,
) -> (ResultMap, FollowSet) {
    let cfg = &grammar_config.cfg;

    let terminals = grammar_config.cfg.get_ordered_terminals_owned();

    let max_terminal_index = terminals.len() + FIRST_USER_TOKEN as usize;

    let ti = Rc::new(grammar_config.cfg.get_terminal_index_function());

    let first_k_of_nt = first_cache.get(k, grammar_config);

    let start_symbol = cfg.get_start_symbol();

    let nti = Rc::new(cfg.get_non_terminal_index_function());

    let non_terminal_positions = Rc::new(
        cfg.get_non_terminal_positions()
            .iter()
            .filter(|(p, _)| p.sy_index() > 0)
            .fold(HashMap::<Pos, usize>::new(), |mut acc, (p, s)| {
                acc.insert(*p, nti.non_terminal_index(s));
                acc
            }),
    );

    let equation_system: EquationSystem =
        cfg.pr
            .iter()
            .enumerate()
            .fold(EquationSystem::new(), |es, (i, pr)| {
                let args = UpdateProductionEquationsArgs {
                    prod_num: i,
                    pr,
                    first_k_of_nt: first_k_of_nt.clone(),
                    ti: Rc::clone(&ti),
                    nti: Rc::clone(&nti),
                    k,
                    max_terminal_index,
                };
                update_production_equations(es, args)
            });

    trace!(
        "Number of equations in equation system for FOLLOW(k) is {}",
        equation_system.len()
    );

    let step_function: StepFunction = Rc::new(
        move |result_map: Rc<ResultMap>,
              non_terminal_positions: Rc<HashMap<Pos, usize>>,
              non_terminal_results: Rc<RefCell<FollowSet>>| {
            let mut new_result_vector = ResultMap::new();

            result_map.iter().for_each(|(pos, _)| {
                // Call each function of the equation system
                let pos_result =
                    equation_system[pos](Rc::clone(&result_map), Rc::clone(&non_terminal_results));

                // Combine the result to the non_terminal_results.
                let sym = non_terminal_positions.get(pos).unwrap();
                if let Some(set) = non_terminal_results
                    .borrow_mut()
                    .non_terminals
                    .get_mut(*sym)
                {
                    *set = set.union(&pos_result).0;
                }

                // And put the result into the new result vector.
                new_result_vector.insert(*pos, pos_result);
            });
            new_result_vector
        },
    );

    let non_terminal_results = Rc::new(RefCell::new(FollowSet::new(
        cfg.get_non_terminal_set()
            .iter()
            .fold(Vec::new(), |mut acc, nt| {
                if nt == start_symbol {
                    acc.push(
                        DomainTypeBuilder::new()
                            .k(k)
                            .max_terminal_index(max_terminal_index)
                            .end()
                            .unwrap(),
                    );
                } else {
                    acc.push(
                        DomainTypeBuilder::new()
                            .k(k)
                            .max_terminal_index(max_terminal_index)
                            .build()
                            .unwrap(),
                    );
                }
                acc
            }),
    )));

    let mut result_map = if k == 0 {
        // k == 0: No previous cache result available
        Rc::new(
            non_terminal_positions
                .iter()
                .fold(ResultMap::new(), |mut acc, (p, _)| {
                    acc.insert(
                        *p,
                        DomainTypeBuilder::new()
                            .k(k)
                            .max_terminal_index(max_terminal_index)
                            .build()
                            .unwrap(),
                    );
                    acc
                }),
        )
    } else {
        let cached = follow_cache
            .get(k - 1, grammar_config, first_cache)
            .borrow()
            .last_result
            .iter()
            .map(|(p, t)| (*p, t.clone().set_k(k)))
            .collect();
        Rc::new(cached)
    };

    let mut iterations = 0usize;
    let mut new_result_vector;
    loop {
        new_result_vector = step_function(
            Rc::clone(&result_map),
            Rc::clone(&non_terminal_positions),
            Rc::clone(&non_terminal_results),
        );
        if new_result_vector == *result_map {
            break;
        }
        result_map = Rc::new(new_result_vector);
        iterations += 1;
        trace!("Iteration number {} completed", iterations);
    }

    (
        new_result_vector,
        Rc::try_unwrap(non_terminal_results).unwrap().into_inner(),
    )
}

/// Arguments for the update_production_equations function
struct UpdateProductionEquationsArgs<'a, T, N> {
    /// The production number
    prod_num: usize,
    /// The production
    pr: &'a Pr,
    /// The FIRST(k) sets of non-terminals
    first_k_of_nt: Rc<RefCell<FirstSet>>,
    /// The terminal index function
    ti: Rc<T>,
    /// The non-terminal index function
    nti: Rc<N>,
    /// The k value
    k: usize,
    /// The maximum terminal index
    max_terminal_index: usize,
}

///
/// Creates functions that calculate the FOLLOW k sets for each occurrence of
/// a non-terminal in the given production and adds them to the equation system.
///
fn update_production_equations<T, N>(
    mut es: EquationSystem,
    args: UpdateProductionEquationsArgs<T, N>,
) -> EquationSystem
where
    T: TerminalIndexFn + 'static,
    N: NonTerminalIndexFn,
{
    let parts = args.pr.get_r().iter().enumerate().fold(
        Vec::<(usize, SymbolString)>::new(),
        |mut acc, (i, s)| {
            match s {
                // For each non-terminal create a separate SymbolString
                Symbol::N(..) => acc.push((i + 1, SymbolString(vec![s.clone()]))),
                // Stack terminals as long as possible
                Symbol::T(_) => {
                    if acc.is_empty() {
                        acc.push((i + 1, SymbolString(vec![s.clone()])));
                    } else {
                        let last = acc.len() - 1;
                        let last_len = acc[last].1.len();
                        let last_terminal = &acc[last].1 .0[last_len - 1];
                        if matches!(last_terminal, Symbol::T(_)) {
                            // Only add to terminals
                            acc[last].1 .0.push(s.clone());
                        } else {
                            // Create a new start of terminal list
                            acc.push((i + 1, SymbolString(vec![s.clone()])));
                        }
                    }
                }
                Symbol::S(_) => (),
                Symbol::Push(_) => (),
                Symbol::Pop => (),
            }
            acc
        },
    );

    // For each non-terminal of the production (parts are separated into strings
    // of terminals and single non-terminals combined with the symbol-index) we
    // have to provide an equation.
    for (part_index, (symbol_index, symbol_string)) in parts.iter().enumerate() {
        if let Symbol::N(..) = &symbol_string.0[0] {
            let mut result_function: TransferFunction = Box::new(move |_, _| {
                DomainTypeBuilder::new()
                    .k(args.k)
                    .max_terminal_index(args.max_terminal_index)
                    .eps()
                    .unwrap()
            });
            for (_, symbol_string) in parts.iter().skip(part_index + 1) {
                let symbol_string_clone = symbol_string.clone();
                let symbol = symbol_string_clone.0[0].clone();
                match symbol {
                    Symbol::T(_) => {
                        let ti = args.ti.clone();
                        result_function =
                            Box::new(move |result_map: Rc<ResultMap>, non_terminal_results| {
                                let mapper = |s| CompiledTerminal::create(s, Rc::clone(&ti));
                                let terminal_indices: Vec<TerminalIndex> =
                                    symbol_string_clone.0.iter().map(|s| mapper(s).0).collect();
                                result_function(result_map, non_terminal_results).k_concat(
                                    &DomainTypeBuilder::new()
                                        .k(args.k)
                                        .max_terminal_index(args.max_terminal_index)
                                        .clone()
                                        .terminal_indices(&[&terminal_indices])
                                        .build()
                                        .unwrap(),
                                    args.k,
                                )
                            });
                    }
                    Symbol::N(nt, _, _) => {
                        let first_k_of_nt = args.first_k_of_nt.clone();
                        let nt_i = args.nti.non_terminal_index(&nt);
                        result_function =
                            Box::new(move |result_map: Rc<ResultMap>, non_terminal_results| {
                                let borrowed_first_of_nt = first_k_of_nt.borrow();
                                let first_of_nt =
                                    borrowed_first_of_nt.non_terminals.get(nt_i).unwrap();
                                result_function(result_map, non_terminal_results)
                                    .k_concat(first_of_nt, args.k)
                            });
                    }
                    Symbol::S(_) => (),
                    Symbol::Push(_) => (),
                    Symbol::Pop => (),
                }
            }
            let nt = args.nti.non_terminal_index(args.pr.get_n_str());
            es.insert(
                (args.prod_num, *symbol_index).into(),
                Box::new(
                    move |result_map, non_terminal_results: Rc<RefCell<FollowSet>>| {
                        result_function(result_map, non_terminal_results.clone()).k_concat(
                            non_terminal_results.borrow().non_terminals.get(nt).unwrap(),
                            args.k,
                        )
                    },
                ),
            );
        }
    }

    es
}
