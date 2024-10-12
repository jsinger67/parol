//!
//! The module contains the conversion to a the PAR format.
//!
use crate::{
    generators::grammar_config::FnScannerStateResolver, grammar::cfg::FnPrimaryNonTerminalFinder,
    group_by, parser::parol_grammar::GrammarType, GrammarConfig, ScannerConfig, StrVec,
};
use anyhow::Result;

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
///
/// Formats the given GrammarConfig in the PAR format.
///
pub fn render_par_string(
    grammar_config: &GrammarConfig,
    add_index_comment: bool,
) -> Result<String> {
    let title = grammar_config
        .title
        .as_ref()
        .map_or("".to_owned(), |title| format!("%title \"{}\"\n", title));

    let comment = grammar_config
        .comment
        .as_ref()
        .map_or("".to_owned(), |comment| {
            format!("%comment \"{}\"\n", comment)
        });

    let grammar_type = match grammar_config.grammar_type {
        // For compatibility reasons we do not output the grammar type for LLK grammars
        // This is no problem as the default is LLK
        GrammarType::LLK => "".to_owned(), // "\n%grammar_type 'll(k)'".to_owned(),
        GrammarType::LALR1 => "%grammar_type 'lalr(1)'\n".to_owned(),
    };

    let scanner_state_resolver = grammar_config.get_scanner_state_resolver();
    let primary_non_terminal_finder = grammar_config.cfg.get_primary_non_terminal_finder();

    let initial_scanner_state = render_scanner_config_string(
        0,
        &grammar_config.scanner_configurations[0],
        &scanner_state_resolver,
        &primary_non_terminal_finder,
    );

    let user_types = grammar_config
        .user_type_defs
        .iter()
        .fold(String::new(), |mut acc, (a, u)| {
            acc.push_str(&format!("%user_type {} = {}\n", a, u));
            acc
        });

    let user_type_resolver = grammar_config.get_user_type_resolver();

    let mut productions =
        grammar_config
            .cfg
            .pr
            .iter()
            .try_fold(Vec::new(), |mut acc: Vec<String>, p| {
                p.format(&scanner_state_resolver, &user_type_resolver)
                    .map(|s| {
                        acc.push(s);
                        acc
                    })
            })?;

    if add_index_comment {
        let width = (productions.len() as f32).log10() as usize + 1;
        productions = productions
            .drain(..)
            .enumerate()
            .map(|(i, p)| format!("/* {:w$} */ {}", i, p, w = width))
            .collect();
    }

    let mut scanner_states = grammar_config
        .scanner_configurations
        .iter()
        .enumerate()
        .skip(1)
        .fold(String::new(), |mut acc, (i, e)| {
            acc.push_str(&render_scanner_config_string(
                i,
                e,
                &scanner_state_resolver,
                &primary_non_terminal_finder,
            ));
            acc.push('\n');
            acc
        });

    if !scanner_states.is_empty() {
        scanner_states.push('\n');
    }

    let productions = productions.drain(..).fold(StrVec::new(0), |mut acc, e| {
        acc.push(e);
        acc
    });

    let start_symbol = grammar_config.cfg.st.clone();
    Ok(format!(
        "%start {start_symbol}
{title}{comment}{grammar_type}{initial_scanner_state}{user_types}
{scanner_states}%%

{productions}"
    ))
}

fn render_scanner_config_string(
    index: usize,
    scanner_config: &ScannerConfig,
    scanner_state_resolver: &FnScannerStateResolver,
    primary_non_terminal_finder: &FnPrimaryNonTerminalFinder,
) -> String {
    let scanner_name = &scanner_config.scanner_name;

    let mut scanner_directives = String::with_capacity(1024); // Start capacity with 1KB
    let indent = if index == crate::parser::parol_grammar::INITIAL_STATE {
        ""
    } else {
        "    "
    };

    for c in &scanner_config.line_comments {
        scanner_directives.push_str(&format!("{}%line_comment \"{}\"\n", indent, c));
    }

    for (s, e) in &scanner_config.block_comments {
        scanner_directives.push_str(&format!("{}%block_comment \"{}\" \"{}\"\n", indent, s, e));
    }

    if !scanner_config.auto_newline {
        scanner_directives.push_str(&format!("{}%auto_newline_off\n", indent));
    }

    if !scanner_config.auto_ws {
        scanner_directives.push_str(&format!("{}%auto_ws_off\n", indent));
    }

    for (scanner, primary_nts) in group_by(&scanner_config.transitions, |(_, v)| *v) {
        let mut primary_nts = primary_nts
            .iter()
            .map(|(k, _)| primary_non_terminal_finder(*k).unwrap_or(format!("{}", k)))
            .collect::<Vec<_>>();
        primary_nts.sort();
        scanner_directives.push_str(&format!(
            "{}%on {} %enter {}\n",
            indent,
            primary_nts.join(", "),
            scanner_state_resolver(&[scanner])
        ));
    }

    if index == crate::parser::parol_grammar::INITIAL_STATE {
        scanner_directives
    } else {
        if !scanner_directives.is_empty() {
            scanner_directives.insert(0, '\n');
        }
        format!("%scanner {} {{{}}}", scanner_name, scanner_directives)
    }
}

#[cfg(test)]
mod test {
    use crate::conversions::par::render_par_string;
    use crate::{
        Cfg, GrammarConfig, Pr, ScannerConfig, Symbol, SymbolAttribute, Terminal, TerminalKind,
    };
    use pretty_assertions::assert_eq;

    macro_rules! terminal {
        ($term:literal) => {
            Symbol::T(Terminal::Trm(
                $term.to_string(),
                TerminalKind::Legacy,
                vec![0],
                SymbolAttribute::None,
                None,
                None,
            ))
        };
    }

    #[test]
    fn check_par_format() {
        let g = Cfg::with_start_symbol("S")
            .add_pr(Pr::new("S", vec![terminal!("a"), Symbol::n("X")]))
            .add_pr(Pr::new("X", vec![terminal!("b"), Symbol::n("S")]))
            .add_pr(Pr::new(
                "X",
                vec![
                    terminal!("a"),
                    Symbol::n("Y"),
                    terminal!("b"),
                    Symbol::n("Y"),
                ],
            ))
            .add_pr(Pr::new("Y", vec![terminal!("b"), terminal!("a")]))
            .add_pr(Pr::new("Y", vec![terminal!("a"), Symbol::n("Z")]))
            .add_pr(Pr::new(
                "Z",
                vec![terminal!("a"), Symbol::n("Z"), Symbol::n("X")],
            ));

        let title = Some("Test grammar".to_owned());
        let comment = Some("A simple grammar".to_owned());

        let grammar_config = GrammarConfig::new(g, 1)
            .with_title(title)
            .with_comment(comment)
            .add_scanner(ScannerConfig::default());

        let par_str = render_par_string(&grammar_config, true).unwrap();
        let par_str = par_str.replace("\r\n", "\n");
        let expected = r#"%start S
%title "Test grammar"
%comment "A simple grammar"

%%

/* 0 */ S: "a" X;
/* 1 */ X: "b" S;
/* 2 */ X: "a" Y "b" Y;
/* 3 */ Y: "b" "a";
/* 4 */ Y: "a" Z;
/* 5 */ Z: "a" Z X;
"#;
        let expected = expected.replace("\r\n", "\n");
        assert_eq!(expected, par_str);
    }
}
