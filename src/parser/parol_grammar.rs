use super::parol_grammar_trait::{
    AlternationList, Declaration, GrammarDefinition, Parol, ParolGrammarTrait, Prolog, PrologList,
    PrologList0, ScannerDirectives, ScannerState, StartDeclaration,
};
use super::ParolParserError;
use crate::grammar::ProductionAttribute;
use crate::grammar::{Decorate, SymbolAttribute};

use miette::{bail, miette, Result};
use parol_runtime::errors::FileSource;
use parol_runtime::lexer::Token;

use std::fmt::{Debug, Display, Error, Formatter};
use std::marker::PhantomData;
use std::path::PathBuf;

lazy_static! {
    /// Used for implementation of trait `Default` for `&ParolGrammar`.
    static ref DEFAULT_PAROL_GRAMMAR: ParolGrammar<'static> =
        ParolGrammar::default();
}

const INITIAL_STATE: usize = 0;

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
        self.0.push(fac)
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
        self.0.push(alt)
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
        ParolGrammar::<'_> {
            scanner_configurations: vec![ScannerConfig::default()],
            ..Default::default()
        }
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

    fn trim_quotes(string: &super::parol_grammar_trait::String) -> String {
        string.string.symbol.trim_matches('"').to_string()
    }

    fn process_declaration(&mut self, declaration: &PrologList) -> Result<()> {
        match &*declaration.declaration {
            Declaration::Declaration0(title_decl) => {
                self.title = Some(Self::trim_quotes(&title_decl.string))
            }
            Declaration::Declaration1(comment_decl) => {
                self.comment = Some(Self::trim_quotes(&comment_decl.string))
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
                .push(Self::trim_quotes(&line_comment.string)),
            ScannerDirectives::ScannerDirectives1(block_comment) => {
                self.current_scanner.block_comments.push((
                    Self::trim_quotes(&block_comment.string),
                    Self::trim_quotes(&block_comment.string0),
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

    fn to_alternation_vec<'t>(
        alts: &'t super::parol_grammar_trait::Alternations<'t>,
    ) -> Vec<&'t super::parol_grammar_trait::Alternation<'t>> {
        alts.alternations_list
            .iter()
            .fold(vec![&*alts.alternation], |mut acc, a| {
                acc.push(&*a.alternation);
                acc
            })
    }

    fn process_production(&mut self, prod: &super::parol_grammar_trait::Production) -> Result<()> {
        let lhs = prod.identifier.identifier.symbol.to_string();
        let alternations = Self::to_alternation_vec(&prod.alternations);
        let rhs = self.process_alternations(&alternations)?;
        self.productions.push(Production { lhs, rhs });
        Ok(())
    }

    fn process_alternations(
        &mut self,
        alternations: &[&super::parol_grammar_trait::Alternation],
    ) -> Result<Alternations> {
        let mut result = Alternations::new();
        for a in alternations {
            result.insert(self.process_alternation(&a.alternation_list)?);
        }
        Ok(result)
    }

    fn process_alternation(&mut self, alternation_list: &[AlternationList]) -> Result<Alternation> {
        let mut result = Alternation::new();
        for a in alternation_list {
            result.insert(self.process_factor(&*a.factor)?)
        }
        Ok(result)
    }

    fn process_factor(&mut self, factor: &super::parol_grammar_trait::Factor) -> Result<Factor> {
        match factor {
            super::parol_grammar_trait::Factor::Factor0(group) => {
                let alternations = Self::to_alternation_vec(&group.group.alternations);
                Ok(Factor::Group(self.process_alternations(&alternations)?))
            }
            super::parol_grammar_trait::Factor::Factor1(repeat) => {
                let alternations = Self::to_alternation_vec(&repeat.repeat.alternations);
                Ok(Factor::Repeat(self.process_alternations(&alternations)?))
            }
            super::parol_grammar_trait::Factor::Factor2(optional) => {
                let alternations = Self::to_alternation_vec(&optional.optional.alternations);
                Ok(Factor::Optional(self.process_alternations(&alternations)?))
            }
            super::parol_grammar_trait::Factor::Factor3(symbol) => {
                self.process_symbol(&*symbol.symbol)
            }
        }
    }

    fn process_symbol(&mut self, symbol: &super::parol_grammar_trait::Symbol) -> Result<Factor> {
        match symbol {
            super::parol_grammar_trait::Symbol::Symbol0(non_terminal) => Ok(Factor::NonTerminal(
                non_terminal
                    .non_terminal
                    .identifier
                    .identifier
                    .symbol
                    .to_string(),
                SymbolAttribute::None,
            )),
            super::parol_grammar_trait::Symbol::Symbol1(simple_token) => Ok(Factor::Terminal(
                Self::trim_quotes(&simple_token.simple_token.string),
                vec![0],
            )),
            super::parol_grammar_trait::Symbol::Symbol2(token_with_states) => {
                let mut scanner_states = self
                    .process_scanner_state_list(&*token_with_states.token_with_states.state_list)?;
                scanner_states.sort_unstable();
                Ok(Factor::Terminal(
                    Self::trim_quotes(&token_with_states.token_with_states.string),
                    scanner_states,
                ))
            }
            super::parol_grammar_trait::Symbol::Symbol3(scanner_switch) => {
                self.process_scanner_switch(&*scanner_switch)
            }
        }
    }

    fn process_scanner_state_list(
        &mut self,
        state_list: &super::parol_grammar_trait::StateList,
    ) -> Result<Vec<usize>> {
        let mut result = vec![self.resolve_scanner(&state_list.identifier.identifier)?];
        for s in &state_list.state_list_list {
            result.push(self.resolve_scanner(&s.identifier.identifier)?);
        }
        Ok(result)
    }

    #[named]
    fn resolve_scanner<'t>(&self, scanner_name: &Token<'t>) -> Result<usize> {
        let context = function_name!();
        self.scanner_configurations
            .iter()
            .position(|s| s.name == scanner_name.symbol)
            .ok_or(miette!(ParolParserError::UnknownScanner {
                context: context.to_owned(),
                name: scanner_name.symbol.to_string(),
                input: FileSource::try_new(self.file_name.clone())?.into(),
                token: scanner_name.into()
            }))
    }

    fn process_scanner_switch(
        &self,
        scanner_switch: &super::parol_grammar_trait::Symbol3,
    ) -> Result<Factor> {
        match &*scanner_switch.scanner_switch {
            super::parol_grammar_trait::ScannerSwitch::ScannerSwitch0(sw) => {
                match &sw.scanner_switch_opt {
                    Some(st) => Ok(Factor::ScannerSwitch(
                        self.resolve_scanner(&st.identifier.identifier)?,
                    )),
                    None => Ok(Factor::ScannerSwitch(INITIAL_STATE)),
                }
            }
            super::parol_grammar_trait::ScannerSwitch::ScannerSwitch1(sw) => Ok(
                Factor::ScannerSwitchPush(self.resolve_scanner(&sw.identifier.identifier)?),
            ),
            super::parol_grammar_trait::ScannerSwitch::ScannerSwitch2(_) => {
                Ok(Factor::ScannerSwitchPop)
            }
        }
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
