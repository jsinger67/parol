use parol_runtime::TerminalIndex;
use parol_runtime::log::trace;
use parol_runtime::once_cell::sync::Lazy;

use super::{ScannerConfig, generate_terminal_name};
use crate::parser::parol_grammar::{GrammarType, LookaheadExpression};
use crate::parser::try_to_convert;
use crate::{Cfg, ParolGrammar, generate_name};
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
#[derive(Debug, Clone, Default)]
pub struct GrammarConfig {
    /// The non-terminals of the original context free grammar from the user's grammar description
    /// before any checks and transformation has been conducted.
    pub non_terminals: Vec<String>,
    ///
    /// The actual context free grammar.
    /// It should be checked and left-factored here.
    /// For this task use the
    /// [crate::generators::grammar_trans::check_and_transform_grammar]
    /// function to prepare the grammar.
    ///
    pub cfg: Cfg,

    /// The type of the grammar
    pub grammar_type: GrammarType,

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
    /// The first element of the tuple is the alias, the second the type name.
    ///
    pub user_type_defs: Vec<(String, String)>,

    ///
    /// Non-terminal type definitions, i.e., user defined types for non-terminals.
    /// The first element of the tuple is the non-terminal, the second the type name.
    ///
    pub nt_type_defs: Vec<(String, String)>,

    ///
    /// Terminal type definitions, i.e., a single optional user defined type for terminals
    ///
    pub t_type_def: Option<String>,

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
/// Also it resolves a user type for a non-terminal to the non-terminal's name.
/// For terminals it resolves a user type to the fixed string "%t_type".
/// The latter two are used to skip the user type on non-terminal and terminal occurrences because
/// they are globally defined and need not be repeated.
pub(crate) type FnUserTypeResolver = Box<dyn Fn(&str) -> Option<String>>;

impl GrammarConfig {
    /// Creates a new item
    pub fn new(cfg: Cfg, lookahead_size: usize) -> Self {
        Self {
            cfg,
            lookahead_size,
            ..Default::default()
        }
    }

    /// Sets the non-terminals
    pub fn with_non_terminals(mut self, non_terminals: Vec<String>) -> Self {
        self.non_terminals = non_terminals;
        self
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

    /// Sets the grammar type
    pub fn with_grammar_type(mut self, grammar_type: GrammarType) -> Self {
        trace!("GrammarConfig::with_grammar_type({:?})", grammar_type);
        self.grammar_type = grammar_type;
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

    /// Adds a nt type definition
    pub fn add_nt_type_def(mut self, alias: String, type_name: String) -> Self {
        self.nt_type_defs.push((alias, type_name));
        self
    }

    /// Sets the lookahead size
    pub fn update_lookahead_size(&mut self, k: usize) {
        self.lookahead_size = k;
    }

    /// Updates the cfg member after the grammar has been checked and transformed
    pub fn update_cfg(&mut self, cfg: Cfg) {
        self.cfg = cfg;
    }

    ///
    /// Generates the augmented tokens vector in the format needed by the lexer
    /// generator.
    ///
    pub fn generate_augmented_terminals(&self) -> Vec<(String, Option<LookaheadExpression>)> {
        let terminals = vec![
            ("UNMATCHABLE_TOKEN".to_owned(), None),
            ("UNMATCHABLE_TOKEN".to_owned(), None),
            ("UNMATCHABLE_TOKEN".to_owned(), None),
            ("UNMATCHABLE_TOKEN".to_owned(), None),
            ("UNMATCHABLE_TOKEN".to_owned(), None),
        ];
        let mut terminals = self.cfg.get_ordered_terminals().into_iter().fold(
            terminals,
            |mut acc, (t, k, l, _)| {
                acc.push((k.expand(t), l.clone()));
                acc
            },
        );
        terminals.push(("ERROR_TOKEN".to_owned(), None));
        terminals
    }

    /// Generates a terminal names for the terminal match arms
    pub fn generate_terminal_names(&self) -> Vec<(usize, String)> {
        let names = vec![
            "NewLine".to_owned(),
            "Whitespace".to_owned(),
            "LineComment".to_owned(),
            "BlockComment".to_owned(),
        ];
        self.cfg
            .get_ordered_terminals()
            .into_iter()
            .enumerate()
            .map(|(i, (t, _, _, _))| (i + 5, t))
            .fold(names, |mut acc, (i, t)| {
                let name = generate_name(
                    acc.iter(),
                    generate_terminal_name(t, Some(i as TerminalIndex), &self.cfg),
                );
                acc.push(name);
                acc
            })
            .into_iter()
            .enumerate()
            .map(|(i, n)| (i + 1, n))
            .collect()
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
        let mut user_type_map =
            self.user_type_defs
                .iter()
                .fold(HashMap::new(), |mut acc, (a, u)| {
                    acc.insert(u.to_string(), a.to_string());
                    acc
                });
        user_type_map = self
            .nt_type_defs
            .iter()
            .fold(user_type_map, |mut acc, (nt, _u)| {
                // This allows to skip the user type on non-terminal occurrences
                acc.insert(nt.to_string(), "%nt_type".to_string());
                acc
            });
        if let Some(t) = &self.t_type_def {
            // This allows to skip the user type on terminal occurrences
            user_type_map.insert(t.to_string(), "%t_type".to_string());
        }
        Box::new(move |u: &str| user_type_map.get(u).cloned())
    }

    /// Generates a dummy user_type_resolver function that can be used in Pr::format
    pub fn dummy_user_type_resolver() -> FnUserTypeResolver {
        Box::new(|_u: &str| None)
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

    use crate::generators::GrammarConfig;
    use crate::parser::parol_grammar::LookaheadExpression;
    use crate::{
        Cfg, Pr, Symbol, SymbolAttribute, Terminal, TerminalKind, obtain_grammar_config_from_string,
    };

    macro_rules! terminal {
        ($term:literal) => {
            Symbol::T(Terminal::Trm(
                $term.to_string(),
                TerminalKind::Legacy,
                vec![0],
                SymbolAttribute::None,
                None,
                None,
                None,
            ))
        };
    }

    macro_rules! augmented_terminals {
        ($($term:literal),+) => {
            &[
                ("UNMATCHABLE_TOKEN", None),
                ("UNMATCHABLE_TOKEN", None),
                ("UNMATCHABLE_TOKEN", None),
                ("UNMATCHABLE_TOKEN", None),
                ("UNMATCHABLE_TOKEN", None),
                $(($term, None),)*
                ("ERROR_TOKEN", None),
            ]
        };
    }

    #[derive(Debug)]
    struct TestData {
        input: &'static str,
        augment_terminals: &'static [(&'static str, Option<LookaheadExpression>)],
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
            input: r"%start A %% A: B /r/; B: C /\d/; C: A /d/;",
            augment_terminals: augmented_terminals!["r", r"\d", "d"],
        },
        TestData {
            input: r"%start A %% A: B /r/; B: C /\s/; C: A /d/;",
            augment_terminals: augmented_terminals!["r", r"\s", "d"],
        },
        TestData {
            input: r"%start A %% A: B /r/; B: C '\s'; C: A /d/;",
            augment_terminals: augmented_terminals!["r", r"\\s", "d"],
        },
    ];

    #[test]
    fn check_generate_augmented_terminals_generic() {
        for (i, test) in TESTS.iter().enumerate() {
            let grammar_config = obtain_grammar_config_from_string(test.input, false)
                .unwrap_or_else(|e| panic!("Error parsing text #{i}: '{}'\n{e:?}", test.input));
            let original_augment_terminals = grammar_config.generate_augmented_terminals();
            let augment_terminals = original_augment_terminals
                .iter()
                .map(|(t, l)| (t.as_str(), l.clone()))
                .collect::<Vec<(&str, Option<LookaheadExpression>)>>();

            assert_eq!(
                test.augment_terminals, augment_terminals,
                "Error at test #{i}"
            );
        }
    }

    #[test]
    fn check_generate_terminal_names_conflict() {
        let g = Cfg::with_start_symbol("S")
            .add_pr(Pr::new("NewLine", vec![terminal!("n")]))
            .add_pr(Pr::new("A", vec![terminal!("a")]))
            .add_pr(Pr::new("B", vec![terminal!("b")]));

        let grammar_config = GrammarConfig::new(g, 1);

        let terminal_names = grammar_config.generate_terminal_names();

        assert_eq!(
            terminal_names,
            vec![
                (1, "NewLine".to_owned()),
                (2, "Whitespace".to_owned()),
                (3, "LineComment".to_owned()),
                (4, "BlockComment".to_owned()),
                (5, "NewLine0".to_owned()),
                (6, "A".to_owned()),
                (7, "B".to_owned()),
            ]
        );
    }
}
