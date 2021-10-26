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
/// Newly added feature is to optionally switch automatic handling off newlines off.
///
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
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
    /// Strings with the characters that starts line comments
    ///
    pub line_comments: Vec<String>,

    ///
    /// (String, String) tuples with the characters that start and end
    /// a block comments, respectively.
    ///
    pub block_comments: Vec<(String, String)>,

    ///
    /// If true the lexer handles (and skips) newlines.
    /// If false the user has to handle newlines on its own.
    ///
    pub auto_newline: bool,

    ///
    /// The maximum lookahead size, used for lexer generation
    ///
    pub lookahead_size: usize,
}

impl GrammarConfig {
    pub fn new(cfg: Cfg, lookahead_size: usize) -> Self {
        Self {
            cfg,
            title: None,
            comment: None,
            line_comments: Vec::new(),
            block_comments: Vec::new(),
            auto_newline: true,
            lookahead_size,
        }
    }

    pub fn with_title(mut self, title: Option<String>) -> Self {
        self.title = title;
        self
    }

    pub fn with_comment(mut self, comment: Option<String>) -> Self {
        self.comment = comment;
        self
    }

    pub fn with_line_comments(mut self, line_comments: Vec<String>) -> Self {
        self.line_comments = line_comments;
        self
    }

    pub fn with_block_comments(mut self, block_comments: Vec<(String, String)>) -> Self {
        self.block_comments = block_comments;
        self
    }

    pub fn with_auto_newline(mut self, auto_newline: bool) -> Self {
        self.auto_newline = auto_newline;
        self
    }

    pub fn update_lookahead_size(&mut self, k: usize) {
        self.lookahead_size = k;
    }

    pub fn update_cfg(&mut self, cfg: Cfg) {
        self.cfg = cfg;
    }

    ///
    /// Generates the augmented tokens vector in the format needed by the lexer
    /// generator.
    ///
    pub fn generate_augmented_terminals(&self) -> Vec<String> {
        let mut terminals = vec![
            "UNMATCHABLE_TOKEN".to_owned(),
            if self.auto_newline {
                "NEW_LINE_TOKEN".to_owned()
            } else {
                "UNMATCHABLE_TOKEN".to_owned()
            },
            "WHITESPACE_TOKEN".to_owned(),
        ];
        if !self.line_comments.is_empty() {
            let line_comments_rx = self
                .line_comments
                .iter()
                .map(|s| format!(r###"({}.*(\r\n|\r|\n|$))"###, s))
                .collect::<Vec<String>>()
                .join("|");
            terminals.push(line_comments_rx);
        } else {
            terminals.push("UNMATCHABLE_TOKEN".to_owned());
        }
        if !self.block_comments.is_empty() {
            let block_comments_rx = self
                .block_comments
                .iter()
                .map(|(s, e)| format!(r###"((?ms){}.*?{})"###, s, e))
                .collect::<Vec<String>>()
                .join("|");
            terminals.push(block_comments_rx);
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
}

impl Default for GrammarConfig {
    fn default() -> Self {
        Self {
            cfg: Cfg::default(),
            title: None,
            comment: None,
            line_comments: Vec::new(),
            block_comments: Vec::new(),
            auto_newline: true,
            lookahead_size: 0,
        }
    }
}

impl Display for GrammarConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        writeln!(f, "title: {:?}", self.title)?;
        writeln!(f, "comment: {:?}", self.comment)?;
        writeln!(f, "line_comments: {:?}", self.line_comments)?;
        writeln!(f, "block_comments: {:?}", self.block_comments)?;
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

        let grammar_config = GrammarConfig::new(g, 1)
            .with_title(title)
            .with_comment(comment)
            .with_line_comments(vec!["//".to_owned()])
            .with_block_comments(vec![(r#"/\*"#.to_owned(), r#"\*/"#.to_owned())]);
        let augment_terminals = grammar_config.generate_augmented_terminals();

        assert_eq!(
            vec![
                "UNMATCHABLE_TOKEN",
                "NEW_LINE_TOKEN",
                "WHITESPACE_TOKEN",
                r###"(//.*(\r\n|\r|\n|$))"###,
                r###"((?ms)/\*.*?\*/)"###,
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
