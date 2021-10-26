use crate::analysis::lookahead_dfa::ProductionIndex;
use crate::analysis::LookaheadDFA;
use crate::analysis::{first_k, follow_k, FirstSet, FollowSet};
use crate::errors::*;
use crate::{GrammarConfig, KTuples};
use log::trace;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

pub struct FirstCache(pub Rc<RefCell<HashMap<usize, FirstSet>>>);
pub struct FollowCache(pub Rc<RefCell<HashMap<usize, FollowSet>>>);

impl FirstCache {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(HashMap::new())))
    }
    pub fn get(&self, k: usize, grammar_config: &GrammarConfig) -> FirstSet {
        let exists = {
            let borrowed_entry = self.0.borrow();
            borrowed_entry.get(&k).is_some()
        };
        if exists {
            trace!("FirstCache::get: reusing first set for k={}", k);
            self.0.borrow().get(&k).unwrap().clone()
        } else {
            trace!("FirstCache::get: calculating first set for k={}...", k);
            let entry = first_k(grammar_config, k, self);
            trace!("finished");
            self.0.borrow_mut().insert(k, entry);
            self.get(k, grammar_config)
        }
    }
}

impl Default for FirstCache {
    fn default() -> Self {
        Self::new()
    }
}

impl FollowCache {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(HashMap::new())))
    }
    pub fn get(
        &self,
        k: usize,
        grammar_config: &GrammarConfig,
        first_cache: &FirstCache,
    ) -> FollowSet {
        let exists = {
            let borrowed_entry = self.0.borrow();
            borrowed_entry.get(&k).is_some()
        };
        if exists {
            trace!("FollowCache::get: reusing follow set for k={}", k);
            self.0.borrow().get(&k).unwrap().clone()
        } else {
            trace!("FollowCache::get: calculating follow set for k={}...", k);
            let entry = follow_k(grammar_config, k, first_cache);
            trace!("finished");
            self.0.borrow_mut().insert(k, entry);
            self.get(k, grammar_config, first_cache)
        }
    }
}

impl Default for FollowCache {
    fn default() -> Self {
        Self::new()
    }
}

///
/// Calculates if for a certain non-terminal of grammar cfg the production to
/// use can be determined deterministically with at maximum max_k lookahead.
/// To accomplish this, for all productions of the given non-terminal k-tuples
/// of at most length k are generated, starting with k=1.
/// If all k-tuples are distinct between all productions the number k is
/// returned. Otherwise the value of k is incremented by 1 and the process is
/// retried.
/// If k_max is exceeded the function returns an error.
///
pub fn decidable(
    grammar_config: &GrammarConfig,
    non_terminal: &str,
    max_k: usize,
    first_cache: &FirstCache,
    follow_cache: &FollowCache,
) -> Result<usize> {
    let cfg = &grammar_config.cfg;
    let productions = cfg.matching_productions(non_terminal);
    if productions.is_empty() {
        Err("The given non-terminal isn't part of the given grammar!".into())
    } else if productions.len() == 1 {
        // The trivial case - no lookahead is needed.
        Ok(0)
    } else {
        let mut current_k = 1;
        loop {
            if current_k > max_k {
                break;
            }
            let productions = cfg.matching_productions(non_terminal);
            let k_tuples_of_productions = productions
                .iter()
                .map(|(pi, _)| {
                    let k_tuples = first_cache.get(current_k, grammar_config).0[*pi].clone();
                    (*pi, k_tuples)
                })
                .collect::<Vec<(ProductionIndex, KTuples)>>();

            let cached = follow_cache.get(current_k, grammar_config, first_cache);
            if let Some(follow_set) = cached.get(non_terminal) {
                let concatenated_k_tuples = k_tuples_of_productions
                    .iter()
                    .map(|(i, t)| (*i, t.clone().k_concat(follow_set, current_k)))
                    .collect::<Vec<(ProductionIndex, KTuples)>>();

                if concatenated_k_tuples.iter().all(|(i, t1)| {
                    concatenated_k_tuples
                        .iter()
                        .all(|(j, t2)| i == j || t1.is_disjoint(t2))
                }) {
                    return Ok(current_k);
                }
            } else {
                return Err("Internal error".into());
            }
            current_k += 1;
        }
        Err("max_k exceeded".into())
    }
}

///
/// Calculates maximum lookahead size where max_k is the limit.
///
pub fn calculate_k(
    grammar_config: &GrammarConfig,
    max_k: usize,
    first_cache: &FirstCache,
    follow_cache: &FollowCache,
) -> Result<usize> {
    let cfg = &grammar_config.cfg;
    cfg.get_non_terminal_set()
        .iter()
        .map(|n| decidable(grammar_config, n, max_k, first_cache, follow_cache))
        .fold(Ok(0), |k, r| match (&k, &r) {
            (Err(_), _) => k, // The first error is retained
            (Ok(max_k), Ok(current_k)) => Ok(std::cmp::max(*max_k, *current_k)),
            (Ok(_), Err(_)) => r, // The first error occurred here
        })
}

///
/// Calculates lookahead tuples for all productions, where max_k is the limit.
///
pub fn calculate_k_tuples(
    grammar_config: &GrammarConfig,
    max_k: usize,
    first_cache: &FirstCache,
    follow_cache: &FollowCache,
) -> Result<BTreeMap<usize, KTuples>> {
    let cfg = &grammar_config.cfg;
    cfg.get_non_terminal_set()
        .iter()
        .map(|n| {
            (
                n.clone(),
                decidable(grammar_config, n, max_k, first_cache, follow_cache),
            )
        })
        .fold(Ok(BTreeMap::new()), |acc, (n, r)| match (acc, r) {
            (Err(e), _) => Err(e),
            (Ok(mut m), Ok(k)) => {
                let productions = cfg.matching_productions(&n);
                let mut k_tuples = productions
                    .iter()
                    .fold(BTreeMap::new(), |mut acc, (pi, _)| {
                        let k_tuples = first_cache.get(k, grammar_config).0[*pi].clone();
                        let cached = follow_cache.get(k, grammar_config, first_cache);
                        if let Some(follow_set) = cached.get(&n) {
                            acc.insert(*pi, k_tuples.k_concat(follow_set, k));
                        }
                        acc
                    });
                m.append(&mut k_tuples);
                Ok(m)
            }
            (_, Err(e)) => Err(e.description().into()),
        })
}

///
/// Calculates lookahead DFAs for all non-terminals, where k is the limit.
///
pub fn calculate_lookahead_dfas(
    grammar_config: &GrammarConfig,
    max_k: usize,
) -> Result<BTreeMap<String, LookaheadDFA>> {
    let cfg = &grammar_config.cfg;

    let first_cache = FirstCache::new();
    let follow_cache = FollowCache::new();

    let k_tuples_of_productions =
        calculate_k_tuples(grammar_config, max_k, &first_cache, &follow_cache)?;
    k_tuples_of_productions
        .iter()
        .fold(Ok(BTreeMap::new()), |acc, (i, t)| {
            if let Ok(mut acc) = acc {
                let nt = cfg[*i].get_n();
                let dfa = LookaheadDFA::from_k_tuples(t, *i);
                if let Some(found_dfa) = acc.remove(&nt) {
                    let united_dfa = found_dfa.unite(&dfa)?;
                    acc.insert(nt, united_dfa);
                } else {
                    acc.insert(nt, dfa);
                }
                Ok(acc)
            } else {
                acc
            }
        })
}

///
/// Returns conflicts for a given non-terminal at given lookahead size.
///
pub fn explain_conflicts(
    grammar_config: &GrammarConfig,
    non_terminal: &str,
    k: usize,
    first_cache: &FirstCache,
    follow_cache: &FollowCache,
) -> Result<Vec<(ProductionIndex, KTuples, ProductionIndex, KTuples)>> {
    let cfg = &grammar_config.cfg;
    let productions = cfg.matching_productions(non_terminal);
    if productions.is_empty() {
        Err("The given non-terminal isn't part of the given grammar!".into())
    } else if productions.len() == 1 {
        // The trivial case - no lookahead is needed, no conflicts can occur.
        Ok(Vec::new())
    } else {
        let productions = cfg.matching_productions(non_terminal);
        let k_tuples_of_productions = productions
            .iter()
            .map(|(pi, _)| {
                let k_tuples = first_cache.get(k, grammar_config).0[*pi].clone();
                (*pi, k_tuples)
            })
            .collect::<Vec<(ProductionIndex, KTuples)>>();

        let cached = follow_cache.get(k, grammar_config, first_cache);
        if let Some(follow_set) = cached.get(non_terminal) {
            let concatenated_k_tuples = k_tuples_of_productions
                .iter()
                .map(|(i, t)| (*i, t.clone().k_concat(follow_set, k)))
                .collect::<Vec<(ProductionIndex, KTuples)>>();
            let mut conflicting_k_tuples = Vec::new();
            for (i, ki) in &concatenated_k_tuples {
                for (j, kj) in &concatenated_k_tuples {
                    if i != j
                        && !ki.is_disjoint(kj)
                        && !conflicting_k_tuples
                            .iter()
                            .any(|(p1, _, p2, _)| p1 != i && p2 != j)
                    {
                        conflicting_k_tuples.push((*i, ki.clone(), *j, kj.clone()));
                    }
                }
            }
            return Ok(conflicting_k_tuples);
        }
        Err("Internal error".into())
    }
}

#[cfg(test)]
mod test {
    use super::{calculate_k, decidable, FirstCache, FollowCache};
    use crate::{Cfg, GrammarConfig, Pr, Symbol};

    #[test]
    fn check_decidable() {
        let cfg = Cfg::with_start_symbol("S")
            .add_pr(Pr::new("S", vec![Symbol::t("a"), Symbol::n("X")]))
            .add_pr(Pr::new("X", vec![Symbol::t("b"), Symbol::n("S")]))
            .add_pr(Pr::new(
                "X",
                vec![
                    Symbol::t("a"),
                    Symbol::n("Y"),
                    Symbol::t("b"),
                    Symbol::n("Y"),
                ],
            ))
            .add_pr(Pr::new("Y", vec![Symbol::t("b"), Symbol::t("a")]))
            .add_pr(Pr::new("Y", vec![Symbol::t("a"), Symbol::n("Z")]))
            .add_pr(Pr::new(
                "Z",
                vec![Symbol::t("a"), Symbol::n("Z"), Symbol::n("X")],
            ));
        let grammar_config = GrammarConfig::new(cfg, 5);
        let first_cache = FirstCache::new();
        let follow_cache = FollowCache::new();
        let result = decidable(&grammar_config, "S", 5, &first_cache, &follow_cache).unwrap();
        assert_eq!(0, result);
        let result = decidable(&grammar_config, "X", 5, &first_cache, &follow_cache).unwrap();
        assert_eq!(1, result);
        let result = decidable(&grammar_config, "Y", 5, &first_cache, &follow_cache).unwrap();
        assert_eq!(1, result);
        let result = decidable(&grammar_config, "Z", 5, &first_cache, &follow_cache).unwrap();
        assert_eq!(0, result);
        assert_eq!(
            "The given non-terminal isn't part of the given grammar!",
            decidable(&grammar_config, "A", 5, &first_cache, &follow_cache)
                .err()
                .unwrap()
                .description()
        );
    }

    #[test]
    fn check_calculate_k() {
        let cfg = Cfg::with_start_symbol("S")
            .add_pr(Pr::new("S", vec![Symbol::t("a"), Symbol::n("X")]))
            .add_pr(Pr::new("X", vec![Symbol::t("b"), Symbol::n("S")]))
            .add_pr(Pr::new(
                "X",
                vec![
                    Symbol::t("a"),
                    Symbol::n("Y"),
                    Symbol::t("b"),
                    Symbol::n("Y"),
                ],
            ))
            .add_pr(Pr::new("Y", vec![Symbol::t("b"), Symbol::t("a")]))
            .add_pr(Pr::new("Y", vec![Symbol::t("a"), Symbol::n("Z")]))
            .add_pr(Pr::new(
                "Z",
                vec![Symbol::t("a"), Symbol::n("Z"), Symbol::n("X")],
            ));
        let grammar_config = GrammarConfig::new(cfg, 5);
        let first_cache = FirstCache::new();
        let follow_cache = FollowCache::new();
        let result = calculate_k(&grammar_config, 5, &first_cache, &follow_cache).unwrap();
        assert_eq!(1, result);
    }
}
