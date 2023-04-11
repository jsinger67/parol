//!
//! Grammar flow analysis
//! FIRSTk of productions and non-terminals
//!

use crate::analysis::compiled_la_dfa::TerminalIndex;
use crate::analysis::FirstCache;
use crate::grammar::symbol_string::SymbolString;
use crate::{CompiledTerminal, GrammarConfig, KTuple, KTuples, Pr, Symbol, TerminalKind};
use parol_runtime::log::trace;
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::thread;

/// 0: KTuples for terminals in terminal-index order
/// 1: KTuples for non-terminals in non-terminal-index (alphabetical) order
pub type FirstSet = (Vec<KTuples>, Vec<KTuples>);

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
type TransferFunction = Arc<dyn Fn(Arc<ResultVector>) -> DomainType + Send + Sync + 'static>;

type EquationSystem = Vec<TransferFunction>;

type StepFunction =
    Arc<dyn Fn(Arc<EquationSystem>, Arc<ResultVector>) -> ResultVector + Send + Sync + 'static>;

///
/// Calculates the FIRST k sets for all productions of the given grammar.
/// The indices in the returned vector correspond to the production number.
///
pub fn first_k(grammar_config: &GrammarConfig, k: usize, first_cache: &FirstCache) -> FirstSet {
    let cfg = &grammar_config.cfg;

    let pr_count = cfg.pr.len();
    let nt_count = cfg.get_non_terminal_set().len();

    // The indices returned from this function corresponds to the indices in the result-vector.
    let non_terminal_index = Arc::new(grammar_config.cfg.get_non_terminal_index_function());
    // The indices returned from this function are used to create CompiledTerminals.
    let terminal_index = Arc::new(grammar_config.cfg.get_terminal_index_function());

    let nt_for_production: Vec<usize> =
        cfg.get_non_terminal_set()
            .iter()
            .fold(vec![0; pr_count], |mut acc, nt| {
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
            .fold(Vec::with_capacity(pr_count), |mut es, pr| {
                // trace!("{}:", pr);
                es.push(combine_production_equation(
                    pr,
                    pr_count,
                    terminal_index.clone(),
                    non_terminal_index.clone(),
                    k,
                ));
                es
            });

    let equation_system = Arc::new(equation_system);

    // Heuristically tweaked
    let factor = 1;
    let max_threads: usize = num_cpus::get() * factor;

    let step_function: StepFunction = {
        // let terminals = terminals.clone();
        Arc::new(
            move |es: Arc<EquationSystem>, result_vector: Arc<ResultVector>| {
                let (tx, rx) = channel();
                let iter = &mut (0..pr_count) as &mut dyn Iterator<Item = usize>;
                let mut new_result_vector = vec![DomainType::new(k); result_vector.len()];
                loop {
                    let mut threads = 0;
                    iter.take(max_threads).for_each(|pr_i| {
                        threads += 1;
                        let tx = tx.clone();
                        let es = es.clone();
                        let result_vector = result_vector.clone();
                        thread::spawn(move || {
                            tx.send((pr_i, es[pr_i](result_vector))).unwrap();
                        });
                    });
                    (0..threads).for_each(|_| {
                        let (pr_i, r) = rx.recv().unwrap();
                        new_result_vector[pr_i] = r.clone();
                        new_result_vector[pr_count + nt_for_production[pr_i]].append(r);
                    });
                    if threads == 0 {
                        break;
                    }
                }
                new_result_vector

                // let mut new_result_vector: ResultVector =
                //     vec![DomainType::new(k); result_vector.len()];
                // for pr_i in 0..pr_count {
                //     let r = es[pr_i](result_vector.clone());
                //     // trace!(
                //     //     "Result for production {} is {}",
                //     //     pr_i,
                //     //     r.to_string(&terminals)
                //     // );
                //     new_result_vector[pr_i] = r.clone();
                //     // trace!(
                //     //     "Nt index for production {} is {}",
                //     //     pr_i,
                //     //     pr_count + nt_for_production[pr_i]
                //     // );
                //     new_result_vector[pr_count + nt_for_production[pr_i]].append(r);
                // }
                // new_result_vector
            },
        )
    };

    let mut result_vector = Arc::new(if k <= 1 {
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
        for (nt_i, t) in n.iter().enumerate() {
            p[pr_count + nt_i] = t.clone();
        }
        p.drain(..).map(|t| t.set_k(k)).collect()
    });

    let mut iterations = 0usize;
    loop {
        // trace!(
        //     "\nInput\n{}",
        //     result_vector_to_string(&result_vector, pr_count, &terminals, &non_terminals)
        // );
        let new_result_vector = Arc::new(step_function(
            equation_system.clone(),
            result_vector.clone(),
        ));
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

    let (r, k_tuples_of_nt) = result_vector.split_at(pr_count);

    (r.to_vec(), k_tuples_of_nt.to_vec())
}

///
/// Creates a function that calculates the FIRST k set for the given production.
///
fn combine_production_equation<N, T>(
    pr: &Pr,
    pr_count: usize,
    terminal_index: Arc<T>,
    non_terminal_index: Arc<N>,
    k: usize,
) -> TransferFunction
where
    T: Fn(&str, TerminalKind) -> TerminalIndex + Send + Sync + 'static,
    N: Fn(&str) -> usize + Send + 'static,
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
                Symbol::S(_) => (),
                Symbol::Push(_) => (),
                Symbol::Pop => (),
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
    let mut result_function: TransferFunction = Arc::new(move |_| DomainType::eps(k));
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
                let terminal_index = terminal_index.clone();
                result_function = Arc::new(move |result_vector: Arc<ResultVector>| {
                    let mapper = |s| CompiledTerminal::create(s, terminal_index.clone());
                    result_function(result_vector).k_concat(
                        &DomainType::of(&[KTuple::from_slice_with(&symbol_string.0, mapper, k)], k),
                        k,
                    )
                });
            }
            Symbol::N(nt, _, _) => {
                let f = create_union_access_function(nt, pr_count, non_terminal_index.clone());
                result_function = Arc::new(move |result_vector: Arc<ResultVector>| {
                    result_function(result_vector.clone()).k_concat(&f(result_vector), k)
                });
            }
            Symbol::S(_) => (),
            Symbol::Push(_) => (),
            Symbol::Pop => (),
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
fn create_union_access_function(
    nt: &str,
    pr_count: usize,
    non_terminal_index: Arc<dyn Fn(&str) -> usize>,
) -> TransferFunction {
    let nt = nt.to_owned();
    let index = non_terminal_index(&nt);
    Arc::new(move |result_vector: Arc<ResultVector>| {
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
