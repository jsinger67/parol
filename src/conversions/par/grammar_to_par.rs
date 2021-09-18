//!
//! The module contains the conversion to a the PAR format.
//!
use crate::{GrammarConfig, StrVec};
use std::fmt::Debug;

#[derive(BartDisplay, Debug, Default)]
#[template = "templates/par_template.par"]
struct YaccElements {
    start_symbol: String,
    title: String,
    comment: String,
    line_comment: String,
    block_comment: String,
    productions: StrVec,
}

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

    let line_comment = if let Some(line_comment) = grammar_config.line_comment.as_ref() {
        format!("\n%line_comment \"{}\"", line_comment)
    } else {
        "".to_owned()
    };

    let block_comment = if let Some(block_comment) = grammar_config.block_comment.as_ref() {
        format!(
            "\n%block_comment \"{}\" \"{}\"",
            block_comment.0, block_comment.1
        )
    } else {
        "".to_owned()
    };

    let mut productions = Vec::new();

    grammar_config.cfg.pr.iter().for_each(|p| {
        productions.push(format!("{}", p));
    });

    if add_index_comment {
        let width = (productions.len() as f32).log10() as usize + 1;
        productions = productions
            .drain(..)
            .enumerate()
            .map(|(i, p)| format!("/* {:w$} */ {}", i, p, w = width))
            .collect();
    }

    let productions = productions.drain(..).fold(StrVec::new(0), |mut acc, e| {
        acc.push(e);
        acc
    });

    let elements = YaccElements {
        start_symbol: grammar_config.cfg.st.clone(),
        title,
        comment,
        line_comment,
        block_comment,
        productions,
    };
    format!("{}", elements)
}

#[cfg(test)]
mod test {
    use crate::conversions::par::render_par_string;
    use crate::{Cfg, GrammarConfig, Pr, Symbol};

    #[test]
    fn check_par_format() {
        let g = Cfg::with_start_symbol("S")
            .add_pr(Pr::new("S", vec![Symbol::t("a"), Symbol::n("X")]))
            .add_pr(Pr::new("X", vec![Symbol::t("b"), Symbol::n("S")]))
            .add_pr(Pr::new(
                "X",
                vec![
                    Symbol::t("a"),
                    Symbol::n("Y"),
                    Symbol::t("b"),
                    Symbol::n("Y"),
                ],
            ))
            .add_pr(Pr::new("Y", vec![Symbol::t("b"), Symbol::t("a")]))
            .add_pr(Pr::new("Y", vec![Symbol::t("a"), Symbol::n("Z")]))
            .add_pr(Pr::new(
                "Z",
                vec![Symbol::t("a"), Symbol::n("Z"), Symbol::n("X")],
            ));

        let title = Some("Test grammar".to_owned());
        let comment = Some("A simple grammar".to_owned());

        let grammar_config = GrammarConfig::new(g, title, comment, None, None, 1);

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
