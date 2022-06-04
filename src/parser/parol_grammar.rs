use super::parol_grammar_trait::{
    Declaration, GrammarDefinition, Parol, ParolGrammarTrait, Prolog, PrologList, PrologList0,
    ScannerDirectives, ScannerState, StartDeclaration,
};
use super::ParolParserError;
use crate::grammar::ProductionAttribute;
use crate::grammar::{Decorate, SymbolAttribute};
use id_tree::Tree;
use log::trace;
use miette::{bail, miette, IntoDiagnostic, Result};
use parol_runtime::errors::FileSource;
use parol_runtime::parser::{ParseTreeStackEntry, ParseTreeType};
use std::fmt::{Debug, Display, Error, Formatter};
use std::marker::PhantomData;
use std::path::PathBuf;

lazy_static! {
    /// Used for implementation of trait `Default` for `&ParolGrammar`.
    static ref DEFAULT_PAROL_GRAMMAR: ParolGrammar<'static> =
        ParolGrammar::default();
}

// To rebuild the parser sources from scratch use the command build_parsers.ps1

// Test run:
// parol -f .\src\parser\parol-grammar.par -v

///
/// [Factor] is part of the structure of the grammar representation
///
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Factor {
    /// A grouping
    Group(Alternations),
    /// A Repetition
    Repeat(Alternations),
    /// An Optional
    Optional(Alternations),
    /// A terminal string with associated scanner states
    Terminal(String, Vec<usize>),
    /// A non-terminal
    NonTerminal(String, SymbolAttribute),
    /// An identifier, scanner state name
    Identifier(String),
    /// A scanner switch instruction
    ScannerSwitch(usize),
    /// A scanner switch & push instruction
    ScannerSwitchPush(usize),
    /// A scanner switch + pop instruction
    ScannerSwitchPop,
}

impl Factor {
    pub(crate) fn default_non_terminal(non_terminal: String) -> Self {
        Self::NonTerminal(non_terminal, SymbolAttribute::default())
    }

    pub(crate) fn inner_alts_mut(&mut self) -> Result<&mut Alternations> {
        match self {
            Factor::Group(alts) | Factor::Repeat(alts) | Factor::Optional(alts) => Ok(alts),
            _ => bail!("Ain't no inner alternations"),
        }
    }
}

impl Display for Factor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match self {
            Self::Group(g) => write!(f, "G({})", g),
            Self::Repeat(r) => write!(f, "R{{{}}}", r),
            Self::Optional(o) => write!(f, "O[{}]", o),
            Self::Terminal(t, s) => write!(
                f,
                "<{}>T({})",
                s.iter()
                    .map(|s| format!("{}", s))
                    .collect::<Vec<String>>()
                    .join(", "),
                t
            ),
            Self::NonTerminal(n, a) => {
                let mut s = String::new();
                a.decorate(&mut s, &format!("N({})", n))?;
                write!(f, "{}", s)
            }
            Self::Identifier(n) => write!(f, "Id({})", n),
            Self::ScannerSwitch(n) => write!(f, "S({})", n),
            Self::ScannerSwitchPush(n) => write!(f, "Push({})", n),
            Self::ScannerSwitchPop => write!(f, "Pop"),
        }
    }
}

impl Factor {
    /// Generate parol's syntax
    pub fn to_par(&self) -> String {
        match self {
            Self::Group(g) => format!("({})", g.to_par()),
            Self::Repeat(r) => format!("{{{}}}", r.to_par()),
            Self::Optional(o) => format!("[{}]", o.to_par()),
            Self::Terminal(t, s) => format!(
                "<{}>\"{}\"",
                s.iter()
                    .map(|s| format!("{}", s))
                    .collect::<Vec<String>>()
                    .join(", "),
                t
            ),
            Self::NonTerminal(n, a) => {
                let mut buf = String::new();
                a.decorate(&mut buf, n)
                    .expect("Failed to decorate non-terminal!");
                buf
            }
            Factor::Identifier(i) => format!("\"{}\"", i),
            Self::ScannerSwitch(n) => format!("%sc({})", n),
            Self::ScannerSwitchPush(n) => format!("%push({})", n),
            Self::ScannerSwitchPop => "%pop()".to_string(),
        }
    }
}

///
/// An Alternation is a sequence of factors.
/// Valid operation on Alternation is "|".
///
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Alternation(pub Vec<Factor>, pub ProductionAttribute);

impl Display for Alternation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(
            f,
            "Alt({}",
            self.0
                .iter()
                .map(|f| format!("{}", f))
                .collect::<Vec<String>>()
                .join(", ")
        )?;
        if self.1 != ProductionAttribute::default() {
            write!(f, ": {})", self.1)
        } else {
            write!(f, ")")
        }
    }
}

impl Alternation {
    pub(crate) fn new() -> Self {
        Self(Vec::new(), ProductionAttribute::default())
    }

    pub(crate) fn with_factors(mut self, factors: Vec<Factor>) -> Self {
        self.0 = factors;
        self
    }

    pub(crate) fn with_attribute(mut self, attribute: ProductionAttribute) -> Self {
        self.1 = attribute;
        self
    }

    pub(crate) fn insert(&mut self, fac: Factor) {
        self.0.insert(0, fac)
    }

    pub(crate) fn push(&mut self, fac: Factor) {
        self.0.push(fac)
    }

    /// Generate parol's syntax
    pub fn to_par(&self) -> String {
        self.0
            .iter()
            .map(|f| f.to_par())
            .collect::<Vec<String>>()
            .join(" ")
    }
}

///
/// [Alternations] is part of the structure of the grammar representation
///
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Alternations(pub Vec<Alternation>);

impl Alternations {
    pub(crate) fn new() -> Self {
        Self(Vec::new())
    }

    pub(crate) fn insert(&mut self, alt: Alternation) {
        self.0.insert(0, alt)
    }

    /// Generate parol's syntax
    pub fn to_par(&self) -> String {
        self.0
            .iter()
            .map(|a| a.to_par())
            .collect::<Vec<String>>()
            .join(" | ")
    }
}

impl Display for Alternations {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(
            f,
            "Alts({})",
            self.0
                .iter()
                .map(|a| format!("{}", a))
                .collect::<Vec<String>>()
                .join(" | ")
        )
    }
}

///
/// [Production] is part of the structure of the grammar representation
///
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Production {
    /// Left-hand side non-terminal
    pub lhs: String,
    /// Right-hand side
    pub rhs: Alternations,
}

impl Production {
    pub(crate) fn new(lhs: String, rhs: Alternations) -> Self {
        Self { lhs, rhs }
    }
}

impl Display for Production {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(f, "{}: {};", self.lhs, self.rhs)
    }
}

///
/// [ParolGrammarItem] is part of the structure of the grammar representation
///
#[derive(Debug, Clone)]
pub enum ParolGrammarItem {
    /// A production
    Prod(Production),
    /// A collection of alternations
    Alts(Alternations),
    /// A collection of factors
    Alt(Alternation),
    /// A Factor
    Fac(Factor),
    /// A list of scanner states associated with a terminal symbol
    StateList(Vec<usize>),
}

impl Display for ParolGrammarItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match self {
            Self::Prod(p) => write!(f, "{}", p),
            Self::Alts(a) => write!(f, "{}", a),
            Self::Alt(a) => write!(f, "{}", a),
            Self::Fac(t) => write!(f, "{}", t),
            Self::StateList(s) => write!(
                f,
                "SL<{}>",
                s.iter()
                    .map(|e| format!("{}", e))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        }
    }
}

impl ParolGrammarItem {
    /// Generate parol's syntax
    pub fn to_par(&self) -> String {
        match self {
            Self::Prod(Production { lhs, rhs }) => format!("{}: {};", lhs, rhs.to_par()),
            Self::Alts(alts) => alts.to_par(),
            Self::Alt(alt) => alt.to_par(),
            Self::Fac(fac) => fac.to_par(),
            Self::StateList(sl) => sl
                .iter()
                .map(|e| format!("<{}>", e))
                .collect::<Vec<String>>()
                .join(", "),
        }
    }
}

///
/// [ScannerConfig] is part of the structure of the grammar representation
///
#[derive(Debug, Clone)]
pub struct ScannerConfig {
    /// Name of the Scanner State
    pub name: String,
    /// Optional line comments
    pub line_comments: Vec<String>,
    /// Optional block comments
    pub block_comments: Vec<(String, String)>,
    /// Defines whether to handle newlines automatically in scanner
    pub auto_newline_off: bool,
    /// Defines whether to handle whitespace automatically in scanner
    pub auto_ws_off: bool,
}

impl Display for ScannerConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(f, "name: {};", self.name)?;
        write!(f, "line_comments: {:?};", self.line_comments)?;
        write!(f, "block_comments: {:?};", self.block_comments)?;
        write!(f, "auto_newline_off: {};", self.auto_newline_off)?;
        write!(f, "auto_ws_off: {};", self.auto_ws_off)
    }
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            name: "INITIAL".to_owned(),
            line_comments: Vec::new(),
            block_comments: Vec::new(),
            auto_newline_off: false,
            auto_ws_off: false,
        }
    }
}

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
///
/// Data structure used to build up a parol::GrammarConfig during parsing.
///
#[derive(Debug, Clone, Default)]
pub struct ParolGrammar<'t> {
    /// The parsed productions
    pub productions: Vec<Production>,
    /// The optional title of the grammar
    pub title: Option<String>,
    /// The optional comment of the grammar
    pub comment: Option<String>,
    /// The mandatory start symbol of the grammar
    pub start_symbol: String,
    /// All parsed scanner configurations
    pub scanner_configurations: Vec<ScannerConfig>,
    current_scanner: ScannerConfig,
    file_name: PathBuf,
    // Just to hold the lifetime generated by parol
    phantom: PhantomData<&'t str>,
}

impl Default for &ParolGrammar<'_> {
    fn default() -> Self {
        &DEFAULT_PAROL_GRAMMAR
    }
}

impl ParolGrammar<'_> {
    ///
    /// Constructs a new item
    ///
    pub fn new() -> Self {
        ParolGrammar::default()
    }

    #[allow(dead_code)]
    // Use this function for debugging purposes:
    // $env:RUST_LOG="parol::parser=trace"
    //
    // trace!("{}", self.trace_item_stack(context));
    fn trace_item_stack(&self, context: &str) -> String {
        format!(
            "Item stack at {}:\n{}",
            context,
            self.productions
                .iter()
                .rev()
                .map(|s| format!("  {}", s))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }

    fn process_parol(&mut self, parol: &Parol<'_>) -> Result<()> {
        self.process_prolog(&*parol.prolog)?;
        self.process_grammar_definition(&*parol.grammar_definition)
    }

    fn process_prolog(&mut self, prolog: &Prolog) -> Result<()> {
        self.process_start_declaration(&*prolog.start_declaration)?;
        self.process_declarations(&*prolog.prolog_list)?;
        self.process_scanner_states(&*prolog.prolog_list0)
    }

    fn process_declarations(&mut self, declarations: &[PrologList]) -> Result<()> {
        for d in declarations {
            self.process_declaration(d)?;
        }
        self.scanner_configurations[0] = self.current_scanner.clone();
        self.current_scanner = ScannerConfig::default();
        Ok(())
    }

    fn process_declaration(&mut self, declaration: &PrologList) -> Result<()> {
        match &*declaration.declaration {
            Declaration::Declaration0(title_decl) => {
                self.title = Some(title_decl.string.string.symbol.to_string())
            }
            Declaration::Declaration1(comment_decl) => {
                self.comment = Some(comment_decl.string.string.symbol.to_string())
            }
            Declaration::Declaration2(scanner_decl) => {
                self.process_scanner_directive(&*scanner_decl.scanner_directives)?
            }
        }
        Ok(())
    }

    fn process_scanner_directive(&mut self, scanner_directives: &ScannerDirectives) -> Result<()> {
        match scanner_directives {
            ScannerDirectives::ScannerDirectives0(line_comment) => self
                .current_scanner
                .line_comments
                .push(line_comment.string.string.symbol.to_string()),
            ScannerDirectives::ScannerDirectives1(block_comment) => {
                self.current_scanner.block_comments.push((
                    block_comment.string.string.symbol.to_string(),
                    block_comment.string0.string.symbol.to_string(),
                ))
            }
            ScannerDirectives::ScannerDirectives2(_) => {
                self.current_scanner.auto_newline_off = true
            }
            ScannerDirectives::ScannerDirectives3(_) => self.current_scanner.auto_ws_off = true,
        }
        Ok(())
    }

    fn process_scanner_states(&mut self, scanner_states: &[PrologList0]) -> Result<()> {
        for s in scanner_states {
            self.process_scanner_state(&*s.scanner_state)?;
        }
        Ok(())
    }

    fn process_scanner_state(&mut self, scanner_state: &ScannerState) -> Result<()> {
        self.current_scanner.name = scanner_state.identifier.identifier.symbol.to_string();
        for directive in &scanner_state.scanner_state_list {
            self.process_scanner_directive(&*directive.scanner_directives)?;
        }
        self.scanner_configurations
            .push(self.current_scanner.clone());
        self.current_scanner = ScannerConfig::default();
        Ok(())
    }

    fn process_grammar_definition(&mut self, grammar_definition: &GrammarDefinition) -> Result<()> {
        let productions = grammar_definition.grammar_definition_list.iter().fold(
            vec![&*grammar_definition.production],
            |mut acc, p| {
                acc.push(&*p.production);
                acc
            },
        );
        self.process_productions(&productions)
    }

    fn process_start_declaration(&mut self, start_declaration: &StartDeclaration) -> Result<()> {
        self.start_symbol = start_declaration.identifier.identifier.symbol.to_string();
        Ok(())
    }

    fn process_productions(
        &mut self,
        productions: &[&super::parol_grammar_trait::Production],
    ) -> Result<()> {
        for prod in productions {
            self.process_production(prod)?;
        }
        Ok(())
    }

    fn process_production(&mut self, prod: &super::parol_grammar_trait::Production) -> Result<()> {
        let lhs = prod.identifier.identifier.symbol.to_string();
        let alternations = prod.alternations.alternations_list.iter().fold(
            vec![&*prod.alternations.alternation],
            |mut acc, a| {
                acc.push(&*a.alternation);
                acc
            },
        );
        let rhs = self.process_alternations(&alternations)?;
        self.productions.push(Production { lhs, rhs });
        Ok(())
    }

    fn process_alternations(
        &self,
        alternations: &[&super::parol_grammar_trait::Alternation],
    ) -> Result<Alternations> {
        todo!()
    }
}

impl Display for ParolGrammar<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        writeln!(f, "title: {:?}", self.title)?;
        writeln!(f, "comment: {:?}", self.comment)?;
        writeln!(f, "start_symbol: {}", self.start_symbol)?;
        writeln!(f, "current_scanner: {}", self.current_scanner.name)?;
        writeln!(
            f,
            "{}",
            self.scanner_configurations
                .iter()
                .map(|s| format!("{}", s))
                .collect::<Vec<String>>()
                .join("\n")
        )?;
        writeln!(
            f,
            "{}",
            self.productions
                .iter()
                .map(|e| format!("{}", e))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

impl<'t> ParolGrammarTrait<'t> for ParolGrammar<'t> {
    ///
    /// Information provided by parser
    ///
    fn init(&mut self, file_name: &std::path::Path) {
        self.file_name = file_name.into();
    }

    /// Semantic action for non-terminal 'Parol'
    fn parol(&mut self, parol: &Parol<'t>) -> Result<()> {
        self.process_parol(parol)
    }
}
