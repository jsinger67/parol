use crate::parser::{try_to_convert, ParolGrammar};
use crate::Cfg;
use std::convert::TryFrom;
use std::fmt::{Debug, Display, Error, Formatter};

///
/// Configuration information for a context free grammar.
/// Currently containing comment tokens and maximum lookahead size.
/// Can later be amended with further information and even pragmas that control
/// the generation of lexers and parsers.
/// Examples for amendments:
///     * Ignore case, generate a case-insensitive lexer
///     * Parse whitespace instead of skipping them by default
///     * Prologue and epilogue for generated parser output
///
#[derive(Debug, Default, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct GrammarConfig {
    ///
    /// The actual context free grammar.
    /// It should be checked and left-factored here.
    /// For this task use the
    /// ´generators::grammar_trans::check_and_transform_grammar`
    /// function to prepare the grammar.
    ///
    pub cfg: Cfg,

    ///
    /// Title of the grammar
    ///
    pub title: Option<String>,

    ///
    /// Optional comment
    ///
    pub comment: Option<String>,

    ///
    /// Optional String with the characters that starts a line comment
    ///
    pub line_comment: Option<String>,

    ///
    /// Optional (String, String) tuple with the characters that start and end
    /// a block comment, respectively.
    ///
    pub block_comment: Option<(String, String)>,

    ///
    /// The maximum lookahead size, used for lexer generation
    ///
    pub lookahead_size: usize,
}

impl GrammarConfig {
    pub fn new(
        cfg: Cfg,
        title: Option<String>,
        comment: Option<String>,
        line_comment: Option<String>,
        block_comment: Option<(String, String)>,
        lookahead_size: usize,
    ) -> Self {
        Self {
            cfg,
            title,
            comment,
            line_comment,
            block_comment,
            lookahead_size,
        }
    }

    ///
    /// Generates the augmented tokens vector in the format needed by the lexer
    /// generator.
    ///
    pub fn generate_augmented_terminals(&self) -> Vec<String> {
        let mut terminals = vec![
            "UNMATCHABLE_TOKEN".to_owned(),
            "NEW_LINE_TOKEN".to_owned(),
            "WHITESPACE_TOKEN".to_owned(),
        ];
        if let Some(line_comment) = &self.line_comment {
            let line_comment_rx = format!(r###"{}.*"###, line_comment);
            terminals.push(line_comment_rx);
        } else {
            terminals.push("UNMATCHABLE_TOKEN".to_owned());
        }
        if let Some((block_start, block_end)) = &self.block_comment {
            let block_comment_rx = format!(r###"(?ms){}.*?{}"###, block_start, block_end);
            terminals.push(block_comment_rx);
        } else {
            terminals.push("UNMATCHABLE_TOKEN".to_owned());
        }

        let mut terminals =
            self.cfg
                .get_ordered_terminals()
                .iter()
                .fold(terminals, |mut acc, t| {
                    acc.push(t.to_string());
                    acc
                });

        terminals.push("ERROR_TOKEN".to_owned());
        terminals
    }

    pub fn update_lookahead_size(&mut self, k: usize) {
        self.lookahead_size = k;
    }

    pub fn update_cfg(&mut self, cfg: Cfg) {
        self.cfg = cfg;
    }
}

impl Display for GrammarConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        writeln!(f, "title: {:?}", self.title)?;
        writeln!(f, "comment: {:?}", self.comment)?;
        writeln!(f, "line_comment: {:?}", self.line_comment)?;
        writeln!(f, "block_comment: {:?}", self.block_comment)?;
        writeln!(f, "cfg: {:?}", self.cfg)
    }
}

impl TryFrom<ParolGrammar> for GrammarConfig {
    type Error = crate::errors::Error;
    fn try_from(grammar: ParolGrammar) -> crate::errors::Result<Self> {
        try_to_convert(grammar)
    }
}

#[cfg(test)]
mod test {
    use crate::generators::grammar_config::GrammarConfig;
    use crate::{Cfg, Pr, Symbol};

    #[test]
    fn check_generate_augmented_terminals() {
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

        let grammar_config = GrammarConfig::new(
            g,
            title,
            comment,
            Some("//".to_owned()),
            Some((r#"/\*"#.to_owned(), r#"\*/"#.to_owned())),
            1,
        );
        let augment_terminals = grammar_config.generate_augmented_terminals();

        assert_eq!(
            vec![
                "UNMATCHABLE_TOKEN",
                "NEW_LINE_TOKEN",
                "WHITESPACE_TOKEN",
                r###"//.*"###,
                r###"(?ms)/\*.*?\*/"###,
                r###"a"###,
                r###"b"###,
                "ERROR_TOKEN"
            ]
            .iter()
            .map(|t| (*t).to_owned())
            .collect::<Vec<String>>(),
            augment_terminals
        );
    }
}
