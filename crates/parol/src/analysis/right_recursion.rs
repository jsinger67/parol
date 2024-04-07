use std::collections::{BTreeMap, HashSet};

use crate::Cfg;

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
/// Detects right recursions.
/// The result is a collection of vectors of non-terminal names that contain recursions.
pub fn detect_right_recursive_non_terminals(cfg: &Cfg) -> Vec<String> {
    let nullables = cfg.calculate_nullable_non_terminals();
    let mut can_end_with = cfg
        .get_non_terminal_set()
        .iter()
        .map(|nt| (nt.clone(), HashSet::<String>::new()))
        .collect::<BTreeMap<String, HashSet<String>>>();

    // For all non-terminals A, B calculate the relation 'A can-end-with B'
    let mut changed = true;
    while changed {
        changed = false;
        for p in &cfg.pr {
            let lhs = p.0.get_n_ref().unwrap();
            let lhs_entry = can_end_with.get_mut(lhs).unwrap();
            for s in p.1.iter().rev() {
                match s {
                    crate::Symbol::N(ref n, _, _) => {
                        changed |= lhs_entry.insert(n.clone());
                        if !nullables.contains(n) {
                            break;
                        }
                    }
                    crate::Symbol::T(_) => break,
                    crate::Symbol::S(_) | crate::Symbol::Push(_) | crate::Symbol::Pop => (),
                }
            }
        }
    }

    // Calculate transitive closure of the relation 'A can-end-with B'
    // Ex.: A->B, B->C => A->{B, C}
    changed = true;
    while changed {
        changed = false;
        for nt in can_end_with.keys().cloned().collect::<Vec<String>>().iter() {
            let v = can_end_with
                .get(nt)
                .map_or(HashSet::default(), |s| s.clone());
            for e in &v {
                let t = can_end_with
                    .get(e)
                    .map_or(HashSet::default(), |s| s.clone());
                if let Some(f) = can_end_with.get_mut(nt) {
                    f.extend(t.iter().cloned())
                }
            }
            changed |= v.len() < can_end_with.get(nt).map_or(0, |v| v.len());
        }
    }

    // Return all non-terminals that have themselves in the 'can-end-with relation'
    can_end_with.iter().fold(Vec::new(), |mut acc, (k, v)| {
        if v.contains(k) {
            acc.push(k.to_owned());
        }
        acc
    })
}

#[cfg(test)]
mod test {
    use crate::obtain_grammar_config_from_string;

    #[derive(Debug)]
    struct TestData {
        input: &'static str,
        recursive_non_terminals: &'static [&'static str],
    }

    const LR_TESTS: &[TestData] = &[
        TestData {
            input: r#"%start A %% A: "r" B; B: "d" C; C: "t" A;"#,
            recursive_non_terminals: &["A", "B", "C"],
        },
        TestData {
            input: r#"%start S %% S: "a" S; S: ;"#,
            recursive_non_terminals: &["S"],
        },
        TestData {
            input: r#"%start A %% A: B "d" A; A: "a" A; A: "a"; B: "e" B; B: "b";"#,
            recursive_non_terminals: &["A", "B"],
        },
        TestData {
            input: r#"%start E %% E: E "+" E; E: E "*" E; E: "a";"#,
            recursive_non_terminals: &["E"],
        },
        TestData {
            input: r#"%start E %% E: T "+" E; E: T; T: F "*" T; T: F; F: "id";"#,
            recursive_non_terminals: &["E", "T"],
        },
        TestData {
            input: r#"%start S %% S: "(" L ")"; S: "a"; L: S "," L; L: S;"#,
            recursive_non_terminals: &["L"],
        },
        TestData {
            input: r#"%start S %% S: S "0" S "1" S; S: "0" "1";"#,
            recursive_non_terminals: &["S"],
        },
        TestData {
            input: r#"%start S %% S: A; A: "d" A; A: "e" A; A: "a" B; A: "a" "c"; B: "b" B "c"; B: "f";"#,
            recursive_non_terminals: &["A"],
        },
        TestData {
            input: r#"%start A %% A: "a" A A; A: "b";"#,
            recursive_non_terminals: &["A"],
        },
        TestData {
            input: r#"%start A %% A: B "a"; A: "a" A; A: "c"; B: "b" B; B: A "b"; B: "d";"#,
            recursive_non_terminals: &["A", "B"],
        },
        TestData {
            input: r#"%start X %% X: S "b" X; X: S "a"; X: "b"; S: "b" S; S: X "a"; S: "a";"#,
            recursive_non_terminals: &["S", "X"],
        },
        TestData {
            input: r#"%start S %% S: "a" A; S: "b"; A: "c" A; A: "d" S; A: ;"#,
            recursive_non_terminals: &["A", "S"],
        },
    ];

    #[test]
    fn check_detect_right_recursive_non_terminals() {
        for (i, test) in LR_TESTS.iter().enumerate() {
            let grammar_config = obtain_grammar_config_from_string(test.input, false).unwrap();
            let recursions = super::detect_right_recursive_non_terminals(&grammar_config.cfg);
            assert_eq!(
                test.recursive_non_terminals, recursions,
                "Error at test #{i}"
            );
        }
    }
}
