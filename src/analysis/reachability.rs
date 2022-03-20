use crate::{Cfg, Pr, Symbol};
use std::collections::{BTreeMap, BTreeSet};

///
/// Calculates all reachable non-terminals.
///
/// ```
/// use parol::{Cfg, Pr, Symbol};
/// use parol::analysis::reachable_non_terminals;
/// use std::collections::BTreeSet;
/// use std::convert::From;
///
/// let g = Cfg::with_start_symbol("S")
///     .add_pr(Pr::new("S", vec![Symbol::n("Y")]))
///     .add_pr(Pr::new("Y", vec![Symbol::n("Y"), Symbol::n("Z")]))
///     .add_pr(Pr::new("Y", vec![Symbol::n("Y"), Symbol::t("a", vec![0])]))
///     .add_pr(Pr::new("Y", vec![Symbol::t("b", vec![0])]))
///     .add_pr(Pr::new("U", vec![Symbol::n("V")]))
///     .add_pr(Pr::new("X", vec![Symbol::t("c", vec![0])]))
///     .add_pr(Pr::new("V", vec![Symbol::n("V"), Symbol::t("d", vec![0])]))
///     .add_pr(Pr::new("V", vec![Symbol::t("d", vec![0])]))
///     .add_pr(Pr::new("Z", vec![Symbol::n("Z"), Symbol::n("X")]));
/// let productive = reachable_non_terminals(&g);
/// assert_eq!(
///     [
///         "S".to_owned(),
///         "X".to_owned(),
///         "Y".to_owned(),
///         "Z".to_owned()
///     ].iter().cloned().collect::<BTreeSet<String>>(),
///     productive);
/// ```
///
pub fn reachable_non_terminals(cfg: &Cfg) -> BTreeSet<String> {
    fn insert_reachable(pr: &[Pr], mut reachable_so_far: BTreeSet<String>) -> BTreeSet<String> {
        for p in pr {
            if reachable_so_far.contains(&p.get_n()) {
                for s in p.get_r() {
                    if let Symbol::N(n, _) = s {
                        reachable_so_far.insert(n.clone());
                    }
                }
            }
        }
        reachable_so_far
    }

    let mut reachable: BTreeSet<String> = BTreeSet::new();
    reachable.insert(cfg.st.clone());

    let mut current_size = 0;
    while current_size < reachable.len() {
        current_size = reachable.len();
        reachable = insert_reachable(&cfg.pr, reachable);
    }
    reachable
}

///
/// Calculates all unreachable non-terminals.
///
/// ```
/// use parol::{Cfg, Pr, Symbol};
/// use parol::analysis::unreachable_non_terminals;
/// use std::collections::BTreeSet;
/// use std::convert::From;
///
/// let g = Cfg::with_start_symbol("S")
///     .add_pr(Pr::new("S", vec![Symbol::n("Y")]))
///     .add_pr(Pr::new("Y", vec![Symbol::n("Y"), Symbol::n("Z")]))
///     .add_pr(Pr::new("Y", vec![Symbol::n("Y"), Symbol::t("a", vec![0])]))
///     .add_pr(Pr::new("Y", vec![Symbol::t("b", vec![0])]))
///     .add_pr(Pr::new("U", vec![Symbol::n("V")]))
///     .add_pr(Pr::new("X", vec![Symbol::t("c", vec![0])]))
///     .add_pr(Pr::new("V", vec![Symbol::n("V"), Symbol::t("d", vec![0])]))
///     .add_pr(Pr::new("V", vec![Symbol::t("d", vec![0])]))
///     .add_pr(Pr::new("Z", vec![Symbol::n("Z"), Symbol::n("X")]));
/// let productive = unreachable_non_terminals(&g);
/// assert_eq!(
///     [
///         "U".to_owned(),
///         "V".to_owned()
///     ].iter().cloned().collect::<BTreeSet<String>>(),
///     productive);
/// ```
///
pub fn unreachable_non_terminals(cfg: &Cfg) -> BTreeSet<String> {
    cfg.get_non_terminal_set()
        .difference(&reachable_non_terminals(cfg))
        .cloned()
        .collect()
}

///
/// Detects whether all non-terminals are reachable
///
pub fn all_non_terminals_reachable(cfg: &Cfg) -> bool {
    let all_nts = cfg.get_non_terminal_set();
    let reached = reachable_non_terminals(cfg);
    all_nts == reached
}

///
/// Calculates for a given production all reachable non-terminals.
/// Used for special derivation calculations (i.e. FOLLOW k relations)
///
/// ```
/// use parol::{Cfg, Pr, Symbol};
/// use parol::analysis::reachable_from_production;
/// use std::collections::BTreeSet;
/// use std::convert::From;
///
/// let g = Cfg::with_start_symbol("S")
///     .add_pr(Pr::new("S", vec![Symbol::n("Y")]))
///     .add_pr(Pr::new("Y", vec![Symbol::n("Y"), Symbol::n("Z")]))
///     .add_pr(Pr::new("Y", vec![Symbol::n("Y"), Symbol::t("a", vec![0])]))
///     .add_pr(Pr::new("Y", vec![Symbol::t("b", vec![0])]))
///     .add_pr(Pr::new("U", vec![Symbol::n("V")]))
///     .add_pr(Pr::new("X", vec![Symbol::t("c", vec![0])]))
///     .add_pr(Pr::new("V", vec![Symbol::n("V"), Symbol::t("d", vec![0])]))
///     .add_pr(Pr::new("V", vec![Symbol::t("d", vec![0])]))
///     .add_pr(Pr::new("Z", vec![Symbol::n("Z"), Symbol::n("X")]));
/// let productive = reachable_from_production(&g, 0);
/// assert_eq!(
///     [
///         "X".to_owned(),
///         "Y".to_owned(),
///         "Z".to_owned()
///     ].iter().cloned().collect::<BTreeSet<String>>(),
///     productive);
///
/// let productive = reachable_from_production(&g, 1);
/// assert_eq!(
///     [
///         "X".to_owned(),
///         "Y".to_owned(),
///         "Z".to_owned()
///     ].iter().cloned().collect::<BTreeSet<String>>(),
///     productive);
///
/// let productive = reachable_from_production(&g, 2);
/// assert_eq!(
///     [
///         "X".to_owned(),
///         "Y".to_owned(),
///         "Z".to_owned()
///     ].iter().cloned().collect::<BTreeSet<String>>(),
///     productive);
///
/// let productive = reachable_from_production(&g, 3);
/// assert_eq!(
///     [
///     ].iter().cloned().collect::<BTreeSet<String>>(),
///     productive);
///
/// let productive = reachable_from_production(&g, 4);
/// assert_eq!(
///     [
///         "V".to_owned(),
///     ].iter().cloned().collect::<BTreeSet<String>>(),
///     productive);
///
/// let productive = reachable_from_production(&g, 5);
/// assert_eq!(
///     [
///     ].iter().cloned().collect::<BTreeSet<String>>(),
///     productive);
///
/// let productive = reachable_from_production(&g, 6);
/// assert_eq!(
///     [
///         "V".to_owned(),
///     ].iter().cloned().collect::<BTreeSet<String>>(),
///     productive);
///
/// let productive = reachable_from_production(&g, 7);
/// assert_eq!(
///     [
///     ].iter().cloned().collect::<BTreeSet<String>>(),
///     productive);
///
/// let productive = reachable_from_production(&g, 8);
/// assert_eq!(
///     [
///         "X".to_owned(),
///         "Z".to_owned()
///     ].iter().cloned().collect::<BTreeSet<String>>(),
///     productive);
///
/// ```
///
pub fn reachable_from_production(cfg: &Cfg, prod_num: usize) -> BTreeSet<String> {
    fn insert_reachable(pr: &[Pr], mut reachable_so_far: BTreeSet<String>) -> BTreeSet<String> {
        for p in pr {
            if reachable_so_far.contains(&p.get_n()) {
                for s in p.get_r() {
                    if let Symbol::N(n, _) = s {
                        reachable_so_far.insert(n.clone());
                    }
                }
            }
        }
        reachable_so_far
    }

    let mut reachable: BTreeSet<String> = BTreeSet::new();
    let p = &cfg.pr[prod_num];
    for s in p.get_r() {
        if let Symbol::N(n, _) = s {
            reachable.insert(n.clone());
        }
    }

    let mut current_size = 0;
    while current_size < reachable.len() {
        current_size = reachable.len();
        reachable = insert_reachable(&cfg.pr, reachable);
    }
    reachable
}

///
/// Calculates for a given non-terminal all reachable non-terminals.
/// Used for special derivation calculations (i.e. FOLLOW k relations)
///
/// ```
/// use parol::{Cfg, Pr, Symbol};
/// use parol::analysis::reachable_from_non_terminal;
/// use std::collections::BTreeSet;
/// use std::convert::From;
///
/// let g = Cfg::with_start_symbol("S")
///     .add_pr(Pr::new("S", vec![Symbol::n("Y")]))
///     .add_pr(Pr::new("Y", vec![Symbol::n("Y"), Symbol::n("Z")]))
///     .add_pr(Pr::new("Y", vec![Symbol::n("Y"), Symbol::t("a", vec![0])]))
///     .add_pr(Pr::new("Y", vec![Symbol::t("b", vec![0])]))
///     .add_pr(Pr::new("U", vec![Symbol::n("V")]))
///     .add_pr(Pr::new("X", vec![Symbol::t("c", vec![0])]))
///     .add_pr(Pr::new("V", vec![Symbol::n("V"), Symbol::t("d", vec![0])]))
///     .add_pr(Pr::new("V", vec![Symbol::t("d", vec![0])]))
///     .add_pr(Pr::new("Z", vec![Symbol::n("Z"), Symbol::n("X")]));
/// let productive = reachable_from_non_terminal(&g, "S");
/// assert_eq!(
///     [
///         "X".to_owned(),
///         "Y".to_owned(),
///         "Z".to_owned()
///     ].iter().cloned().collect::<BTreeSet<String>>(),
///     productive);
///
/// let productive = reachable_from_non_terminal(&g, "U");
/// assert_eq!(
///     [
///         "V".to_owned(),
///     ].iter().cloned().collect::<BTreeSet<String>>(),
///     productive);
///
/// let productive = reachable_from_non_terminal(&g, "Y");
/// assert_eq!(
///     [
///         "X".to_owned(),
///         "Y".to_owned(),
///         "Z".to_owned()
///     ].iter().cloned().collect::<BTreeSet<String>>(),
///     productive);
///
/// let productive = reachable_from_non_terminal(&g, "X");
/// assert_eq!(
///     [
///     ].iter().cloned().collect::<BTreeSet<String>>(),
///     productive);
///
/// let productive = reachable_from_non_terminal(&g, "Z");
/// assert_eq!(
///     [
///         "X".to_owned(),
///         "Z".to_owned()
///     ].iter().cloned().collect::<BTreeSet<String>>(),
///     productive);
///
/// ```
///
pub fn reachable_from_non_terminal(cfg: &Cfg, nt: &str) -> BTreeSet<String> {
    fn insert_reachable(pr: &[Pr], mut reachable_so_far: BTreeSet<String>) -> BTreeSet<String> {
        for p in pr {
            if reachable_so_far.contains(&p.get_n()) {
                for s in p.get_r() {
                    if let Symbol::N(n, _) = s {
                        reachable_so_far.insert(n.clone());
                    }
                }
            }
        }
        reachable_so_far
    }

    let matching_prods = cfg.matching_productions(nt);
    let mut reachable: BTreeSet<String> = BTreeSet::new();
    matching_prods.iter().for_each(|(_, p)| {
        for s in p.get_r() {
            if let Symbol::N(n, _) = s {
                reachable.insert(n.clone());
            }
        }
    });

    let mut current_size = 0;
    while current_size < reachable.len() {
        current_size = reachable.len();
        reachable = insert_reachable(&cfg.pr, reachable);
    }
    reachable
}

///
/// Calculates the numbers of all productions that eventually can produce
/// the given non-terminal by applying several derivation steps.
///
///
/// ```
/// use parol::{Cfg, Pr, Symbol};
/// use parol::analysis::nt_producing_productions;
/// use std::collections::BTreeSet;
/// use std::convert::From;
///
/// let g = Cfg::with_start_symbol("S")
///     .add_pr(Pr::new("S", vec![Symbol::n("A")]))
///     .add_pr(Pr::new("A", vec![Symbol::t("x", vec![0]), Symbol::n("B"), Symbol::n("AA")]))
///     .add_pr(Pr::new("AA", vec![Symbol::t("d", vec![0]), Symbol::n("AA")]))
///     .add_pr(Pr::new("AA", vec![]))
///     .add_pr(Pr::new("B", vec![Symbol::t("y", vec![0])]))
///     .add_pr(Pr::new("C", vec![Symbol::t("b", vec![0])]));
/// let prod_numbers = nt_producing_productions(&g, "S");
/// assert_eq!(
///     [].iter().cloned().collect::<BTreeSet<usize>>(),
///     prod_numbers,
///     "NT(S)");
///
/// let prod_numbers = nt_producing_productions(&g, "A");
/// assert_eq!(
///     [0,].iter().cloned().collect::<BTreeSet<usize>>(),
///     prod_numbers,
///     "NT(A)");
///
/// let prod_numbers = nt_producing_productions(&g, "AA");
/// assert_eq!(
///     [0, 1, 2].iter().cloned().collect::<BTreeSet<usize>>(),
///     prod_numbers,
///     "NT(AA)");
///
/// let prod_numbers = nt_producing_productions(&g, "B");
/// assert_eq!(
///     [0, 1,].iter().cloned().collect::<BTreeSet<usize>>(),
///     prod_numbers,
///     "NT(B)");
///
/// let prod_numbers = nt_producing_productions(&g, "C");
/// assert_eq!(
///     [].iter().cloned().collect::<BTreeSet<usize>>(),
///     prod_numbers,
///     "NT(C)");
/// ```
///
pub fn nt_producing_productions(cfg: &Cfg, nt: &str) -> BTreeSet<usize> {
    let reachable_of = nt_reachability(cfg);
    cfg.pr
        .iter()
        .enumerate()
        .fold(BTreeSet::new(), |mut acc, (pi, p)| {
            if p.get_r()
                    .iter()
                    .any(|s| matches!(s, Symbol::N(n, _) if reachable_of.get(n).unwrap().contains(nt) || n == nt))
            {
                acc.insert(pi);
            }
            acc
        })
}

///
/// Calculates the reachable non-terminals for each non-terminal of the given Cfg.
///
///
/// Calculates for a given non-terminal all reachable non-terminals.
/// Used for special derivation calculations (i.e. FOLLOW k relations)
///
/// ```
/// use parol::{Cfg, Pr, Symbol};
/// use parol::analysis::nt_reachability;
/// use std::collections::{BTreeMap, BTreeSet};
/// use std::convert::From;
///
/// let g = Cfg::with_start_symbol("S")
///     .add_pr(Pr::new("S", vec![Symbol::n("A")]))
///     .add_pr(Pr::new("A", vec![Symbol::t("x", vec![0]), Symbol::n("B"), Symbol::n("AA")]))
///     .add_pr(Pr::new("AA", vec![Symbol::t("d", vec![0]), Symbol::n("AA")]))
///     .add_pr(Pr::new("AA", vec![]))
///     .add_pr(Pr::new("B", vec![Symbol::t("y", vec![0])]))
///     .add_pr(Pr::new("C", vec![Symbol::t("b", vec![0])]));
/// let reachability = nt_reachability(&g);
/// assert_eq!(
///     [
///         ("A".to_owned(), [
///             "AA".to_owned(),
///             "B".to_owned()].iter().cloned().collect::<BTreeSet<String>>()),
///         ("AA".to_owned(), [
///             "AA".to_owned()].iter().cloned().collect::<BTreeSet<String>>()),
///         ("B".to_owned(), [].iter().cloned().collect::<BTreeSet<String>>()),
///         ("C".to_owned(), [].iter().cloned().collect::<BTreeSet<String>>()),
///         ("S".to_owned(), [
///             "A".to_owned(),
///             "AA".to_owned(),
///             "B".to_owned()].iter().cloned().collect::<BTreeSet<String>>()),
///     ].iter().cloned().collect::<BTreeMap<String, BTreeSet<String>>>(),
///     reachability);
/// ```
///
pub fn nt_reachability(cfg: &Cfg) -> BTreeMap<String, BTreeSet<String>> {
    cfg.get_non_terminal_set()
        .iter()
        .fold(BTreeMap::new(), |mut acc, n| {
            let reachable = reachable_from_non_terminal(cfg, n);
            acc.insert(n.clone(), reachable);
            acc
        })
}
