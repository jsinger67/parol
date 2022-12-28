use parol_runtime::once_cell::sync::Lazy;

use super::ScannerConfig;
use crate::parser::try_to_convert;
use crate::{Cfg, ParolGrammar};
use anyhow::Result;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt::{Debug, Display, Error, Formatter};

/// Used for implementation of trait `Default` for `&GrammarConfig`.
static DEFAULT_GRAMMAR_CONFIG: Lazy<GrammarConfig> = Lazy::new(GrammarConfig::default);

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
///
/// Configuration information for a context free grammar.
/// Currently containing the CFG itself along with accompanying information as title and comment of
/// the grammar, user type aliases, maximum lookahead size and a list of scanner configurations.
/// Can later be amended with further information or pragmas that control lexer/parser generation.
///
#[derive(Debug, Clone)]
pub struct GrammarConfig {
    ///
    /// The actual context free grammar.
    /// It should be checked and left-factored here.
    /// For this task use the
    /// [generators::grammar_trans::check_and_transform_grammar]
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
    /// User type definitions
    ///
    pub user_type_defs: Vec<(String, String)>,

    ///
    /// At least one scanner configurations
    ///
    pub scanner_configurations: Vec<ScannerConfig>,

    ///
    /// The maximum lookahead size, used for lexer generation
    ///
    pub lookahead_size: usize,
}

/// The type of a scanner state resolver function.
/// A scanner state resolver function translates a list of scanner states into a printable string
pub(crate) type FnScannerStateResolver = Box<dyn Fn(&[usize]) -> String>;

/// The type of a user type resolver function.
/// A user type resolver function translates a decorated user type name into its shorter alias
pub(crate) type FnUserTypeResolver = Box<dyn Fn(&str) -> Option<String>>;

impl GrammarConfig {
    /// Creates a new item
    pub fn new(cfg: Cfg, lookahead_size: usize) -> Self {
        Self {
            cfg,
            title: None,
            comment: None,
            user_type_defs: Vec::new(),
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

    /// Adds a user type definition
    pub fn add_user_type_def(mut self, alias: String, type_name: String) -> Self {
        self.user_type_defs.push((alias, type_name));
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
                .into_iter()
                .fold(terminals, |mut acc, (t, k, _)| {
                    acc.push(k.expand(t));
                    acc
                });
        terminals.push("ERROR_TOKEN".to_owned());
        terminals
    }

    /// Generates a function that can be used as scanner_state_resolver argument on Pr::format
    pub fn get_scanner_state_resolver(&self) -> FnScannerStateResolver {
        let scanner_names = self
            .scanner_configurations
            .iter()
            .map(|s| s.scanner_name.clone())
            .collect::<Vec<String>>();
        Box::new(move |s: &[usize]| {
            s.iter()
                .map(|s| scanner_names[*s].clone())
                .collect::<Vec<String>>()
                .join(", ")
        })
    }

    /// Generates a dummy scanner_state_resolver function that can be used in Pr::format
    pub fn dummy_scanner_state_resolver() -> FnScannerStateResolver {
        Box::new(move |s: &[usize]| {
            s.iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        })
    }

    /// Generates a function that can be used as user_type_resolver argument on Pr::format
    pub fn get_user_type_resolver(&self) -> FnUserTypeResolver {
        let user_type_map = self
            .user_type_defs
            .iter()
            .fold(HashMap::new(), |mut acc, (a, u)| {
                acc.insert(u.to_string(), a.to_string());
                acc
            });
        Box::new(move |u: &str| user_type_map.get(u).cloned())
    }

    /// Generates a dummy user_type_resolver function that can be used in Pr::format
    pub fn dummy_user_type_resolver() -> FnUserTypeResolver {
        Box::new(|_u: &str| None)
    }
}

impl Default for GrammarConfig {
    fn default() -> Self {
        Self {
            cfg: Cfg::default(),
            title: None,
            comment: None,
            user_type_defs: Vec::new(),
            scanner_configurations: vec![ScannerConfig::default()], // There must always be a default scanner
            lookahead_size: 0,
        }
    }
}

impl Default for &GrammarConfig {
    fn default() -> Self {
        &DEFAULT_GRAMMAR_CONFIG
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

impl TryFrom<ParolGrammar<'_>> for GrammarConfig {
    type Error = anyhow::Error;
    fn try_from(grammar: ParolGrammar) -> Result<Self> {
        try_to_convert(grammar)
    }
}

#[cfg(test)]
mod test {
    use crate::generators::{GrammarConfig, ScannerConfig};
    use crate::{
        obtain_grammar_config_from_string, Cfg, Pr, Symbol, SymbolAttribute, Terminal, TerminalKind,
    };

    macro_rules! terminal {
        ($term:literal) => {
            Symbol::T(Terminal::Trm(
                $term.to_string(),
                TerminalKind::Legacy,
                vec![0],
                SymbolAttribute::None,
                None,
            ))
        };
    }

    macro_rules! augmented_terminals {
        ($($term:literal),+) => {
            &[
                "UNMATCHABLE_TOKEN",
                "UNMATCHABLE_TOKEN",
                "UNMATCHABLE_TOKEN",
                "UNMATCHABLE_TOKEN",
                "UNMATCHABLE_TOKEN",
                $($term,)*
                "ERROR_TOKEN",
            ]
        };
    }

    #[test]
    fn check_generate_augmented_terminals() {
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

    #[derive(Debug)]
    struct TestData {
        input: &'static str,
        augment_terminals: &'static [&'static str],
    }

    const TESTS: &[TestData] = &[
        TestData {
            input: r#"%start A %% A: B "r"; B: C "d"; C: A "t";"#,
            augment_terminals: augmented_terminals!["r", "d", "t"],
        },
        TestData {
            input: r#"%start A %% A: B /r/; B: C "d"; C: A 't';"#,
            augment_terminals: augmented_terminals!["r", "d", "t"],
        },
        TestData {
            input: r#"%start A %% A: B /r/; B: C "d"; C: A 'd';"#,
            augment_terminals: augmented_terminals!["r", "d", "d"],
        },
        TestData {
            input: r#"%start A %% A: B /r/; B: C "d"; C: A "d";"#,
            augment_terminals: augmented_terminals!["r", "d"],
        },
        TestData {
            input: r#"%start A %% A: B /r/; B: C "d"; C: A /d/;"#,
            augment_terminals: augmented_terminals!["r", "d"],
        },
        TestData {
            input: r#"%start A %% A: B /r/; B: C 'd'; C: A 'd';"#,
            augment_terminals: augmented_terminals!["r", "d"],
        },
        TestData {
            input: r#"%start A %% A: B /r/; B: C /d/; C: A /d/;"#,
            augment_terminals: augmented_terminals!["r", "d"],
        },
        TestData {
            input: r#"%start A %% A: B /r/; B: C /\d/; C: A /d/;"#,
            augment_terminals: augmented_terminals!["r", r"\d", "d"],
        },
        TestData {
            input: r#"%start A %% A: B /r/; B: C /\s/; C: A /d/;"#,
            augment_terminals: augmented_terminals!["r", r"\s", "d"],
        },
        TestData {
            input: r#"%start A %% A: B /r/; B: C '\s'; C: A /d/;"#,
            augment_terminals: augmented_terminals!["r", r"\\s", "d"],
        },
    ];

    #[test]
    fn check_generate_augmented_terminals_generic() {
        for (i, test) in TESTS.iter().enumerate() {
            let grammar_config = obtain_grammar_config_from_string(test.input, false).unwrap();
            let augment_terminals = grammar_config.generate_augmented_terminals();
            assert_eq!(
                test.augment_terminals, augment_terminals,
                "Error at test #{i}"
            );
        }
    }
}
