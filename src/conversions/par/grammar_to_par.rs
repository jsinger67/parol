//!
//! The module contains the conversion to a the PAR format.
//!
use crate::{GrammarConfig, ScannerConfig, StrVec};
use std::fmt::Debug;

#[derive(BartDisplay, Debug, Default)]
#[template = "templates/par_template.par"]
struct YaccElements {
    start_symbol: String,
    title: String,
    comment: String,
    line_comments: String,
    block_comments: String,
    auto_newline_off: String,
    auto_ws_off: String,
    scanner_states: StrVec,
    productions: StrVec,
}

#[derive(BartDisplay, Debug, Default)]
#[template = "templates/scanner_config_template.par"]
struct ScannerConfigElements {
    scanner_name: String,
    scanner_directives: String,
}

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
///
/// Formats the given GrammarConfig in the PAR format.
///
pub fn render_par_string(grammar_config: &GrammarConfig, add_index_comment: bool) -> String {
    let title = format!(
        "\n%title \"{}\"",
        grammar_config.title.clone().unwrap_or_default()
    );

    let comment = if let Some(comment) = grammar_config.comment.as_ref() {
        format!("\n%comment \"{}\"", comment)
    } else {
        "".to_owned()
    };

    let line_comments = grammar_config.scanner_configurations[0]
        .line_comments
        .iter()
        .map(|c| format!("\n%line_comment \"{}\"", c))
        .collect::<Vec<String>>()
        .join("\n");

    let block_comments = grammar_config.scanner_configurations[0]
        .block_comments
        .iter()
        .map(|(s, e)| format!("\n%block_comment \"{}\" \"{}\"", s, e))
        .collect::<Vec<String>>()
        .join("\n");

    let auto_newline_off = if grammar_config.scanner_configurations[0].auto_newline {
        String::new()
    } else {
        "\n%auto_newline_off".to_owned()
    };

    let auto_ws_off = if grammar_config.scanner_configurations[0].auto_ws {
        String::new()
    } else {
        "\n%auto_ws_off".to_owned()
    };

    let scanner_state_resolver = grammar_config.get_scanner_state_resolver();

    let mut productions = Vec::new();

    grammar_config.cfg.pr.iter().for_each(|p| {
        productions.push(p.format(&scanner_state_resolver));
    });

    if add_index_comment {
        let width = (productions.len() as f32).log10() as usize + 1;
        productions = productions
            .drain(..)
            .enumerate()
            .map(|(i, p)| format!("/* {:w$} */ {}", i, p, w = width))
            .collect();
    }

    let mut scanner_states =
        grammar_config
            .scanner_configurations
            .iter()
            .skip(1)
            .fold(StrVec::new(0), |mut acc, e| {
                acc.push(render_scanner_config_string(e));
                acc
            });

    if !scanner_states.is_empty() {
        // Add a separator to beautify the output if there exist additional scanner states.
        scanner_states.push(String::default());
    }

    let productions = productions.drain(..).fold(StrVec::new(0), |mut acc, e| {
        acc.push(e);
        acc
    });

    let elements = YaccElements {
        start_symbol: grammar_config.cfg.st.clone(),
        title,
        comment,
        line_comments,
        block_comments,
        auto_newline_off,
        auto_ws_off,
        scanner_states,
        productions,
    };
    format!("{}", elements)
}

fn render_scanner_config_string(scanner_config: &ScannerConfig) -> String {
    let scanner_name = scanner_config.scanner_name.clone();

    let mut scanner_directives = Vec::<String>::new();

    let line_comments = scanner_config
        .line_comments
        .iter()
        .map(|c| format!("%line_comment \"{}\"", c))
        .collect::<Vec<String>>()
        .join(" ");

    if !line_comments.is_empty() {
        scanner_directives.push(line_comments);
    }

    let block_comments = scanner_config
        .block_comments
        .iter()
        .map(|(s, e)| format!("%block_comment \"{}\" \"{}\"", s, e))
        .collect::<Vec<String>>()
        .join(" ");

    if !block_comments.is_empty() {
        scanner_directives.push(block_comments);
    }

    if !scanner_config.auto_newline {
        scanner_directives.push("%auto_newline_off".to_owned());
    }

    if !scanner_config.auto_ws {
        scanner_directives.push("%auto_ws_off".to_owned());
    }

    let elements = ScannerConfigElements {
        scanner_name,
        scanner_directives: scanner_directives.join(" "),
    };

    format!("{}", elements)
}

#[cfg(test)]
mod test {
    use crate::conversions::par::render_par_string;
    use crate::{Cfg, GrammarConfig, Pr, ScannerConfig, Symbol};

    #[test]
    fn check_par_format() {
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

        let title = Some("Test grammar".to_owned());
        let comment = Some("A simple grammar".to_owned());

        let grammar_config = GrammarConfig::new(g, 1)
            .with_title(title)
            .with_comment(comment)
            .add_scanner(ScannerConfig::default());

        let par_str = render_par_string(&grammar_config, true);
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
