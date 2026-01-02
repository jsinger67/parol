use crate::Cfg;
use std::collections::{BTreeMap, HashSet};

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
///
/// Detects left recursions.
/// The result is a collection of vectors of non-terminal names that contain recursions.
///
pub fn detect_left_recursive_non_terminals(cfg: &Cfg) -> Vec<String> {
    let nullables = cfg.calculate_nullable_non_terminals();
    let mut can_start_with = cfg
        .get_non_terminal_set()
        .iter()
        .map(|nt| (nt.clone(), HashSet::<String>::new()))
        .collect::<BTreeMap<String, HashSet<String>>>();

    // For all non-terminals A, B calculate the relation 'A can-start-with B'
    let mut changed = true;
    while changed {
        changed = false;
        for p in &cfg.pr {
            let lhs = p.0.get_n_ref().unwrap();
            let lhs_entry = can_start_with.get_mut(lhs).unwrap();
            for s in &p.1 {
                match s {
                    crate::Symbol::N(n, _, _, _) => {
                        changed |= lhs_entry.insert(n.clone());
                        if !nullables.contains(n) {
                            break;
                        }
                    }
                    crate::Symbol::T(_) => break,
                    _ => {
                        unreachable!(
                            "Scanner switching directives have been removed from the grammar syntax."
                        );
                    }
                }
            }
        }
    }

    // Calculate transitive closure of the relation 'A can-start-with B'
    // Ex.: A->B, B->C => A->{B, C}
    changed = true;
    while changed {
        changed = false;
        for nt in can_start_with
            .keys()
            .cloned()
            .collect::<Vec<String>>()
            .iter()
        {
            let v = can_start_with
                .get(nt)
                .map_or(HashSet::default(), |s| s.clone());
            for e in &v {
                let t = can_start_with
                    .get(e)
                    .map_or(HashSet::default(), |s| s.clone());
                if let Some(f) = can_start_with.get_mut(nt) {
                    f.extend(t.iter().cloned())
                }
            }
            changed |= v.len() < can_start_with.get(nt).map_or(0, |v| v.len());
        }
    }

    // Return all non-terminals that have themselves in the 'can-start-with relation'
    can_start_with.iter().fold(Vec::new(), |mut acc, (k, v)| {
        if v.contains(k) {
            acc.push(k.to_owned());
        }
        acc
    })
}

#[cfg(test)]
mod test {
    use crate::obtain_grammar_config_from_string;

    use super::detect_left_recursive_non_terminals;

    #[derive(Debug)]
    struct TestData {
        input: &'static str,
        recursive_non_terminals: &'static [&'static str],
    }

    const LL_TESTS: &[TestData] = &[
        TestData {
            input: r#"%start A %% A: B "r"; B: C "d"; C: A "t";"#,
            recursive_non_terminals: &["A", "B", "C"],
        },
        TestData {
            input: r#"%start S %% S: S "a"; S: ;"#,
            recursive_non_terminals: &["S"],
        },
        TestData {
            input: r#"%start A %% A: A B "d"; A: A "a"; A: "a"; B: B "e"; B: "b";"#,
            recursive_non_terminals: &["A", "B"],
        },
        TestData {
            input: r#"%start E %% E: E "+" E; E: E "*" E; E: "a";"#,
            recursive_non_terminals: &["E"],
        },
        TestData {
            input: r#"%start E %% E: E "+" T; E: T; T: T "*" F; T: F; F: "id";"#,
            recursive_non_terminals: &["E", "T"],
        },
        TestData {
            input: r#"%start S %% S: "(" L ")"; S: "a"; L: L "," S; L: S;"#,
            recursive_non_terminals: &["L"],
        },
        TestData {
            input: r#"%start S %% S: S "0" S "1" S; S: "0" "1";"#,
            recursive_non_terminals: &["S"],
        },
        TestData {
            input: r#"%start S %% S: A; A: A "d"; A: A "e"; A: "a" B; A: "a" "c"; B: "b" B "c"; B: "f";"#,
            recursive_non_terminals: &["A"],
        },
        TestData {
            input: r#"%start A %% A: A A "a"; A: "b";"#,
            recursive_non_terminals: &["A"],
        },
        TestData {
            input: r#"%start A %% A: B "a"; A: A "a"; A: "c"; B: B "b"; B: A "b"; B: "d";"#,
            recursive_non_terminals: &["A", "B"],
        },
        TestData {
            input: r#"%start X %% X: X S "b"; X: S "a"; X: "b"; S: S "b"; S: X "a"; S: "a";"#,
            recursive_non_terminals: &["S", "X"],
        },
        TestData {
            input: r#"%start S %% S: A "a"; S: "b"; A: A "c"; A: S "d"; A: ;"#,
            recursive_non_terminals: &["A", "S"],
        },
    ];

    #[test]
    fn check_detect_left_recursive_non_terminals() {
        for (i, test) in LL_TESTS.iter().enumerate() {
            let grammar_config = obtain_grammar_config_from_string(test.input, false).unwrap();
            let recursions = detect_left_recursive_non_terminals(&grammar_config.cfg);
            assert_eq!(
                test.recursive_non_terminals, recursions,
                "Error at test #{i}"
            );
        }
    }
}
