use crate::analysis::lookahead_dfa::ProductionIndex;
use crate::parser::parol_grammar::LookaheadExpression;
use crate::{Pos, Pr, Symbol, Terminal, TerminalKind};
use parol_runtime::once_cell::sync::Lazy;
use parol_runtime::{NonTerminalIndex, TerminalIndex};
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};
use std::collections::{HashMap, HashSet};
use std::ops::Index;

pub(crate) static RX_NUM_SUFFIX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"[0-9]+$").expect("error parsing regex"));

/// Trait to resolve terminal indices
pub trait TerminalIndexFn {
    /// Returns the terminal index for a given terminal string and terminal kind
    fn terminal_index(
        &self,
        t: &str,
        k: TerminalKind,
        l: &Option<LookaheadExpression>,
    ) -> TerminalIndex;
}

impl<F> TerminalIndexFn for F
where
    F: Fn(&str, TerminalKind, &Option<LookaheadExpression>) -> TerminalIndex,
{
    fn terminal_index(
        &self,
        t: &str,
        k: TerminalKind,
        l: &Option<LookaheadExpression>,
    ) -> TerminalIndex {
        self(t, k, l)
    }
}

/// Trait to resolve terminal indices
pub trait NonTerminalIndexFn {
    /// Returns the non-terminal index for a given non-terminal string
    fn non_terminal_index(&self, t: &str) -> NonTerminalIndex;
}

impl<F> NonTerminalIndexFn for F
where
    F: Fn(&str) -> NonTerminalIndex,
{
    fn non_terminal_index(&self, t: &str) -> NonTerminalIndex {
        self(t)
    }
}

/// The type of a primary non-terminal finder function.
/// A primary non-terminal finder function translates a terminal index into a non-terminal name
/// that is the primary non-terminal for the given terminal.
pub(crate) type FnPrimaryNonTerminalFinder = Box<dyn Fn(TerminalIndex) -> Option<String>>;

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
///
/// WrapErr free grammar type
///
#[derive(Debug, Default, Clone)]
pub struct Cfg {
    /// Start symbol of the grammar
    pub st: String,
    /// Set of productions
    pub pr: Vec<Pr>,
}

impl Cfg {
    /// Returns the start symbol of the grammar
    pub fn get_start_symbol(&self) -> &str {
        &self.st
    }

    /// Creates a new item with the given start symbol
    pub fn with_start_symbol(n: &str) -> Self {
        Self {
            st: n.to_owned(),
            ..Default::default()
        }
    }

    /// Adds a production
    pub fn add_pr(mut self, p: Pr) -> Self {
        self.pr.push(p);
        self
    }

    /// Returns the grammar position of the start symbol
    pub fn get_start_symbol_position(&self) -> Option<Pos> {
        self.pr
            .iter()
            .position(|p| p.get_n_str() == self.st)
            .map(|i| (i, 0).into())
    }

    ///
    /// Set of Non-terminals, ordered alphabetically.
    ///
    pub fn get_non_terminal_set(&self) -> BTreeSet<String> {
        let mut set = BTreeSet::new();
        set.insert(self.st.clone());
        self.pr.iter().fold(set, |mut acc, p| {
            acc.insert(p.get_n());
            acc = p.get_r().iter().fold(acc, |mut acc, s| {
                if let Symbol::N(n, ..) = s {
                    acc.insert(n.clone());
                }
                acc
            });
            acc
        })
    }

    ///
    /// Set of Non-terminals, ordered by occurrence
    /// Start symbol comes first
    ///
    pub fn get_non_terminal_ordering(&self) -> Vec<(String, Pos)> {
        let vec = vec![(
            self.st.clone(),
            self.get_start_symbol_position()
                .expect("Start symbol not found in any production"),
        )];
        self.pr.iter().enumerate().fold(vec, |mut acc, (pi, p)| {
            let pos = (pi, 0).into();
            if !acc.contains(&(p.get_n(), pos)) {
                acc.push((p.get_n(), pos));
            }
            acc = p.get_r().iter().enumerate().fold(acc, |mut acc, (si, s)| {
                let pos = (pi, si + 1).into();
                if let Symbol::N(n, ..) = s {
                    let entry = (n.clone(), pos);
                    if !acc.contains(&entry) {
                        acc.push(entry);
                    }
                }
                acc
            });
            acc
        })
    }

    ///
    /// Generates a function that returns the non-terminal index (in alphabetical sort order) for a
    /// given non-terminal name
    ///
    pub fn get_non_terminal_index_function(&self) -> impl NonTerminalIndexFn {
        let vec = self.get_non_terminal_set();
        move |nt_name: &str| vec.iter().position(|nt| nt == nt_name).unwrap()
    }

    ///
    /// Generates a function that returns the terminal index (in ordered of occurrence) for given
    /// terminal string and terminal kind
    ///
    pub fn get_terminal_index_function(&self) -> impl TerminalIndexFn + use<> {
        let vec = self
            .get_ordered_terminals_owned()
            .into_iter()
            .map(|(s, k, l, _)| (s, k, l))
            .collect::<Vec<(String, TerminalKind, Option<LookaheadExpression>)>>();
        move |t: &str, k: TerminalKind, l: &Option<LookaheadExpression>| {
            (vec.iter()
                .position(|(t0, k0, l0)| t == t0 && k.behaves_like(*k0) && l0 == l)
                .unwrap()) as TerminalIndex
                + parol_runtime::lexer::FIRST_USER_TOKEN
        }
    }

    /// Generates a function that can be used as primary_non_terminal_finder
    pub fn get_primary_non_terminal_finder(&self) -> FnPrimaryNonTerminalFinder {
        let terminal_index_finder = self.get_terminal_index_function();
        let primary_non_terminals =
            self.pr
                .iter()
                .fold(HashMap::<TerminalIndex, String>::new(), |mut acc, p| {
                    if p.1.len() == 1 {
                        if let crate::Symbol::T(Terminal::Trm(s, k, _, _, _, _, l)) = &p.1[0] {
                            let t = terminal_index_finder.terminal_index(s, *k, l);
                            acc.insert(t, p.0.get_n().unwrap());
                        }
                    }
                    acc
                });
        Box::new(move |t: TerminalIndex| primary_non_terminals.get(&t).cloned())
    }

    ///
    /// Set of Terminals - ordered by occurrence.
    /// Used for Lexer generation.
    ///
    pub fn get_ordered_terminals(
        &self,
    ) -> Vec<(&str, TerminalKind, Option<LookaheadExpression>, Vec<usize>)> {
        self.pr.iter().fold(Vec::new(), |mut acc, p| {
            acc = p.get_r().iter().fold(acc, |mut acc, s| {
                if let Symbol::T(Terminal::Trm(t, k, s, _, _, _, l)) = s {
                    // Unite the scanner states of all terminals withe the same 'behavior'
                    // The terminals are considered different if they have different lookahead
                    // expressions.
                    if let Some(pos) = acc
                        .iter_mut()
                        .position(|(trm, knd, la, _)| trm == t && knd.behaves_like(*k) && la == l)
                    {
                        for st in s {
                            if !acc[pos].3.contains(st) {
                                acc[pos].3.push(*st);
                            }
                        }
                    } else {
                        acc.push((t, *k, l.clone(), s.to_vec()));
                    }
                }
                acc
            });
            acc
        })
    }

    ///
    /// Set of Terminals - ordered by occurrence as owned values.
    ///
    pub fn get_ordered_terminals_owned(
        &self,
    ) -> Vec<(
        String,
        TerminalKind,
        Option<LookaheadExpression>,
        Vec<usize>,
    )> {
        self.get_ordered_terminals()
            .into_iter()
            .map(|(s, k, l, v)| (s.to_owned(), k, l, v))
            .collect()
    }

    ///
    /// Terminal positions within the grammar
    /// Used for Nt grammar graphs
    ///
    pub fn get_terminal_positions(&self) -> BTreeMap<Pos, &Symbol> {
        self.pr
            .iter()
            .enumerate()
            .fold(BTreeMap::new(), |mut acc, (pi, p)| {
                acc = p.get_r().iter().enumerate().fold(acc, |mut acc, (si, s)| {
                    if matches!(s, Symbol::T(Terminal::Trm(..)))
                        || matches!(s, Symbol::T(Terminal::End))
                    {
                        acc.insert(Pos::new(pi, si + 1), s);
                    }
                    acc
                });
                acc
            })
    }

    ///
    /// Non-terminal positions within the grammar
    /// Used for Nt grammar graphs
    ///
    pub fn get_non_terminal_positions(&self) -> BTreeMap<Pos, String> {
        self.pr
            .iter()
            .enumerate()
            .fold(BTreeMap::new(), |mut acc, (pi, p)| {
                acc.insert(Pos::new(pi, 0), p.get_n());
                acc = p.get_r().iter().enumerate().fold(acc, |mut acc, (si, s)| {
                    if let Symbol::N(n, ..) = s {
                        acc.insert(Pos::new(pi, si + 1), n.clone());
                    }
                    acc
                });
                acc
            })
    }

    ///
    /// Returns a vector of production references with the LHS matching the given non-terminal n
    ///
    pub fn matching_productions(&self, n: &str) -> Vec<(usize, &Pr)> {
        self.pr
            .iter()
            .enumerate()
            .fold(Vec::new(), |mut acc, (i, p)| {
                if p.get_n() == n {
                    acc.push((i, p));
                }
                acc
            })
    }

    ///
    /// Returns the number of alternatives of a given production.
    /// Used for auto generation to get a more stable generation experience.
    ///
    pub fn get_alternations_count(&self, prod_num: ProductionIndex) -> Result<usize, &'static str> {
        if prod_num >= self.pr.len() {
            Err("Invalid production number!")
        } else {
            Ok(self
                .matching_productions(self.pr[prod_num].get_n_str())
                .len())
        }
    }

    ///
    /// Returns the relative index of a production within its alternatives.
    /// Used for auto generation to get a more stable generation experience.
    ///
    pub fn get_alternation_index_of_production(
        &self,
        prod_num: ProductionIndex,
    ) -> Result<usize, &'static str> {
        if prod_num >= self.pr.len() {
            Err("Invalid production number!")
        } else {
            self.matching_productions(self.pr[prod_num].get_n_str())
                .iter()
                .position(|(i, _)| *i == prod_num)
                .ok_or("Invalid production number!")
        }
    }

    ///
    /// Calculates all nullable non-terminals.
    ///
    /// ```
    /// use parol::{Cfg, Pr, Symbol, SymbolAttribute, Terminal, TerminalKind};
    /// use std::collections::BTreeSet;
    /// use std::convert::From;
    ///
    /// macro_rules! terminal {
    ///     ($term:literal) => {Symbol::T(Terminal::Trm($term.to_string(), TerminalKind::Legacy,
    ///         vec![0], SymbolAttribute::None, None, None, None))};
    /// }
    ///
    /// let g = Cfg::with_start_symbol("S")
    ///     .add_pr(Pr::new("S", vec![Symbol::n("Y")]))
    ///     .add_pr(Pr::new("Y", vec![Symbol::n("U"), Symbol::n("Z")]))
    ///     .add_pr(Pr::new("Y", vec![Symbol::n("X"), terminal!("a")]))
    ///     .add_pr(Pr::new("Y", vec![terminal!("b")]))
    ///     .add_pr(Pr::new("U", vec![Symbol::n("V")]))
    ///     .add_pr(Pr::new("U", vec![]))
    ///     .add_pr(Pr::new("X", vec![terminal!("c")]))
    ///     .add_pr(Pr::new("V", vec![Symbol::n("V"), terminal!("d")]))
    ///     .add_pr(Pr::new("V", vec![terminal!("d")]))
    ///     .add_pr(Pr::new("Z", vec![]))
    ///     .add_pr(Pr::new("Z", vec![Symbol::n("Z"), Symbol::n("X")]));
    /// let productive = g.calculate_nullable_non_terminals();
    /// assert_eq!(
    ///     [
    ///         "S".to_owned(),
    ///         "U".to_owned(),
    ///         "Y".to_owned(),
    ///         "Z".to_owned()
    ///     ].iter().cloned().collect::<BTreeSet<String>>(),
    ///     productive);
    /// ```
    ///
    pub fn calculate_nullable_non_terminals(&self) -> BTreeSet<String> {
        fn initial_nullables(cfg: &Cfg, vars: &[String]) -> HashSet<String> {
            vars.iter()
                .filter(|v| {
                    cfg.matching_productions(v)
                        .iter()
                        .any(|(_, p)| p.is_empty())
                })
                .map(<String as Clone>::clone)
                .collect()
        }

        fn collect_nullables(cfg: &Cfg, nullables: &mut HashSet<String>, vars: &[String]) -> bool {
            fn has_nullable_alt(prods: Vec<&Pr>, nullables: &HashSet<String>) -> bool {
                fn is_already_nullable(s: &Symbol, nullables: &HashSet<String>) -> bool {
                    match s {
                        Symbol::N(n, ..) => nullables.contains(n),
                        _ => false,
                    }
                }

                prods
                    .iter()
                    .any(|p| p.get_r().iter().all(|s| is_already_nullable(s, nullables)))
            }
            let start_len = nullables.len();

            for v in vars {
                let v_prods = cfg
                    .matching_productions(v)
                    .into_iter()
                    .map(|(_, p)| p)
                    .collect();
                if has_nullable_alt(v_prods, nullables) {
                    nullables.insert(v.clone());
                }
            }

            start_len < nullables.len()
        }

        let vars = self
            .get_non_terminal_ordering()
            .into_iter()
            .map(|(n, _)| n)
            .collect::<Vec<String>>();
        let mut nullables = initial_nullables(self, &vars);

        while collect_nullables(self, &mut nullables, &vars) {}

        let mut nullables_vec = BTreeSet::new();
        for n in nullables.drain() {
            nullables_vec.insert(n);
        }
        nullables_vec
    }
}

impl Index<usize> for Cfg {
    type Output = Pr;

    fn index(&self, idx: usize) -> &Self::Output {
        &self.pr[idx]
    }
}

#[cfg(test)]
mod test {}
