use crate::parser::{Factor, ParolGrammar};
use crate::transformation::transform_productions;
use crate::{generators, Cfg, GrammarConfig, ScannerConfig, Symbol, Terminal, TerminalKind};
use anyhow::{bail, Result};

use super::parol_grammar::LookaheadExpression;

pub(crate) fn try_to_convert(parol_grammar: ParolGrammar) -> Result<GrammarConfig> {
    let st = parol_grammar.start_symbol;
    let non_terminals =
        parol_grammar
            .productions
            .iter()
            .fold(Vec::<String>::new(), |mut acc, p| {
                if !acc.contains(&p.lhs) {
                    acc.push(p.lhs.clone());
                }
                acc
            });
    let pr = transform_productions(parol_grammar.productions, parol_grammar.grammar_type)?;
    let cfg = Cfg { st, pr };
    let title = parol_grammar.title;
    let comment = parol_grammar.comment;
    // The first scanner configuration should always be the default configuration
    debug_assert_eq!(parol_grammar.scanner_configurations[0].name, "INITIAL");
    let line_comments = parol_grammar.scanner_configurations[0]
        .line_comments
        .clone();
    let block_comments = parol_grammar.scanner_configurations[0]
        .block_comments
        .clone();
    let auto_newline = !parol_grammar.scanner_configurations[0].auto_newline_off;
    let auto_ws = !parol_grammar.scanner_configurations[0].auto_ws_off;
    let lookahead_size = 1; // Updated later

    let scanner_config = ScannerConfig::default()
        .with_line_comments(line_comments)
        .with_block_comments(block_comments)
        .with_auto_newline(auto_newline)
        .with_auto_ws(auto_ws);

    let mut grammar_config = GrammarConfig::new(cfg, lookahead_size)
        .with_title(title)
        .with_non_terminals(non_terminals)
        .with_comment(comment)
        .with_grammar_type(parol_grammar.grammar_type)
        .add_scanner(scanner_config);

    for u in parol_grammar.user_type_definitions {
        grammar_config = grammar_config.add_user_type_def(u.0, u.1.to_string());
    }

    for s in 1..parol_grammar.scanner_configurations.len() {
        grammar_config = grammar_config.add_scanner(try_from_scanner_config(
            &parol_grammar.scanner_configurations[s],
            s,
        )?);
    }

    let terminal_resolver = grammar_config.cfg.get_terminal_index_function();
    let scanner_resolver = |name: &str| -> Option<usize> {
        parol_grammar
            .scanner_configurations
            .iter()
            .position(|sc| sc.name == name)
    };
    // Finds the terminal token from the name of the primary non-terminal
    let pr_copy = grammar_config.cfg.pr.clone();
    let terminal_finder =
        move |name: &str| -> Option<(String, TerminalKind, Option<LookaheadExpression>)> {
            pr_copy.iter().find_map(|p| {
                if p.0.get_n_ref().unwrap() == name && p.1.len() == 1 {
                    match &p.1[0] {
                        Symbol::T(Terminal::Trm(t, k, _, _, _, _, l)) => {
                            Some((t.to_owned(), *k, l.clone()))
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            })
        };
    grammar_config
        .scanner_configurations
        .iter_mut()
        .try_for_each(|sc| {
            insert_transitions(
                sc,
                &parol_grammar.scanner_configurations,
                &terminal_resolver,
                scanner_resolver,
                &terminal_finder,
            )
        })?;

    Ok(grammar_config)
}

fn insert_transitions(
    sc: &mut generators::ScannerConfig,
    scanner_configurations: &[crate::parser::parol_grammar::ScannerConfig],
    terminal_resolver: &impl crate::grammar::cfg::TerminalIndexFn,
    scanner_resolver: impl Fn(&str) -> Option<usize>,
    terminal_finder: impl Fn(&str) -> Option<(String, TerminalKind, Option<LookaheadExpression>)>,
) -> Result<()> {
    if let Some(source_configuartion) = scanner_resolver(&sc.scanner_name) {
        let mut transitions = Vec::new();
        scanner_configurations[source_configuartion]
            .transitions
            .iter()
            .try_for_each(|(token, target_state_name)| {
                if let Some((txt, kind, la)) = terminal_finder(token.text()) {
                    if let Some(target_scanner) = scanner_resolver(target_state_name.text()) {
                        transitions.push((
                            terminal_resolver.terminal_index(&txt, kind, &la),
                            target_scanner,
                        ));
                    } else {
                        bail!(
                            "Target scanner configuration {} not found",
                            target_state_name
                        );
                    }
                } else {
                    bail!("Terminal {} not found", token);
                }
                Ok(())
            })?;
        transitions.sort_by(|a, b| a.0.cmp(&b.0));
        sc.transitions = transitions;
    } else {
        bail!("Scanner configuration {} not found", sc.scanner_name);
    }
    Ok(())
}

fn try_from_scanner_config(
    sc: &crate::parser::parol_grammar::ScannerConfig,
    scanner_state: usize,
) -> Result<ScannerConfig> {
    let scanner_config = ScannerConfig::new(sc.name.clone(), scanner_state)
        .with_line_comments(sc.line_comments.clone())
        .with_block_comments(sc.block_comments.clone())
        .with_auto_newline(!sc.auto_newline_off)
        .with_auto_ws(!sc.auto_ws_off);
    Ok(scanner_config)
}

pub(crate) fn try_from_factor(factor: Factor) -> Result<Symbol> {
    match factor {
        Factor::NonTerminal(n, a, u, m) => {
            // We use the member name here if given
            Ok(Symbol::N(n, a, u, m))
        }
        Factor::Terminal(t, k, s, a, u, m, l) => {
            // We use the member name here if given
            Ok(Symbol::T(Terminal::Trm(t, k, s, a, u, m, l)))
        }
        Factor::ScannerSwitch(s, _) => Ok(Symbol::s(s)),
        Factor::ScannerSwitchPush(s, _) => Ok(Symbol::Push(s)),
        Factor::ScannerSwitchPop(_) => Ok(Symbol::Pop),
        _ => bail!("Unexpected type of factor: {}", factor),
    }
}
