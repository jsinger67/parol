use crate::parser::{Factor, ParolGrammar};
use crate::transformation::transform_productions;
use crate::{Cfg, GrammarConfig, ScannerConfig, Symbol, Terminal};
use anyhow::{bail, Result};

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

    Ok(grammar_config)
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
        Factor::NonTerminal(n, a, u) => Ok(Symbol::N(n, a, u)),
        Factor::Terminal(t, k, s, a, u) => Ok(Symbol::T(Terminal::Trm(t, k, s, a, u))),
        Factor::ScannerSwitch(s) => Ok(Symbol::s(s)),
        Factor::ScannerSwitchPush(s) => Ok(Symbol::Push(s)),
        Factor::ScannerSwitchPop => Ok(Symbol::Pop),
        _ => bail!("Unexpected type of factor: {}", factor),
    }
}
