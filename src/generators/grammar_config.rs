use super::ScannerConfig;
use crate::parser::{try_to_convert, ParolGrammar};
use crate::Cfg;
use std::convert::TryFrom;
use std::fmt::{Debug, Display, Error, Formatter};

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
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
    /// At least one scanner configurations
    ///
    pub scanner_configurations: Vec<ScannerConfig>,

    ///
    /// The maximum lookahead size, used for lexer generation
    ///
    pub lookahead_size: usize,
}

impl GrammarConfig {
    /// Creates a new item
    pub fn new(cfg: Cfg, lookahead_size: usize) -> Self {
        Self {
            cfg,
            title: None,
            comment: None,
            scanner_configurations: Vec::new(),
            lookahead_size,
        }
    }

    /// Sets an optional title
    pub fn with_title(mut self, title: Option<String>) -> Self {
        self.title = title;
        self
    }

    /// Sets an optional comment
    pub fn with_comment(mut self, comment: Option<String>) -> Self {
        self.comment = comment;
        self
    }

    /// Adds a scanner configuration
    pub fn add_scanner(mut self, scanner_config: ScannerConfig) -> Self {
        self.scanner_configurations.push(scanner_config);
        self
    }

    /// Sets the lookahead size
    pub fn update_lookahead_size(&mut self, k: usize) {
        self.lookahead_size = k;
    }

    /// Sets the cfg member
    pub fn update_cfg(&mut self, cfg: Cfg) {
        self.cfg = cfg;
    }

    ///
    /// Generates the augmented tokens vector in the format needed by the lexer
    /// generator.
    ///
    pub fn generate_augmented_terminals(&self) -> Vec<String> {
        let terminals = vec![
            "UNMATCHABLE_TOKEN".to_owned(),
            "UNMATCHABLE_TOKEN".to_owned(),
            "UNMATCHABLE_TOKEN".to_owned(),
            "UNMATCHABLE_TOKEN".to_owned(),
            "UNMATCHABLE_TOKEN".to_owned(),
        ];
        let mut terminals =
            self.cfg
                .get_ordered_terminals()
                .iter()
                .fold(terminals, |mut acc, (t, _)| {
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
            scanner_configurations: vec![ScannerConfig::default()], // There must always be a default scanner
            lookahead_size: 0,
        }
    }
}

impl Display for GrammarConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        writeln!(f, "title: {:?}", self.title)?;
        writeln!(f, "comment: {:?}", self.comment)?;
        for sc in &self.scanner_configurations {
            writeln!(f, "{}", sc)?;
        }
        writeln!(f, "cfg: {:?}", self.cfg)
    }
}

impl TryFrom<ParolGrammar> for GrammarConfig {
    type Error = miette::Error;
    fn try_from(grammar: ParolGrammar) -> miette::Result<Self> {
        try_to_convert(grammar)
    }
}

#[cfg(test)]
mod test {
    use crate::generators::{GrammarConfig, ScannerConfig};
    use crate::{Cfg, Pr, Symbol};

    #[test]
    fn check_generate_augmented_terminals() {
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

        let scanner_config = ScannerConfig::default()
            .with_line_comments(vec!["//".to_owned()])
            .with_block_comments(vec![(r#"/\*"#.to_owned(), r#"\*/"#.to_owned())]);

        let grammar_config = GrammarConfig::new(g, 1)
            .with_title(title)
            .with_comment(comment)
            .add_scanner(scanner_config);

        let augment_terminals = grammar_config.generate_augmented_terminals();

        assert_eq!(
            vec![
                "UNMATCHABLE_TOKEN",
                "UNMATCHABLE_TOKEN",
                "UNMATCHABLE_TOKEN",
                "UNMATCHABLE_TOKEN",
                "UNMATCHABLE_TOKEN",
                r###"a"###,
                r###"b"###,
                "ERROR_TOKEN"
            ]
            .iter()
            .map(|t| (*t).to_owned())
            .collect::<Vec<String>>(),
            augment_terminals
        );

        let (special_terminals, terminal_indices, scanner_name) = grammar_config
            .scanner_configurations[0]
            .generate_build_information(&grammar_config.cfg);

        assert_eq!(
            vec![
                "UNMATCHABLE_TOKEN",
                "NEW_LINE_TOKEN",
                "WHITESPACE_TOKEN",
                r###"(//.*(\r\n|\r|\n|$))"###,
                r###"((?ms)/\*.*?\*/)"###,
            ]
            .iter()
            .map(|t| (*t).to_owned())
            .collect::<Vec<String>>(),
            special_terminals
        );

        assert_eq!(vec![5, 6], terminal_indices);

        assert_eq!("INITIAL", scanner_name);
    }
}
