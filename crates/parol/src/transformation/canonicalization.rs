use crate::analysis::lookahead_dfa::ProductionIndex;
use crate::generate_name;
use crate::grammar::{ProductionAttribute, SymbolAttribute};
use crate::parser::{Alternation, Alternations, Factor, Production};
use crate::utils::combine;
use crate::{Pr, Symbol};
// $env:RUST_LOG="parol::transformation::canonicalization=trace"
use log::trace;
use miette::{bail, Result};
use regex::Regex;
use std::convert::TryFrom;

lazy_static! {
    pub(crate) static ref RX_OPT_WITH_NUM_SUFFIX: Regex =
        Regex::new(r"Opt[0-9]*$").expect("error parsing regex");
}

struct TransformationOperand {
    modified: bool,
    productions: Vec<Production>,
}

fn finalize(productions: Vec<Production>) -> Result<Vec<Pr>> {
    productions
        .into_iter()
        .map(|r| {
            let Alternations(mut e) = r.rhs.clone();
            if e.len() != 1 {
                bail!(
                    "Expected one alternation per production after transformation but found {} at {}!",
                    e.len(), r
                );
            }
            let single_alternative = e.pop().unwrap();
            Ok(Pr(
                Symbol::n(&r.lhs),
                if single_alternative.0.is_empty() {
                    vec![]
                } else {
                    let prod: Result<Vec<Symbol>> =
                        single_alternative
                            .0
                            .into_iter()
                            .try_fold(Vec::new(), |mut acc, f| {
                                let s = Symbol::try_from(f)?;
                                acc.push(s);
                                Ok(acc)
                            });
                    prod?
                },
                single_alternative.1
            ))
        })
        .collect()
}

fn variable_names(productions: &[Production]) -> Vec<String> {
    let mut productions_vars = productions.iter().fold(vec![], |mut res, r| {
        let variable = &r.lhs;
        res.push(variable.clone());
        let mut alternation_vars = r.rhs.0.iter().fold(vec![], |mut res, a| {
            let mut factors_vars = a.0.iter().fold(vec![], |mut res, f| {
                if let Factor::NonTerminal(n, ..) = f {
                    res.push(n.clone());
                }
                res
            });
            res.append(&mut factors_vars);
            res
        });
        res.append(&mut alternation_vars);
        res
    });
    productions_vars.sort();
    productions_vars.dedup();
    productions_vars
}

// Substitutes the production on 'index' in the vector of productions with the result of the transformation
fn apply_production_transformation(
    productions: &mut Vec<Production>,
    index: usize,
    trans: impl Fn(Production) -> Vec<Production>,
) {
    let production_to_substitute = productions.remove(index);
    let mut substitutes = trans(production_to_substitute);
    productions.reserve(substitutes.len());
    for i in index..(index + substitutes.len()) {
        productions.insert(i, substitutes.remove(0));
    }
}

fn find_production_with_factor(
    productions: &[Production],
    pred: impl Fn(&Factor) -> bool,
) -> Option<(ProductionIndex, usize)> {
    let production_index = productions.iter().position(|r| {
        let Alternations(e) = &r.rhs;
        e.iter().any(|r| r.0.iter().any(&pred))
    });
    if let Some(production_index) = production_index {
        let Alternations(e) = &productions[production_index].rhs;
        Some((
            production_index,
            e.iter().position(|r| r.0.iter().any(&pred)).unwrap(),
        ))
    } else {
        None
    }
}

// fn contains_optional(alts: &[Alternation]) -> bool {
//     alts.iter()
//         .any(|a| a.0.iter().any(|f| matches!(f, Factor::Optional(_))))
// }

fn format_productions(prods: &[Production]) -> String {
    prods
        .iter()
        .fold(Vec::new(), |mut acc: Vec<String>, p| {
            acc.push(p.to_string());
            acc
        })
        .join("\n")
}

// Extract inner options into own productions to avoid a combinatorial explosion and some problems
// related to auto-generation.
// A production with n optionals is normally expanded into n^2 productions:
// Example:
// S: [A] [B] [C];
// =>
// S: ε | A | B | C | A B | A C | B C | A B C;
//
// If you extract the optionals as extra productions, you get less productions (2 * n + 1) and a
// much simpler "type representation" of it.
// S: [A] [B] [C];
// =>
// S: AOpt BOpt COpt;
// AOpt: ε | A;
// BOpt: ε | B;
// COpt: ε | C;
//
// This transformation step must be executed first.
fn extract_options(opd: TransformationOperand) -> TransformationOperand {
    fn extract_optional_in_alt(
        alt: &mut Alternation,
        non_terminal: String,
        exclusions: &[String],
    ) -> Option<(String, Alternations)> {
        for factor in &mut alt.0 {
            match factor.clone() {
                Factor::Group(_) | Factor::Repeat(_) => {
                    if let Some((name, alts)) = extract_optional_in_alts(
                        factor.inner_alts_mut().expect("Should always succeed"),
                        non_terminal.clone(),
                        exclusions,
                    ) {
                        return Some((name, alts));
                    }
                }
                Factor::Optional(alts) => {
                    let preferred_name = if RX_OPT_WITH_NUM_SUFFIX.is_match(&non_terminal) {
                        non_terminal
                    } else {
                        format!("{}Opt", non_terminal)
                    };
                    let new_opt_production_name = generate_name(exclusions, preferred_name);
                    *factor = Factor::NonTerminal(
                        new_opt_production_name.clone(),
                        SymbolAttribute::Option,
                        None,
                    );
                    trace!(
                        "Extracting optional {} into production {}",
                        alts.to_par(),
                        new_opt_production_name
                    );
                    return Some((new_opt_production_name, alts));
                }
                _ => {}
            }
        }
        None
    }

    fn extract_optional_in_alts(
        alts: &mut Alternations,
        non_terminal: String,
        exclusions: &[String],
    ) -> Option<(String, Alternations)> {
        for alt in &mut alts.0 {
            if let Some((name, alts)) =
                extract_optional_in_alt(alt, non_terminal.clone(), exclusions)
            {
                return Some((name, alts));
            }
        }
        None
    }

    fn extract_optional_in_production(
        prod: &mut Production,
        non_terminal: String,
        exclusions: &[String],
    ) -> Option<(String, Alternations)> {
        for alt in &mut prod.rhs.0 {
            if let Some((name, alts)) =
                extract_optional_in_alt(alt, non_terminal.clone(), exclusions)
            {
                return Some((name, alts));
            }
        }
        None
    }

    fn extract_optional_in_productions(
        prods: &mut [Production],
        exclusions: &[String],
    ) -> Option<(String, ProductionIndex, Alternations)> {
        let mut prod_num = 0;
        #[allow(clippy::explicit_counter_loop)]
        for prod in prods {
            if let Some((name, alts)) =
                extract_optional_in_production(prod, prod.lhs.clone(), exclusions)
            {
                return Some((name, prod_num, alts));
            }
            prod_num += 1;
        }
        None
    }

    let mut modified = true;
    let mut productions = opd.productions;

    while modified {
        let exclusions = variable_names(&productions);
        if let Some((name, prod_num, alts)) =
            extract_optional_in_productions(&mut productions, &exclusions)
        {
            modified = true;
            // Add the new optional productions
            productions.insert(
                prod_num + 1,
                Production::new(
                    name.clone(),
                    Alternations(vec![
                        Alternation::new().with_attribute(ProductionAttribute::OptionalNone)
                    ]),
                ),
            );
            productions.insert(
                prod_num + 1,
                Production::new(
                    name,
                    Alternations(vec![Alternation(
                        vec![Factor::Group(alts)],
                        ProductionAttribute::OptionalSome,
                    )]),
                ),
            );
        } else {
            modified = false;
        }
    }

    TransformationOperand {
        modified,
        productions,
    }
}

// Transform productions with multiple alternatives
fn separate_alternatives(opd: TransformationOperand) -> TransformationOperand {
    fn production_has_multiple_alts(r: &Production) -> bool {
        let Alternations(e) = &r.rhs;
        e.len() > 1
    }

    // -------------------------------------------------------------------------
    // Replace the given production with multiple alternatives by a list of new productions.
    // -------------------------------------------------------------------------
    fn separate_production_with_multiple_alts(r: Production) -> Vec<Production> {
        let Production { lhs, rhs } = r;
        let Alternations(e) = rhs;

        e.into_iter()
            .map(|a| Production {
                lhs: lhs.clone(),
                rhs: Alternations(vec![a]),
            })
            .collect::<Vec<Production>>()
    }

    fn separate_single_production(productions: &mut Vec<Production>) -> bool {
        let candidate_index = productions.iter().position(&production_has_multiple_alts);
        if let Some(index) = candidate_index {
            apply_production_transformation(
                productions,
                index,
                separate_production_with_multiple_alts,
            );
            true
        } else {
            false
        }
    }

    let mut modified = opd.modified;
    let mut productions = opd.productions;

    while separate_single_production(&mut productions) {
        modified |= true;
    }

    TransformationOperand {
        modified,
        productions,
    }
}

// -------------------------------------------------------------------------
// Replace the first Factor that is a R with a non-left-recursive substitution.
// -------------------------------------------------------------------------
// R  -> x { a } y
// =>
// Case 1: Iff a is only of size 1
// R  -> x R' y        (1) - Non-terminal R' receives SymbolAttribute::RepetitionAnchor
// R' -> a R'          (2) - Alternation receives ProductionAttribute::AddToCollection
// R' ->               (2a)- Alternation receives ProductionAttribute::CollectionStart
// Case 2: Otherwise
// R  -> x R' y        (1) - Non-terminal R' receives SymbolAttribute::RepetitionAnchor
// R' -> (a) R'        (2) - Alternation receives ProductionAttribute::AddToCollection
// R' ->               (2a)- Alternation receives ProductionAttribute::CollectionStart
///
fn eliminate_single_rep(
    exclusions: &[String],
    alt_index: usize,
    production: Production,
) -> Vec<Production> {
    let production_name = production.lhs.clone();
    if let Some(rpt_index_in_alt) = production.rhs.0[alt_index]
        .0
        .iter()
        .position(|f| matches!(f, Factor::Repeat(_)))
    {
        let r_tick_name = generate_name(exclusions, production_name + "List");
        if let Factor::Repeat(repeat) = production.rhs.0[alt_index].0[rpt_index_in_alt].clone() {
            let mut production1 = production.clone();
            production1.rhs.0[alt_index].0[rpt_index_in_alt] =
                Factor::NonTerminal(r_tick_name.clone(), SymbolAttribute::RepetitionAnchor, None);

            let production2 = if repeat.0.len() == 1 {
                // Case 1
                let mut rhs_p2 = repeat.0;
                rhs_p2[0].push(Factor::default_non_terminal(r_tick_name.clone()));
                rhs_p2[0].1 = ProductionAttribute::AddToCollection;

                Production {
                    lhs: r_tick_name.clone(),
                    rhs: Alternations(rhs_p2),
                }
            } else {
                // Case 2
                Production {
                    lhs: r_tick_name.clone(),
                    rhs: Alternations(vec![Alternation::new()
                        .with_factors(if repeat.0.len() == 1 {
                            let mut fs = repeat.0[0].0.clone();
                            fs.push(Factor::default_non_terminal(r_tick_name.clone()));
                            fs
                        } else {
                            vec![
                                Factor::Group(repeat),
                                Factor::default_non_terminal(r_tick_name.clone()),
                            ]
                        })
                        .with_attribute(ProductionAttribute::AddToCollection)]),
                }
            };

            let production2a = Production {
                lhs: r_tick_name,
                rhs: Alternations(vec![
                    Alternation::new().with_attribute(ProductionAttribute::CollectionStart)
                ]),
            };

            vec![production1, production2, production2a]
        } else {
            panic!("Expected Factor::Repeat!");
        }
    } else {
        vec![production]
    }
}

// Eliminate repetitions
fn eliminate_repetitions(opd: TransformationOperand) -> TransformationOperand {
    fn find_production_with_repetition(
        productions: &[Production],
    ) -> Option<(ProductionIndex, usize)> {
        find_production_with_factor(productions, |f| matches!(f, Factor::Repeat(_)))
    }

    fn eliminate_repetition(productions: &mut Vec<Production>) -> bool {
        if let Some((production_index, alt_index)) = find_production_with_repetition(productions) {
            let exclusions = variable_names(productions);
            apply_production_transformation(productions, production_index, |r| {
                eliminate_single_rep(&exclusions, alt_index, r)
            });
            true
        } else {
            false
        }
    }

    let mut modified = opd.modified;
    let mut productions = opd.productions;

    // trace!(
    //     "\nTry to remove repetitions\n{}",
    //     format_productions(&productions)
    // );

    while eliminate_repetition(&mut productions) {
        modified |= true;
        // trace!(
        //     "\nRemoved repetitions\n{}",
        //     format_productions(&productions)
        // );
    }
    TransformationOperand {
        modified,
        productions,
    }
}

// -------------------------------------------------------------------------
// Replace the first Factor that is an O with new productions.
// -------------------------------------------------------------------------
// R  -> x [ a ] y.
// =>
// Case 1: Iff a is only of size 1
// R  -> x a y         (1)
// R  -> x y           (1a)
// Case 2: Otherwise
// R  -> x R' y.       (1)
// R  -> x y.          (1a)
// R' -> a.            (2)
fn eliminate_single_opt(
    exclusions: &[String],
    alt_index: usize,
    production: Production,
) -> Vec<Production> {
    let production_name = production.lhs.clone();
    if let Some(opt_index_in_alt) = production.rhs.0[alt_index]
        .0
        .iter()
        .position(|f| matches!(f, Factor::Optional(_)))
    {
        if let Factor::Optional(optional) = production.rhs.0[alt_index].0[opt_index_in_alt].clone()
        {
            if optional.0.len() == 1 {
                // Case 1
                let mut production1 = production.clone();
                production1.rhs.0[alt_index].0.remove(opt_index_in_alt);
                optional.0[0].0.iter().rev().for_each(|fac| {
                    production1.rhs.0[alt_index]
                        .0
                        .insert(opt_index_in_alt, fac.clone())
                });
                let mut production1a = production.clone();
                production1a.rhs.0[alt_index].0.remove(opt_index_in_alt);

                vec![production1, production1a]
            } else {
                // Case 2
                let r_tick_name = generate_name(exclusions, production_name + "Opt");
                let mut production1 = production.clone();
                production1.rhs.0[alt_index].0[opt_index_in_alt] =
                    Factor::default_non_terminal(r_tick_name.clone());

                let mut production1a = production1.clone();
                production1a.rhs.0[0].0.remove(opt_index_in_alt);

                let production2 = Production {
                    lhs: r_tick_name,
                    rhs: if optional.0.len() == 1 {
                        Alternations(vec![
                            Alternation::new().with_factors(optional.0[0].0.clone())
                        ])
                    } else {
                        optional
                    },
                };

                vec![production1, production1a, production2]
            }
        } else {
            panic!("Expected Factor::Optional!");
        }
    } else {
        vec![production]
    }
}

fn eliminate_options(opd: TransformationOperand) -> TransformationOperand {
    fn find_production_with_optional(
        productions: &[Production],
    ) -> Option<(ProductionIndex, usize)> {
        find_production_with_factor(productions, |f| matches!(f, Factor::Optional(_)))
    }
    fn eliminate_option(productions: &mut Vec<Production>) -> bool {
        if let Some((production_index, alt_index)) = find_production_with_optional(productions) {
            let exclusions = variable_names(productions);
            apply_production_transformation(productions, production_index, |r| {
                eliminate_single_opt(&exclusions, alt_index, r)
            });
            true
        } else {
            false
        }
    }

    let mut modified = opd.modified;
    let mut productions = opd.productions;

    trace!(
        "\nTry to remove options\n{}",
        format_productions(&productions)
    );

    while eliminate_option(&mut productions) {
        modified |= true;
        trace!("\nRemoved options\n{}", format_productions(&productions));
    }
    TransformationOperand {
        modified,
        productions,
    }
}

// -------------------------------------------------------------------------
// Replace the first Factor that is a G with new productions.
// -------------------------------------------------------------------------
// R  -> x ( g ) y.
// =>
// Case 1: Iff g is only of size 1
// R  -> x g y.        (1)
// Case 2: Otherwise
// R  -> x G y.        (1)
// G  -> g.            (2)
fn eliminate_single_grp(
    exclusions: &[String],
    alt_index: usize,
    production: Production,
) -> Vec<Production> {
    let production_name = production.lhs.clone();
    if let Some(grp_index_in_alt) = production.rhs.0[alt_index]
        .0
        .iter()
        .position(|f| matches!(f, Factor::Group(_)))
    {
        if let Factor::Group(group) = production.rhs.0[alt_index].0[grp_index_in_alt].clone() {
            if group.0.len() == 1 {
                // Case 1
                let mut production1 = production.clone();
                production1.rhs.0[alt_index].0.remove(grp_index_in_alt);
                group.0[0].0.iter().rev().for_each(|fac| {
                    production1.rhs.0[alt_index]
                        .0
                        .insert(grp_index_in_alt, fac.clone())
                });
                vec![production1]
            } else {
                // Case 2
                let g_name = generate_name(exclusions, production_name + "Group");
                if let Factor::Group(group) =
                    production.rhs.0[alt_index].0[grp_index_in_alt].clone()
                {
                    let mut production1 = production.clone();
                    production1.rhs.0[alt_index].0[grp_index_in_alt] =
                        Factor::default_non_terminal(g_name.clone());
                    let production2 = Production {
                        lhs: g_name,
                        rhs: group,
                    };

                    vec![production1, production2]
                } else {
                    panic!("Expected Factor::Group!");
                }
            }
        } else {
            panic!("Expected Group here");
        }
    } else {
        vec![production]
    }
}

fn eliminate_groups(opd: TransformationOperand) -> TransformationOperand {
    fn find_production_with_group(productions: &[Production]) -> Option<(ProductionIndex, usize)> {
        find_production_with_factor(productions, |f| matches!(f, Factor::Group(_)))
    }
    fn eliminate_group(productions: &mut Vec<Production>) -> bool {
        if let Some((production_index, alt_index)) = find_production_with_group(productions) {
            let exclusions = variable_names(productions);
            apply_production_transformation(productions, production_index, |r| {
                eliminate_single_grp(&exclusions, alt_index, r)
            });
            true
        } else {
            false
        }
    }

    let mut modified = opd.modified;
    let mut productions = opd.productions;

    while eliminate_group(&mut productions) {
        modified |= true;
    }
    TransformationOperand {
        modified,
        productions,
    }
}

fn eliminate_duplicates(opd: TransformationOperand) -> TransformationOperand {
    fn find_productions_with_same_rhs(
        productions: &[Production],
    ) -> Option<(ProductionIndex, ProductionIndex)> {
        for i in 0..productions.len() {
            for j in 0..productions.len() {
                if i != j && productions[i].rhs == productions[j].rhs {
                    let (i, j) = if i < j { (i, j) } else { (j, i) };
                    let first = &productions[i].lhs;
                    let duplicate = &productions[j].lhs;
                    if productions.iter().filter(|pr| &pr.lhs == first).count() == 1
                        && productions.iter().filter(|pr| &pr.lhs == duplicate).count() == 1
                    {
                        return Some((i, j));
                    }
                }
            }
        }
        None
    }
    // -------------------------------------------------------------------------
    // Replace the all occurrences of the LHS of the second production within
    // all productions RHS.
    // Then Remove the second production.
    // -------------------------------------------------------------------------
    fn eliminate_single_duplicate(
        productions: &mut Vec<Production>,
        production_index_1: ProductionIndex,
        production_index_2: ProductionIndex,
    ) {
        let to_find = productions[production_index_2].lhs.clone();
        let replace_with = productions[production_index_1].lhs.clone();

        #[allow(clippy::needless_range_loop)]
        for pi in 0..productions.len() {
            if pi != production_index_2 {
                let pr = &mut productions[pi];
                debug_assert!(pr.rhs.0.len() == 1, "Only one single Alternation expected!");
                for s in &mut pr.rhs.0[0].0 {
                    if let Factor::NonTerminal(n, ..) = s {
                        if n == &to_find {
                            *s = Factor::default_non_terminal(replace_with.clone());
                        }
                    }
                }
            }
        }

        productions.remove(production_index_2);
    }
    fn eliminate_duplicate(productions: &mut Vec<Production>) -> bool {
        if let Some((production_index_1, production_index_2)) =
            find_productions_with_same_rhs(productions)
        {
            eliminate_single_duplicate(productions, production_index_1, production_index_2);
            true
        } else {
            false
        }
    }

    let mut modified = opd.modified;
    let mut productions = opd.productions;

    while eliminate_duplicate(&mut productions) {
        modified |= true;
    }
    TransformationOperand {
        modified,
        productions,
    }
}

// -------------------------------------------------------------------------
// Guidelines:
// After applying all transformation inner (sub-) expressions should be factored out.
// The grammar's structure should be 'linear' then (i.e no loops like in {}).
// The input order should be preserved as much as possible.
// -------------------------------------------------------------------------
pub(crate) fn transform_productions(productions: Vec<Production>) -> Result<Vec<Pr>> {
    trace!(
        "\nStarting transformation\n{}",
        format_productions(&productions)
    );

    let mut operand = TransformationOperand {
        modified: true,
        productions,
    };

    while operand.modified {
        operand.modified = false;
        operand = extract_options(operand);
    }

    let trans_fn = combine(
        combine(
            combine(separate_alternatives, eliminate_repetitions),
            eliminate_options,
        ),
        eliminate_groups,
    );

    operand.modified = true;
    while operand.modified {
        operand.modified = false;
        operand = trans_fn(operand);
    }

    operand.modified = true;
    while operand.modified {
        operand.modified = false;
        operand = eliminate_duplicates(operand);
    }

    finalize(operand.productions)
}

#[cfg(test)]
mod test {
    use super::{
        eliminate_single_grp, eliminate_single_opt, eliminate_single_rep, Alternation,
        Alternations, Factor, Production,
    };
    use crate::{
        grammar::{ProductionAttribute, SymbolAttribute, TerminalKind},
        transformation::canonicalization::apply_production_transformation,
    };

    macro_rules! terminal {
        ($term:literal) => {
            Factor::Terminal(
                $term.to_string(),
                TerminalKind::Legacy,
                vec![0],
                SymbolAttribute::None,
                None,
            )
        };
    }

    // R  -> x { r1 r2 } y
    // =>
    // R  -> x R' y        (1)
    // R' -> r1 r2 R'      (2)
    // R' ->               (2a)

    #[test]
    fn eliminate_single_rep_case_1() {
        // Start: x { r1 r2 } y;
        let production = Production {
            lhs: "Start".to_string(),
            rhs: Alternations(vec![Alternation::new().with_factors(vec![
                terminal!("x"),
                Factor::Repeat(Alternations(vec![
                    Alternation::new().with_factors(vec![terminal!("r1"), terminal!("r2")])
                ])),
                terminal!("y"),
            ])]),
        };

        let productions = eliminate_single_rep(&[production.lhs.clone()], 0, production);
        assert_eq!(3, productions.len());
        // Start: x StartList y;
        assert_eq!(
            Production {
                lhs: "Start".to_string(),
                rhs: Alternations(vec![Alternation::new().with_factors(vec![
                    terminal!("x"),
                    Factor::NonTerminal(
                        "StartList".to_string(),
                        SymbolAttribute::RepetitionAnchor,
                        None
                    ),
                    terminal!("y"),
                ])])
            },
            productions[0]
        );
        // StartList: r1 r2 StartList;
        assert_eq!(
            Production {
                lhs: "StartList".to_string(),
                rhs: Alternations(vec![Alternation::new()
                    .with_factors(vec![
                        terminal!("r1"),
                        terminal!("r2"),
                        Factor::default_non_terminal("StartList".to_string()),
                    ])
                    .with_attribute(ProductionAttribute::AddToCollection)])
            },
            productions[1]
        );
        // StartList: ;
        assert_eq!(
            Production {
                lhs: "StartList".to_string(),
                rhs: Alternations(vec![
                    Alternation::new().with_attribute(ProductionAttribute::CollectionStart)
                ])
            },
            productions[2]
        );
    }

    // R  -> x { r1 | r2 } y
    // =>
    // R  -> x R' y        (1)
    // R' -> ( r1 | r2 ) R'        (2)
    // R' ->               (2a)
    #[test]
    fn eliminate_single_rep_case_2() {
        // Start: x { r1 | r2 } y;
        let production = Production {
            lhs: "Start".to_string(),
            rhs: Alternations(vec![Alternation::new().with_factors(vec![
                terminal!("x"),
                Factor::Repeat(Alternations(vec![
                    Alternation::new().with_factors(vec![terminal!("r1")]),
                    Alternation::new().with_factors(vec![terminal!("r2")]),
                ])),
                terminal!("x"),
            ])]),
        };

        let productions = eliminate_single_rep(&[production.lhs.clone()], 0, production);
        assert_eq!(3, productions.len());
        // Start: x StartList y;
        assert_eq!(
            Production {
                lhs: "Start".to_string(),
                rhs: Alternations(vec![Alternation::new().with_factors(vec![
                    terminal!("x"),
                    Factor::NonTerminal(
                        "StartList".to_string(),
                        SymbolAttribute::RepetitionAnchor,
                        None
                    ),
                    terminal!("x"),
                ])])
            },
            productions[0]
        );
        // StartList: ( r1 | r2 ) StartList;
        assert_eq!(
            Production {
                lhs: "StartList".to_string(),
                rhs: Alternations(vec![Alternation::new()
                    .with_factors(vec![
                        Factor::Group(Alternations(vec![
                            Alternation::new().with_factors(vec![terminal!("r1"),]),
                            Alternation::new().with_factors(vec![terminal!("r2"),]),
                        ])),
                        Factor::default_non_terminal("StartList".to_string()),
                    ])
                    .with_attribute(ProductionAttribute::AddToCollection)])
            },
            productions[1]
        );
        // StartList: ;
        assert_eq!(
            Production {
                lhs: "StartList".to_string(),
                rhs: Alternations(vec![
                    Alternation::new().with_attribute(ProductionAttribute::CollectionStart)
                ])
            },
            productions[2]
        );
    }

    // R  -> x [ o1 o2 ] y.
    // =>
    // R  -> x o1 o2 y     (1)
    // R  -> x y           (1a)
    #[test]
    fn eliminate_single_opt_case_1() {
        // Start: x [ o1 o2 ] y;
        let production = Production {
            lhs: "Start".to_string(),
            rhs: Alternations(vec![Alternation::new().with_factors(vec![
                terminal!("x"),
                Factor::Optional(Alternations(vec![
                    Alternation::new().with_factors(vec![terminal!("o1"), terminal!("o2")])
                ])),
                terminal!("y"),
            ])]),
        };

        let productions = eliminate_single_opt(&[production.lhs.clone()], 0, production);
        assert_eq!(2, productions.len());
        // Start: x o1 o2 y;
        assert_eq!(
            Production {
                lhs: "Start".to_string(),
                rhs: Alternations(vec![Alternation::new().with_factors(vec![
                    terminal!("x"),
                    terminal!("o1"),
                    terminal!("o2"),
                    terminal!("y"),
                ])])
            },
            productions[0]
        );
        // Start: x y;
        assert_eq!(
            Production {
                lhs: "Start".to_string(),
                rhs: Alternations(vec![
                    Alternation::new().with_factors(vec![terminal!("x"), terminal!("y"),])
                ])
            },
            productions[1]
        );
    }

    // R  -> x [ o1 | o2 ] y.
    // =>
    // R  -> x R' y.       (1)
    // R  -> x y.          (1a)
    // R' -> o1 | o2.      (2)
    #[test]
    fn eliminate_single_opt_case_2() {
        // Start: x [ o1 | o2 ] y;
        let production = Production {
            lhs: "Start".to_string(),
            rhs: Alternations(vec![Alternation::new().with_factors(vec![
                terminal!("x"),
                Factor::Optional(Alternations(vec![
                    Alternation::new().with_factors(vec![terminal!("o1")]),
                    Alternation::new().with_factors(vec![terminal!("o2")]),
                ])),
                terminal!("y"),
            ])]),
        };

        let productions = eliminate_single_opt(&[production.lhs.clone()], 0, production);
        assert_eq!(3, productions.len());
        // Start: x StartOpt y;
        assert_eq!(
            Production {
                lhs: "Start".to_string(),
                rhs: Alternations(vec![Alternation::new().with_factors(vec![
                    terminal!("x"),
                    Factor::default_non_terminal("StartOpt".to_string()),
                    terminal!("y"),
                ])])
            },
            productions[0]
        );
        // Start: x y;
        assert_eq!(
            Production {
                lhs: "Start".to_string(),
                rhs: Alternations(vec![
                    Alternation::new().with_factors(vec![terminal!("x"), terminal!("y"),]),
                ])
            },
            productions[1]
        );
        // StartOpt: o1 | o2;
        assert_eq!(
            Production {
                lhs: "StartOpt".to_string(),
                rhs: Alternations(vec![
                    Alternation::new().with_factors(vec![terminal!("o1")]),
                    Alternation::new().with_factors(vec![terminal!("o2")]),
                ])
            },
            productions[2]
        );
    }

    // R  -> x ( g1 g2 ) y.
    // =>
    // R  -> x g y.        (1)
    #[test]
    fn eliminate_single_grp_case_1() {
        // Start: x ( g1 g2 ) y;
        let production = Production {
            lhs: "Start".to_string(),
            rhs: Alternations(vec![Alternation::new().with_factors(vec![
                terminal!("x"),
                Factor::Group(Alternations(vec![
                    Alternation::new().with_factors(vec![terminal!("g1"), terminal!("g2")])
                ])),
                terminal!("y"),
            ])]),
        };

        let productions = eliminate_single_grp(&[production.lhs.clone()], 0, production);
        assert_eq!(1, productions.len());
        // Start: x g1 g2 y;
        assert_eq!(
            Production {
                lhs: "Start".to_string(),
                rhs: Alternations(vec![Alternation::new().with_factors(vec![
                    terminal!("x"),
                    terminal!("g1"),
                    terminal!("g2"),
                    terminal!("y"),
                ])])
            },
            productions[0]
        );
    }

    // R  -> x ( g1 | g2 ) y.
    // =>
    // R  -> x G y.         (1)
    // G  -> g1 | g2.       (2)
    #[test]
    fn eliminate_single_grp_case_2() {
        // Start: x ( g1 | g2 ) y;
        let production = Production {
            lhs: "Start".to_string(),
            rhs: Alternations(vec![Alternation::new().with_factors(vec![
                terminal!("x"),
                Factor::Group(Alternations(vec![
                    Alternation::new().with_factors(vec![terminal!("g1")]),
                    Alternation::new().with_factors(vec![terminal!("g2")]),
                ])),
                terminal!("y"),
            ])]),
        };

        let productions = eliminate_single_grp(&[production.lhs.clone()], 0, production);
        assert_eq!(2, productions.len());
        // Start: x StartGroup y;
        assert_eq!(
            Production {
                lhs: "Start".to_string(),
                rhs: Alternations(vec![Alternation::new().with_factors(vec![
                    terminal!("x"),
                    Factor::default_non_terminal("StartGroup".to_string()),
                    terminal!("y"),
                ])])
            },
            productions[0]
        );
        // StartGroup: g1 | g2;
        assert_eq!(
            Production {
                lhs: "StartGroup".to_string(),
                rhs: Alternations(vec![
                    Alternation::new().with_factors(vec![terminal!("g1")]),
                    Alternation::new().with_factors(vec![terminal!("g2")]),
                ])
            },
            productions[1]
        );
    }

    #[test]
    fn test_apply_production_transformation1() {
        let mut productions = vec![
            Production::new("A".to_string(), Alternations::new()),
            Production::new("B".to_string(), Alternations::new()),
        ];

        apply_production_transformation(&mut productions, 1, |_| {
            vec![
                Production::new("C".to_string(), Alternations::new()),
                Production::new("D".to_string(), Alternations::new()),
            ]
        });

        assert_eq!(
            vec![
                Production::new("A".to_string(), Alternations::new()),
                Production::new("C".to_string(), Alternations::new()),
                Production::new("D".to_string(), Alternations::new()),
            ],
            productions
        );
    }

    #[test]
    fn test_apply_production_transformation2() {
        let mut productions = vec![
            Production::new("A".to_string(), Alternations::new()),
            Production::new("B".to_string(), Alternations::new()),
        ];

        apply_production_transformation(&mut productions, 0, |_| {
            vec![
                Production::new("C".to_string(), Alternations::new()),
                Production::new("D".to_string(), Alternations::new()),
            ]
        });

        assert_eq!(
            vec![
                Production::new("C".to_string(), Alternations::new()),
                Production::new("D".to_string(), Alternations::new()),
                Production::new("B".to_string(), Alternations::new()),
            ],
            productions
        );
    }

    #[test]
    fn test_apply_production_transformation_no_change() {
        let mut productions = vec![
            Production::new("A".to_string(), Alternations::new()),
            Production::new("B".to_string(), Alternations::new()),
        ];

        apply_production_transformation(&mut productions, 0, |_| {
            vec![Production::new("A".to_string(), Alternations::new())]
        });

        assert_eq!(
            vec![
                Production::new("A".to_string(), Alternations::new()),
                Production::new("B".to_string(), Alternations::new()),
            ],
            productions
        );
    }

    #[test]
    fn test_apply_production_transformation_empty_substitute() {
        let mut productions = vec![
            Production::new("A".to_string(), Alternations::new()),
            Production::new("B".to_string(), Alternations::new()),
        ];

        apply_production_transformation(&mut productions, 0, |_| vec![]);

        assert_eq!(
            vec![Production::new("B".to_string(), Alternations::new()),],
            productions
        );
    }
}
