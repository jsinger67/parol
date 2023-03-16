//!
//! Grammar flow analysis
//! FOLLOW k of productions and non-terminals
//!

use crate::analysis::compiled_la_dfa::TerminalIndex;
use crate::analysis::compiled_terminal::CompiledTerminal;
use crate::analysis::FirstCache;
use crate::grammar::symbol_string::SymbolString;
use crate::{GrammarConfig, KTuple, KTuples, Pos, Pr, Symbol, TerminalKind};
use parol_runtime::lexer::FIRST_USER_TOKEN;
use parol_runtime::log::trace;
use std::collections::{BTreeMap, HashMap};
use std::sync::mpsc::channel;
use std::sync::{Arc, RwLock};
use std::thread;

/// Result type for each non-terminal:
/// The set of the follow k terminals
type DomainType = KTuples;

/// Mapping of non-terminals to KTuples
pub type FollowSet = BTreeMap<String, DomainType>;

/// The result map is applied to each iteration step.
/// It is also returned after each iteration step.
/// It maps non-terminal positions to follow sets.
type ResultMap = HashMap<Pos, DomainType>;

/// The type of the function in the equation system
/// It is called for each non-terminal
type TransferFunction = Arc<
    dyn Fn(Arc<ResultMap>, Arc<RwLock<HashMap<String, DomainType>>>) -> DomainType
        + Send
        + Sync
        + 'static,
>;

type EquationSystem = HashMap<Pos, TransferFunction>;

type StepFunction = Arc<
    dyn Fn(
        Arc<EquationSystem>,
        Arc<ResultMap>,
        Arc<HashMap<Pos, String>>,
        Arc<RwLock<HashMap<String, DomainType>>>,
    ) -> ResultMap,
>;

///
/// Calculates the FOLLOW k sets for all non-terminals of the given grammar.
///
pub fn follow_k(grammar_config: &GrammarConfig, k: usize, first_cache: &FirstCache) -> FollowSet {
    let cfg = &grammar_config.cfg;

    let terminals = grammar_config.cfg.get_ordered_terminals_owned();

    let terminal_index = Arc::new(move |t: &str, k: TerminalKind| -> usize {
        terminals
            .iter()
            .position(|(trm, kind, _)| *trm == t && kind.behaves_like(k))
            .unwrap()
            + FIRST_USER_TOKEN
    });

    let first_k_of_nt = Arc::new(first_cache.get(k, grammar_config).1);

    let start_symbol = cfg.get_start_symbol();

    let non_terminal_results: Arc<RwLock<HashMap<String, DomainType>>> = Arc::new(RwLock::new(
        cfg.get_non_terminal_set()
            .iter()
            .fold(HashMap::new(), |mut acc, nt| {
                if nt == start_symbol {
                    acc.insert(nt.to_string(), DomainType::end(k));
                } else {
                    acc.insert(nt.to_string(), DomainType::new(k));
                }
                acc
            }),
    ));

    let non_terminal_positions = Arc::new(
        cfg.get_non_terminal_positions()
            .iter()
            .filter(|(p, _)| p.sy_index() > 0)
            .fold(HashMap::<Pos, String>::new(), |mut acc, (p, s)| {
                acc.insert(*p, s.to_string());
                acc
            }),
    );

    let equation_system: EquationSystem =
        cfg.pr
            .iter()
            .enumerate()
            .fold(EquationSystem::new(), |es, (i, pr)| {
                // trace!("{}:", pr);
                update_production_equations(
                    es,
                    i,
                    pr,
                    first_k_of_nt.clone(),
                    terminal_index.clone(),
                    k,
                )
            });

    let equation_system = Arc::new(equation_system);

    let step_function: StepFunction = Arc::new(
        |es: Arc<EquationSystem>,
         result_map: Arc<ResultMap>,
         non_terminal_positions: Arc<HashMap<Pos, String>>,
         non_terminal_results: Arc<RwLock<HashMap<String, DomainType>>>| {
            let (tx, rx) = channel();
            result_map.iter().map(|(pos, _)| *pos).for_each(|pos| {
                let tx = tx.clone();
                let es = es.clone();
                let result_map = result_map.clone();
                let non_terminal_results = non_terminal_results.clone();

                // Call each function of the equation system...
                thread::spawn(move || {
                    tx.send((pos, es[&pos](result_map, non_terminal_results)))
                        .unwrap();
                });
            });

            let mut new_result_vector = ResultMap::new();
            result_map.iter().for_each(|(_, _)| {
                let (pos, pos_result) = rx.recv().unwrap();

                // ...and put the result into the new result vector.
                new_result_vector.insert(pos, pos_result.clone());

                // Also combine the result to the non_terminal_results.
                let sym = non_terminal_positions.get(&pos).unwrap();
                if let Some(set) = non_terminal_results.write().unwrap().get_mut(sym) {
                    *set = set.union(pos_result);
                }
            });
            new_result_vector
        },
    );

    let mut result_map = Arc::new(non_terminal_positions.iter().fold(
        ResultMap::new(),
        |mut acc, (p, _)| {
            acc.insert(*p, DomainType::new(k));
            acc
        },
    ));

    let mut iterations = 0usize;
    loop {
        let new_result_vector = step_function(
            equation_system.clone(),
            result_map.clone(),
            non_terminal_positions.clone(),
            non_terminal_results.clone(),
        );
        // trace!("\nStep:{}", trace_result_vector(&new_result_vector));
        if new_result_vector == *result_map {
            break;
        }
        result_map = Arc::new(new_result_vector);
        iterations += 1;
        trace!("Iteration number {} completed", iterations);
    }

    Arc::try_unwrap(non_terminal_results)
        .unwrap()
        .into_inner()
        .unwrap()
        .drain()
        .collect::<FollowSet>()
}

///
/// Creates functions that calculate the FOLLOW k sets for each occurrence of
/// a non-terminal in the given production and adds them to the equation system.
///
fn update_production_equations<T>(
    mut es: EquationSystem,
    prod_num: usize,
    pr: &Pr,
    first_k_of_nt: Arc<HashMap<String, DomainType>>,
    terminal_index: Arc<T>,
    k: usize,
) -> EquationSystem
where
    T: Fn(&str, TerminalKind) -> TerminalIndex + Clone + Send + Sync + 'static,
{
    let parts = pr.get_r().iter().enumerate().fold(
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
    // trace!(
    //     "Parts: {}",
    //     parts
    //         .iter()
    //         .map(|(i, s)| format!("{}:{}", i, s))
    //         .collect::<Vec<String>>()
    //         .join(", ")
    // );

    // For each non-terminal of the production (parts are separated into strings
    // of terminals and single non-terminals combined with the symbol-index) we
    // have to provide an equation.
    for (part_index, (symbol_index, symbol_string)) in parts.iter().enumerate() {
        // trace!(" + {}:{}", symbol_index, symbol_string);
        if let Symbol::N(..) = &symbol_string.0[0] {
            // trace!("  For non-terminal {}", nt);
            let mut result_function: TransferFunction = Arc::new(move |_, _| DomainType::eps(k));
            for (_, symbol_string) in parts.iter().skip(part_index + 1) {
                let symbol_string_clone = symbol_string.clone();
                let symbol = symbol_string_clone.0[0].clone();
                match symbol {
                    Symbol::T(_) => {
                        // trace!("  concat terminals: {}", symbol_string_clone);
                        let terminal_index = terminal_index.clone();
                        result_function =
                            Arc::new(move |result_map: Arc<ResultMap>, non_terminal_results| {
                                let mapper =
                                    |s| CompiledTerminal::create(s, terminal_index.clone());
                                result_function(result_map, non_terminal_results).k_concat(
                                    &DomainType::of(
                                        &[KTuple::from_slice_with(
                                            &symbol_string_clone.0,
                                            mapper,
                                            k,
                                        )],
                                        k,
                                    ),
                                    k,
                                )
                            });
                    }
                    Symbol::N(nt, _, _) => {
                        // trace!("  concat first k of nt: {}:{}", nt, first_of_nt);
                        let first_k_of_nt = first_k_of_nt.clone();
                        result_function =
                            Arc::new(move |result_map: Arc<ResultMap>, non_terminal_results| {
                                let first_of_nt = first_k_of_nt.get(&nt).unwrap();
                                result_function(result_map, non_terminal_results)
                                    .k_concat(first_of_nt, k)
                            });
                    }
                    Symbol::S(_) => (),
                    Symbol::Push(_) => (),
                    Symbol::Pop => (),
                }
            }
            // trace!("  concat Follow({}, {})", pr.get_n_str(), k);
            let nt = pr.get_n_str().to_string();
            es.insert(
                (prod_num, *symbol_index).into(),
                Arc::new(
                    move |result_map, non_terminal_results: Arc<RwLock<HashMap<String, DomainType>>>| {
                        result_function(result_map, non_terminal_results.clone())
                            .k_concat(non_terminal_results.read().unwrap().get(&nt).unwrap(), k)
                    },
                ),
            );
        }
    }

    es
}

#[allow(dead_code)]
fn trace_result_vector(result_map: &ResultMap) -> String {
    result_map
        .iter()
        .enumerate()
        .map(|(i, (n, f))| format!("{}({}): {}", i, n, f))
        .collect::<Vec<String>>()
        .join("\n")
}
