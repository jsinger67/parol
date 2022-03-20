use crate::{generate_name, group_by, Cfg, Pr, Rhs, Symbol};
use std::collections::hash_map::HashMap;
use std::hash::Hash;

struct TransformationOperand {
    modified: bool,
    pr: Vec<Pr>,
}

/// Substitutes a set of rules with given name in the given rules with the result of
/// the transformation
fn apply_rule_transformation(
    operand: TransformationOperand,
    rule_name: &str,
    trans: impl Fn(Vec<Pr>) -> Vec<Pr>,
) -> TransformationOperand {
    let TransformationOperand {
        modified: _,
        mut pr,
    } = operand;
    if let Some(rule_index) = pr.iter().position(|r| r.get_n_str() == rule_name) {
        let mut affected_rules = vec![];
        while let Some(rule_index) = pr.iter().position(|r| r.get_n_str() == rule_name) {
            affected_rules.push(pr.remove(rule_index));
        }
        let mut upper_rules = pr.split_off(rule_index);
        pr.append(&mut trans(affected_rules));
        pr.append(&mut upper_rules);

        TransformationOperand { modified: true, pr }
    } else {
        TransformationOperand {
            modified: false,
            pr,
        }
    }
}

fn find_prefix<T>(candidates: &[Vec<T>]) -> Vec<T>
where
    T: Eq + Hash + Clone,
{
    type Candidates<T> = [Vec<T>];

    fn find_prefix<T>(candidates: &Candidates<T>, n: usize) -> Vec<T>
    where
        T: Eq + Hash + Clone,
    {
        let candidates_with_len_n = candidates
            .iter()
            .filter(|c| c.len() >= n)
            .map(|c| &c[..n])
            .collect::<Vec<&[T]>>();

        if candidates_with_len_n.len() < 2 {
            return vec![]; // We need at least two items that share the same prefix!
        }

        let groups: HashMap<&[T], i32> =
            candidates_with_len_n
                .iter()
                .fold(HashMap::new(), |mut acc, c| {
                    if let Some(v) = acc.get_mut(c) {
                        *v += 1; // Increment member count per group
                    } else {
                        acc.insert(c, 1); // Insert new group with one member
                    };
                    acc
                });
        // Choose the group with the most members
        if let Some((k, v)) = groups.iter().max_by_key(|c| c.1) {
            if v > &1 {
                // Found prefix is only useful if the group contains more than one member
                k.to_vec()
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }

    fn find_longest_prefix<T>(candidates: &Candidates<T>, n: usize) -> Vec<T>
    where
        T: Eq + Hash + Clone,
    {
        let p1 = find_prefix(candidates, n);
        let p2 = find_prefix(candidates, n + 1);
        match (&p1[..], &p2[..]) {
            (&[], &[]) => vec![],
            (_, &[]) => p1,
            (_, _) => {
                let p3 = find_longest_prefix(candidates, n + 2);
                if p3.is_empty() {
                    p2
                } else {
                    p3
                }
            }
        }
    }

    find_longest_prefix(candidates, 1)
}

/// Finds the longest left prefixes in rules given.
/// Can be used to factor out these left prefixes later.
pub fn find_longest_prefixes(rules: &[Pr]) -> Vec<(String, Rhs)> {
    let rule_groups = group_by(rules, |r| r.get_n_str().to_owned());
    rule_groups.iter().fold(Vec::new(), |mut acc, e| {
        let (rule_name, rules) = &e;
        let prefix = find_prefix(
            &rules
                .iter()
                .map(|r| r.get_r().clone())
                .collect::<Vec<Rhs>>(),
        );
        if !prefix.is_empty() {
            acc.push((rule_name.clone(), prefix));
        }
        acc
    })
}

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
///
/// Applies left-factoring to the given grammar.
///
pub fn left_factor(g: &Cfg) -> Cfg {
    let Cfg { st, pr } = g.clone();

    let mut operand = TransformationOperand { modified: true, pr };

    fn mod_factor(exclusions: &[String], prefix: &[Symbol], rules: Vec<Pr>) -> Vec<Pr> {
        fn factor_out_rule(pr_name: &str, prefix: &[Symbol], pr: Pr) -> Pr {
            let prefix_len = prefix.len();
            if pr.len() < prefix_len || pr.get_r()[0..prefix_len] != prefix[..] {
                // This production of a production-set don't share the prefix - leave it unchanged.
                pr
            } else {
                let (_, mut rhs) = pr.take();
                let rhs = rhs.split_off(prefix_len);
                Pr::new(pr_name, rhs)
            }
        }

        if rules.is_empty() {
            panic!(
                "Internal error: \"Expected a non-empty production list!\" in {}({})",
                file!(),
                line!()
            );
        }

        // We change the productions A -> prefix suffix to one new production A -> prefix A'
        // where A' is a new production of the form:
        // A' -> suffix1|suffix2|... i.e. A' -> suffix1;  A' -> suffix2;  ...
        let first_rule = &rules[0];
        let suffix_rule_name =
            generate_name(exclusions, first_rule.get_n_str().to_owned() + "Suffix");
        let mut prod = prefix.to_owned();
        prod.push(Symbol::n(&suffix_rule_name));
        let prefix_rule = Pr::new(first_rule.get_n_str(), prod);
        let mut left_factored_rules = rules
            .iter()
            .map(|r| factor_out_rule(&suffix_rule_name, prefix, r.clone()))
            .collect::<Vec<Pr>>();

        left_factored_rules.insert(0, prefix_rule);
        left_factored_rules
    }

    fn var_names(pr: &[Pr]) -> Vec<String> {
        pr.iter().fold(Vec::new(), |mut acc, p| {
            if !acc.contains(&p.get_n()) {
                acc.push(p.get_n_str().to_owned());
            }
            acc = p.get_r().iter().fold(acc, |mut acc, s| {
                if let Symbol::N(n, _) = s {
                    if !acc.contains(n) {
                        acc.push(n.clone());
                    }
                }
                acc
            });
            acc
        })
    }

    fn factor_out_prefix(
        operand: TransformationOperand,
        prefix: &(String, Rhs),
    ) -> TransformationOperand {
        let exclusions = var_names(&operand.pr);
        apply_rule_transformation(operand, &prefix.0, |rs| {
            mod_factor(&exclusions, &prefix.1, rs)
        })
    }

    fn factor_out(operand: TransformationOperand) -> TransformationOperand {
        let prefixes = find_longest_prefixes(&operand.pr);
        prefixes.iter().fold(operand, &factor_out_prefix)
    }

    while operand.modified {
        operand.modified = false;
        operand = factor_out(operand);
    }
    Cfg { st, pr: operand.pr }
}

#[cfg(test)]
mod tests {
    use super::find_prefix;

    const EMPTY_VEC: Vec<i32> = Vec::new();

    #[test]
    fn check_find_prefix_on_empty() {
        let candidates: Vec<Vec<i32>> = vec![vec![]];
        assert_eq!(EMPTY_VEC, find_prefix(&candidates));
    }

    #[test]
    fn check_find_prefix_on_single() {
        let candidates = vec![vec![1, 2, 3]];
        // There is no common prefix for just one single sequence
        assert_eq!(EMPTY_VEC, find_prefix(&candidates));
    }

    #[test]
    fn check_find_prefix_on_diffs() {
        let candidates = vec![vec![4, 5], vec![1, 2, 3]];
        assert_eq!(EMPTY_VEC, find_prefix(&candidates));
    }

    #[test]
    fn check_find_prefix_1() {
        let candidates = vec![
            vec![1, 2, 3, 4, 5],
            vec![2, 3, 4, 5],
            vec![1, 2, 3, 5],
            vec![1, 2, 6, 5],
        ];
        assert_eq!(vec![1, 2, 3], find_prefix(&candidates));
    }
    #[test]
    fn check_find_prefix_2() {
        let candidates = vec![vec![1, 2, 3, 5], vec![1, 2, 6, 5]];
        assert_eq!(vec![1, 2], find_prefix(&candidates));
    }
}
