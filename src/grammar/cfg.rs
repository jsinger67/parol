use crate::{Pos, Pr, Symbol, Terminal};
use regex::Regex;
use std::collections::HashSet;
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Index;

lazy_static! {
    pub(crate) static ref RX_NUM_SUFFIX: Regex = Regex::new(r"\d+$").expect("error parsing regex");
}

///
/// Context free grammar type
///
#[derive(Debug, Default, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Cfg {
    /// Start symbol of the grammar
    pub st: String,
    /// Set of productions
    pub pr: Vec<Pr>,
}

impl Cfg {
    pub fn get_start_symbol(&self) -> &str {
        &self.st
    }

    pub fn with_start_symbol(n: &str) -> Self {
        Self {
            st: n.to_owned(),
            ..Default::default()
        }
    }

    pub fn add_pr(mut self, p: Pr) -> Self {
        self.pr.push(p);
        self
    }

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
                if let Symbol::N(n) = s {
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
                if let Symbol::N(n) = s {
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
    /// Set of Terminals - ordered by occurrence.
    /// Used for Lexer generation.
    ///
    pub fn get_ordered_terminals(&self) -> Vec<(&str, Vec<usize>)> {
        self.pr.iter().fold(Vec::new(), |mut acc, p| {
            acc = p.get_r().iter().fold(acc, |mut acc, s| {
                if let Symbol::T(Terminal::Trm(t, s)) = s {
                    if let Some(pos) = acc.iter_mut().position(|(trm, _)| trm == t) {
                        for st in s {
                            if !acc[pos].1.contains(st) {
                                acc[pos].1.push(*st);
                            }
                        }
                    } else {
                        acc.push((t, s.to_vec()));
                    }
                }
                acc
            });
            acc
        })
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
                    if matches!(s, Symbol::T(Terminal::Trm(_, _)))
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
                    if let Symbol::N(n) = s {
                        acc.insert(Pos::new(pi, si + 1), n.clone());
                    }
                    acc
                });
                acc
            })
    }

    ///
    /// Calculates all productive non-terminals.
    ///
    /// ```
    /// use parol::{Cfg, Pr, Symbol};
    /// use std::collections::BTreeSet;
    /// let mut g = Cfg::default();
    /// assert!(g.productive_non_terminals().is_empty());
    ///
    /// let g = Cfg::with_start_symbol("S'")
    ///     .add_pr(Pr::new("S'", vec![Symbol::n("S")]))
    ///     .add_pr(Pr::new("S", vec![Symbol::t("a", vec![0]), Symbol::n("X")]))
    ///     .add_pr(Pr::new("X", vec![Symbol::t("b", vec![0]), Symbol::n("S")]))
    ///     .add_pr(Pr::new("X", vec![Symbol::t("a", vec![0]), Symbol::n("Y"), Symbol::t("b", vec![0]), Symbol::n("Y")]))
    ///     .add_pr(Pr::new("Y", vec![Symbol::t("b", vec![0]), Symbol::t("a", vec![0])]))
    ///     .add_pr(Pr::new("Y", vec![Symbol::t("a", vec![0]), Symbol::n("Z")]))
    ///     .add_pr(Pr::new("Z", vec![Symbol::t("a", vec![0]), Symbol::n("Z"), Symbol::n("X")]));
    /// let productive = g.productive_non_terminals();
    /// assert_eq!(["S'".to_owned(), "S".to_owned(), "X".to_owned(), "Y".to_owned()].iter().cloned().collect::<BTreeSet<String>>(), productive);
    /// ```
    ///
    pub fn productive_non_terminals(&self) -> BTreeSet<String> {
        fn insert_productive(
            pr: &[Pr],
            mut productive_so_far: BTreeSet<String>,
        ) -> BTreeSet<String> {
            for p in pr {
                if p.get_r().iter().all(|s| match s {
                    Symbol::T(Terminal::Trm(_, _)) => true,
                    Symbol::N(n) => productive_so_far.contains(n),
                    _ => panic!("Unexpected symbol kind!"),
                }) {
                    productive_so_far.insert(p.get_n().clone());
                }
            }
            productive_so_far
        }

        let mut productive: BTreeSet<String> =
            self.pr.iter().fold(BTreeSet::new(), |mut acc, p| {
                if p.get_r()
                    .iter()
                    .all(|s| matches!(s, Symbol::T(Terminal::Trm(_, _))))
                {
                    acc.insert(p.get_n());
                }
                acc
            });

        let mut current_size = 0;
        while current_size < productive.len() {
            current_size = productive.len();
            productive = insert_productive(&self.pr, productive);
        }
        productive
    }

    ///
    /// Calculates all non-productive non-terminals.
    ///
    /// ```
    /// use parol::{Cfg, Pr, Symbol};
    /// use std::collections::BTreeSet;
    /// use std::convert::From;
    /// let mut g = Cfg::default();
    /// assert!(g.productive_non_terminals().is_empty());
    ///
    /// let g = Cfg::with_start_symbol("S'")
    ///     .add_pr(Pr::new("S'", vec![Symbol::n("S")]))
    ///     .add_pr(Pr::new("S", vec![Symbol::t("a", vec![0]), Symbol::n("X")]))
    ///     .add_pr(Pr::new("X", vec![Symbol::t("b", vec![0]), Symbol::n("S")]))
    ///     .add_pr(Pr::new("X", vec![Symbol::t("a", vec![0]), Symbol::n("Y"), Symbol::t("b", vec![0]), Symbol::n("Y")]))
    ///     .add_pr(Pr::new("Y", vec![Symbol::t("b", vec![0]), Symbol::t("a", vec![0])]))
    ///     .add_pr(Pr::new("Y", vec![Symbol::t("a", vec![0]), Symbol::n("Z")]))
    ///     .add_pr(Pr::new("Z", vec![Symbol::t("a", vec![0]), Symbol::n("Z"), Symbol::n("X")]));
    /// let productive = g.unproductive_non_terminals();
    /// assert_eq!(["Z".to_owned()].iter().cloned().collect::<BTreeSet<String>>(), productive);
    /// ```
    ///
    pub fn unproductive_non_terminals(&self) -> BTreeSet<String> {
        self.get_non_terminal_set()
            .difference(&self.productive_non_terminals())
            .cloned()
            .collect()
    }

    ///
    /// Detects whether all non-terminals are productive
    ///
    pub fn all_non_terminals_productive(&self) -> bool {
        let all_nts = self.get_non_terminal_set();
        let productive = self.productive_non_terminals();
        all_nts == productive
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
    /// Calculates all nullable non-terminals.
    ///
    /// ```
    /// use parol::{Cfg, Pr, Symbol};
    /// use std::collections::BTreeSet;
    /// use std::convert::From;
    ///
    /// let g = Cfg::with_start_symbol("S")
    ///     .add_pr(Pr::new("S", vec![Symbol::n("Y")]))
    ///     .add_pr(Pr::new("Y", vec![Symbol::n("U"), Symbol::n("Z")]))
    ///     .add_pr(Pr::new("Y", vec![Symbol::n("X"), Symbol::t("a", vec![0])]))
    ///     .add_pr(Pr::new("Y", vec![Symbol::t("b", vec![0])]))
    ///     .add_pr(Pr::new("U", vec![Symbol::n("V")]))
    ///     .add_pr(Pr::new("U", vec![]))
    ///     .add_pr(Pr::new("X", vec![Symbol::t("c", vec![0])]))
    ///     .add_pr(Pr::new("V", vec![Symbol::n("V"), Symbol::t("d", vec![0])]))
    ///     .add_pr(Pr::new("V", vec![Symbol::t("d", vec![0])]))
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
                        Symbol::N(n) => nullables.contains(n),
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

    pub fn get_at(&self, pos: &Pos) -> &Symbol {
        &self.pr[pos.pr_index()][pos.sy_index()]
    }
}

impl Index<usize> for Cfg {
    type Output = Pr;

    fn index(&self, idx: usize) -> &Self::Output {
        &self.pr[idx]
    }
}

impl Index<&Pos> for Cfg {
    type Output = Symbol;

    fn index(&self, pos: &Pos) -> &Self::Output {
        &self.pr[pos.pr_index()][pos.sy_index()]
    }
}

#[cfg(test)]
mod test {
    use crate::{Cfg, Pr, Symbol};

    #[test]
    fn check_serialization() {
        let g = Cfg::with_start_symbol("S")
            .add_pr(Pr::new("S", vec![Symbol::t("a", vec![0]), Symbol::n("X")]))
            .add_pr(Pr::new("X", vec![Symbol::t("b", vec![0]), Symbol::n("S")]))
            .add_pr(Pr::new(
                "X",
                vec![
                    Symbol::t("a", vec![0]),
                    Symbol::n("Y"),
                    Symbol::t("b", vec![0]),
                    Symbol::n("Y"),
                ],
            ))
            .add_pr(Pr::new(
                "Y",
                vec![Symbol::t("b", vec![0]), Symbol::t("a", vec![0])],
            ))
            .add_pr(Pr::new("Y", vec![Symbol::t("a", vec![0]), Symbol::n("Z")]))
            .add_pr(Pr::new(
                "Z",
                vec![Symbol::t("a", vec![0]), Symbol::n("Z"), Symbol::n("X")],
            ));

        let serialized = serde_json::to_string(&g).unwrap();
        println!("{}", serialized);
        let g1: Cfg = serde_json::from_str(&serialized).unwrap();
        assert_eq!(g, g1);
    }
}
