use crate::parser::{Factor, ParolGrammar};
use crate::transformation::transform_productions;
use crate::{Cfg, GrammarConfig, ScannerConfig, Symbol};
use miette::{miette, Result};

pub(crate) fn try_to_convert(parol_grammar: ParolGrammar) -> Result<GrammarConfig> {
    let st = parol_grammar.start_symbol;
    let pr = transform_productions(parol_grammar.item_stack)?;
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
        .with_comment(comment)
        .add_scanner(scanner_config);

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
        Factor::NonTerminal(n, a) => Ok(Symbol::N(n, a)),
        Factor::Terminal(t, s) => Ok(Symbol::t(&t, s)),
        Factor::ScannerSwitch(s) => Ok(Symbol::s(s)),
        Factor::ScannerSwitchPush(s) => Ok(Symbol::Push(s)),
        Factor::ScannerSwitchPop => Ok(Symbol::Pop),
        _ => Err(miette!("Unexpected type of factor: {}", factor)),
    }
}
