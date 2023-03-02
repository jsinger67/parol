use super::parol_grammar_trait::{
    AlternationList, Declaration, GrammarDefinition, Parol, ParolGrammarTrait, Prolog, PrologList,
    PrologList0, ScannerDirectives, StartDeclaration, TokenLiteral,
};
use crate::grammar::{Decorate, ProductionAttribute, SymbolAttribute, TerminalKind};
use crate::ParolParserError;
use anyhow::anyhow;

use parol_macros::{bail, parol};

use parol_runtime::{lexer::Token, once_cell::sync::Lazy, Result};
use serde::{Deserialize, Serialize};

use std::collections::BTreeMap;
use std::fmt::{Debug, Display, Error, Formatter, Write};
use std::marker::PhantomData;

/// Used for implementation of trait `Default` for `&ParolGrammar`.
static DEFAULT_PAROL_GRAMMAR: Lazy<ParolGrammar<'static>> = Lazy::new(ParolGrammar::default);

const INITIAL_STATE: usize = 0;

/// A user defined type name
#[derive(Debug, Clone, Default, Hash, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
pub struct UserDefinedTypeName(Vec<String>);

impl UserDefinedTypeName {
    /// Creates a new [`UserDefinedTypeName`].
    pub fn new(names: Vec<String>) -> Self {
        Self(names)
    }

    /// Returns the length of this [`UserDefinedTypeName`].
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Checks if this [`UserDefinedTypeName`] is empty.
    /// ```
    /// use parol::parser::parol_grammar::UserDefinedTypeName;
    /// let user_type_name = UserDefinedTypeName::default();
    /// assert!(user_type_name.is_empty());
    /// let user_type_name = UserDefinedTypeName::new(vec!["bool".to_string()]);
    /// assert!(!user_type_name.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns a reference to the names of this [`UserDefinedTypeName`].
    pub fn names(&self) -> &[String] {
        &self.0
    }

    /// Returns module scoped name of this [`UserDefinedTypeName`].
    /// If you have a type `x::y::Z` this should return `x::y::Z`.
    /// ```
    /// use parol::parser::parol_grammar::UserDefinedTypeName;
    /// let user_type_name = UserDefinedTypeName::new(vec!["x".to_string(), "y".to_string(), "Z".to_string()]);
    /// assert_eq!("x::y::Z".to_string(), user_type_name.get_module_scoped_name());
    /// let user_type_name = UserDefinedTypeName::new(vec!["bool".to_string()]);
    /// assert_eq!("bool".to_string(), user_type_name.get_module_scoped_name());
    /// ```
    pub fn get_module_scoped_name(&self) -> String {
        self.0.to_vec().join("::")
    }
}

impl Display for UserDefinedTypeName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(f, "{}", self.0.to_vec().join("::"))
    }
}

/// This trait is used to automatically convert the generated type `UserTypeName` to our own
/// `UserDefinedTypeName`.
impl TryFrom<&super::parol_grammar_trait::UserTypeName<'_>> for UserDefinedTypeName {
    type Error = anyhow::Error;
    fn try_from(
        user_type_names: &super::parol_grammar_trait::UserTypeName<'_>,
    ) -> std::result::Result<Self, Self::Error> {
        Ok(Self(user_type_names.user_type_name_list.iter().fold(
            vec![user_type_names.identifier.identifier.text().to_string()],
            |mut acc, a| {
                acc.push(a.identifier.identifier.text().to_string());
                acc
            },
        )))
    }
}

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
    /// A terminal string with associated scanner states, a symbol attribute an an optional user
    /// type name
    Terminal(
        /// The scanned text
        String,
        /// interpretation of the terminals context regarding regular expression meta characters
        TerminalKind,
        /// The associated scanner states
        Vec<usize>,
        /// The symbol attribute associated with this terminal
        SymbolAttribute,
        /// A possibly provided user destination type
        Option<UserDefinedTypeName>,
    ),
    /// A non-terminal with a symbol attribute an an optional user type name
    NonTerminal(String, SymbolAttribute, Option<UserDefinedTypeName>),
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
        Self::NonTerminal(non_terminal, SymbolAttribute::default(), None)
    }

    pub(crate) fn inner_alts_mut(&mut self) -> Result<&mut Alternations> {
        match self {
            Factor::Group(alts) | Factor::Repeat(alts) | Factor::Optional(alts) => Ok(alts),
            _ => Err(parol_runtime::ParolError::UserError(anyhow!(
                "Ain't no inner alternations"
            ))),
        }
    }

    /// Generate parol's syntax
    pub fn to_par(&self) -> String {
        match self {
            Self::Group(g) => format!("({})", g.to_par()),
            Self::Repeat(r) => format!("{{{}}}", r.to_par()),
            Self::Optional(o) => format!("[{}]", o.to_par()),
            Self::Terminal(t, k, s, a, u) => {
                let mut d = String::new();
                a.decorate(&mut d, &format!("T({})", t))
                    .expect("Failed to decorate terminal!");
                if let Some(ref user_type) = u {
                    let _ = write!(d, " /* : {} */", user_type);
                }
                let delimiter = k.delimiter();
                format!(
                    "<{}>{}{}{}",
                    s.iter()
                        .map(|s| format!("{}", s))
                        .collect::<Vec<String>>()
                        .join(", "),
                    delimiter,
                    d,
                    delimiter
                )
            }
            Self::NonTerminal(n, a, u) => {
                let mut buf = String::new();
                a.decorate(&mut buf, n)
                    .expect("Failed to decorate non-terminal!");
                if let Some(ref user_type) = u {
                    let _ = write!(buf, " /* : {} */", user_type);
                }
                buf
            }
            Factor::Identifier(i) => format!("\"{}\"", i),
            Self::ScannerSwitch(n) => format!("%sc({})", n),
            Self::ScannerSwitchPush(n) => format!("%push({})", n),
            Self::ScannerSwitchPop => "%pop()".to_string(),
        }
    }

    fn is_used_scanner(&self, scanner_index: usize) -> bool {
        match self {
            Factor::Terminal(_, _, s, _, _) => s.contains(&scanner_index),
            Factor::Group(a) | Factor::Repeat(a) | Factor::Optional(a) => {
                a.is_used_scanner(scanner_index)
            }
            _ => false,
        }
    }
}

impl Display for Factor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match self {
            Self::Group(g) => write!(f, "G({})", g),
            Self::Repeat(r) => write!(f, "R{{{}}}", r),
            Self::Optional(o) => write!(f, "O[{}]", o),
            Self::Terminal(t, k, s, a, u) => {
                let mut d = String::new();
                let delimiter = k.delimiter();
                a.decorate(&mut d, &format!("T({}{}{})", delimiter, t, delimiter))?;
                if let Some(ref user_type) = u {
                    write!(d, " : {}", user_type)?;
                }
                write!(
                    f,
                    "<{}>{}",
                    s.iter()
                        .map(|s| format!("{}", s))
                        .collect::<Vec<String>>()
                        .join(", "),
                    d
                )
            }
            Self::NonTerminal(n, a, u) => {
                let mut s = String::new();
                a.decorate(&mut s, &format!("N({})", n))?;
                if let Some(ref user_type) = u {
                    write!(s, " : {}", user_type)?;
                }
                write!(f, "{}", s)
            }
            Self::Identifier(n) => write!(f, "Id({})", n),
            Self::ScannerSwitch(n) => write!(f, "S({})", n),
            Self::ScannerSwitchPush(n) => write!(f, "Push({})", n),
            Self::ScannerSwitchPop => write!(f, "Pop"),
        }
    }
}

///
/// An Alternation is a sequence of factors.
/// Valid operation on Alternation is "|".
///
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Alternation(pub Vec<Factor>, pub ProductionAttribute);

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

    fn is_used_scanner(&self, scanner_index: usize) -> bool {
        self.0.iter().any(|f| f.is_used_scanner(scanner_index))
    }
}

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

    fn is_used_scanner(&self, scanner_index: usize) -> bool {
        self.0.iter().any(|a| a.is_used_scanner(scanner_index))
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty() || (self.0.len() == 1 && self.0[0].0.is_empty())
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
            line_comments: Vec::default(),
            block_comments: Vec::default(),
            auto_newline_off: false,
            auto_ws_off: false,
        }
    }
}

/// This trait is used to automatically convert the generated type `ScannerState` to our own
/// `ScannerConfig`.
impl TryFrom<&super::parol_grammar_trait::ScannerState<'_>> for ScannerConfig {
    type Error = anyhow::Error;
    fn try_from(
        scanner_state: &super::parol_grammar_trait::ScannerState<'_>,
    ) -> std::result::Result<Self, Self::Error> {
        let mut me = Self {
            name: scanner_state.identifier.identifier.text().to_string(),
            ..Default::default()
        };
        for scanner_directive in &scanner_state.scanner_state_list {
            match &*scanner_directive.scanner_directives {
                ScannerDirectives::PercentLineUnderscoreCommentTokenLiteral(line_comment) => {
                    me.line_comments.push(ParolGrammar::expanded_token_literal(
                        &line_comment.token_literal,
                    ))
                }
                ScannerDirectives::PercentBlockUnderscoreCommentTokenLiteralTokenLiteral(
                    block_comment,
                ) => me.block_comments.push((
                    ParolGrammar::expanded_token_literal(&block_comment.token_literal),
                    ParolGrammar::expanded_token_literal(&block_comment.token_literal0),
                )),
                ScannerDirectives::PercentAutoUnderscoreNewlineUnderscoreOff(_) => {
                    me.auto_newline_off = true
                }
                ScannerDirectives::PercentAutoUnderscoreWsUnderscoreOff(_) => me.auto_ws_off = true,
            }
        }
        Ok(me)
    }
}

#[derive(Debug, Clone)]
enum ASTControlKind {
    Attr(SymbolAttribute),
    UserTyped(UserDefinedTypeName),
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
    /// User type definitions (aliases)
    pub user_type_definitions: BTreeMap<String, UserDefinedTypeName>,
    /// Contains information about token aliases:
    /// (LHS identifier, Token literal, expanded text)
    token_aliases: Vec<(Token<'static>, String)>,
    // Just to hold the lifetime generated by parol
    phantom: PhantomData<&'t str>,
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
        self.process_prolog(&parol.prolog)?;
        self.process_grammar_definition(&parol.grammar_definition)?;
        self.check()
    }

    fn process_prolog(&mut self, prolog: &Prolog) -> Result<()> {
        self.process_start_declaration(&prolog.start_declaration)?;
        self.process_declarations(&prolog.prolog_list)?;
        self.process_scanner_states(&prolog.prolog_list0);
        Ok(())
    }

    fn process_declarations(&mut self, declarations: &[PrologList]) -> Result<()> {
        for d in declarations {
            self.process_declaration(d)?;
        }
        Ok(())
    }

    fn trim_quotes(string: &str) -> String {
        let delimiters: &[_] = &['"', '\'', '/'];
        string
            .strip_prefix(delimiters)
            .unwrap()
            .strip_suffix(delimiters)
            .unwrap()
            .to_string()
    }

    fn process_declaration(&mut self, declaration: &PrologList) -> Result<()> {
        match &*declaration.declaration {
            Declaration::PercentTitleString(title_decl) => {
                self.title = Some(Self::trim_quotes(title_decl.string.string.text()))
            }
            Declaration::PercentCommentString(comment_decl) => {
                self.comment = Some(Self::trim_quotes(comment_decl.string.string.text()))
            }
            Declaration::PercentUserUnderscoreTypeIdentifierEquUserTypeName(user_type_def) => {
                self.process_user_type_definition(user_type_def)
            }
            Declaration::ScannerDirectives(scanner_decl) => {
                self.process_scanner_directive(&scanner_decl.scanner_directives)?
            }
        }
        Ok(())
    }

    fn process_scanner_directive(&mut self, scanner_directives: &ScannerDirectives) -> Result<()> {
        match scanner_directives {
            ScannerDirectives::PercentLineUnderscoreCommentTokenLiteral(line_comment) => self
                .scanner_configurations[INITIAL_STATE]
                .line_comments
                .push(Self::expanded_token_literal(&line_comment.token_literal)),
            ScannerDirectives::PercentBlockUnderscoreCommentTokenLiteralTokenLiteral(
                block_comment,
            ) => self.scanner_configurations[INITIAL_STATE]
                .block_comments
                .push((
                    Self::expanded_token_literal(&block_comment.token_literal),
                    Self::expanded_token_literal(&block_comment.token_literal0),
                )),
            ScannerDirectives::PercentAutoUnderscoreNewlineUnderscoreOff(_) => {
                self.scanner_configurations[INITIAL_STATE].auto_newline_off = true
            }
            ScannerDirectives::PercentAutoUnderscoreWsUnderscoreOff(_) => {
                self.scanner_configurations[INITIAL_STATE].auto_ws_off = true
            }
        }
        Ok(())
    }

    fn process_scanner_states(&mut self, scanner_states: &[PrologList0]) {
        for s in scanner_states {
            self.scanner_configurations.push(s.scanner_state.clone());
        }
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
        self.start_symbol = start_declaration.identifier.identifier.text().to_string();
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
        let lhs = prod.identifier.identifier.text().to_string();
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
            result.insert(self.process_factor(&a.factor)?)
        }
        Ok(result)
    }

    #[named]
    fn process_factor(&mut self, factor: &super::parol_grammar_trait::Factor) -> Result<Factor> {
        let context = function_name!();
        match factor {
            super::parol_grammar_trait::Factor::Group(group) => {
                let alternations = Self::to_alternation_vec(&group.group.alternations);
                let factors = self.process_alternations(&alternations)?;
                if factors.is_empty() {
                    Err(parol!(ParolParserError::EmptyGroup {
                        context: context.to_string(),
                        input: group.group.l_paren.location.file_name.to_path_buf(),
                        start: group.group.l_paren.location.clone(),
                        end: group.group.r_paren.location.clone(),
                    }))
                } else {
                    Ok(Factor::Group(factors))
                }
            }
            super::parol_grammar_trait::Factor::Repeat(repeat) => {
                let alternations = Self::to_alternation_vec(&repeat.repeat.alternations);
                let factors = self.process_alternations(&alternations)?;
                if factors.is_empty() {
                    Err(parol!(ParolParserError::EmptyRepetition {
                        context: context.to_string(),
                        input: repeat.repeat.l_brace.location.file_name.to_path_buf(),
                        start: repeat.repeat.l_brace.location.clone(),
                        end: repeat.repeat.r_brace.location.clone(),
                    }))
                } else {
                    Ok(Factor::Repeat(factors))
                }
            }
            super::parol_grammar_trait::Factor::Optional(optional) => {
                let alternations = Self::to_alternation_vec(&optional.optional.alternations);
                let factors = self.process_alternations(&alternations)?;
                if factors.is_empty() {
                    Err(parol!(ParolParserError::EmptyOptional {
                        context: context.to_string(),
                        input: optional.optional.l_bracket.location.file_name.to_path_buf(),
                        start: optional.optional.l_bracket.location.clone(),
                        end: optional.optional.r_bracket.location.clone(),
                    }))
                } else {
                    Ok(Factor::Optional(factors))
                }
            }
            super::parol_grammar_trait::Factor::Symbol(symbol) => {
                self.process_symbol(&symbol.symbol)
            }
        }
    }

    fn process_ast_control(
        &mut self,
        ast_control: &super::parol_grammar_trait::ASTControl,
    ) -> ASTControlKind {
        match ast_control {
            super::parol_grammar_trait::ASTControl::CutOperator(_) => {
                ASTControlKind::Attr(SymbolAttribute::Clipped)
            }
            super::parol_grammar_trait::ASTControl::UserTypeDeclaration(t) => {
                let mut user_type_name = t.user_type_declaration.user_type_name.clone();
                if let Some(defined_type) = self
                    .user_type_definitions
                    .get(&user_type_name.get_module_scoped_name())
                {
                    user_type_name = defined_type.clone();
                }
                ASTControlKind::UserTyped(user_type_name)
            }
        }
    }

    fn measure_token_literal<'a>(literal: &'a TokenLiteral) -> (&'a str, TerminalKind) {
        match literal {
            super::parol_grammar_trait::TokenLiteral::String(s) => {
                (s.string.string.text(), TerminalKind::Legacy)
            }
            super::parol_grammar_trait::TokenLiteral::RawString(l) => {
                (l.raw_string.raw_string.text(), TerminalKind::Raw)
            }
            super::parol_grammar_trait::TokenLiteral::Regex(r) => {
                (r.regex.regex.text(), TerminalKind::Regex)
            }
        }
    }

    fn process_symbol(&mut self, symbol: &super::parol_grammar_trait::Symbol) -> Result<Factor> {
        match symbol {
            super::parol_grammar_trait::Symbol::NonTerminal(non_terminal) => {
                let mut attr = SymbolAttribute::None;
                let mut user_type_name = None;
                if let Some(ref non_terminal_opt) = &non_terminal.non_terminal.non_terminal_opt {
                    match self.process_ast_control(&non_terminal_opt.a_s_t_control) {
                        ASTControlKind::Attr(a) => attr = a,
                        ASTControlKind::UserTyped(u) => user_type_name = Some(u),
                    }
                }
                Ok(Factor::NonTerminal(
                    non_terminal
                        .non_terminal
                        .identifier
                        .identifier
                        .text()
                        .to_string(),
                    attr,
                    user_type_name,
                ))
            }
            super::parol_grammar_trait::Symbol::SimpleToken(simple_token) => {
                let mut attr = SymbolAttribute::None;
                let mut user_type_name = None;
                if let Some(ref terminal_opt) = &simple_token.simple_token.simple_token_opt {
                    match self.process_ast_control(&terminal_opt.a_s_t_control) {
                        ASTControlKind::Attr(a) => attr = a,
                        ASTControlKind::UserTyped(u) => user_type_name = Some(u),
                    }
                }
                let (content, kind) =
                    Self::measure_token_literal(&simple_token.simple_token.token_literal);
                Ok(Factor::Terminal(
                    Self::trim_quotes(content),
                    kind,
                    vec![0],
                    attr,
                    user_type_name,
                ))
            }
            super::parol_grammar_trait::Symbol::TokenWithStates(token_with_states) => {
                let mut scanner_states = self
                    .process_scanner_state_list(&token_with_states.token_with_states.state_list)?;
                scanner_states.sort_unstable();
                let mut attr = SymbolAttribute::None;
                let mut user_type_name = None;
                if let Some(ref terminal_opt) =
                    &token_with_states.token_with_states.token_with_states_opt
                {
                    match self.process_ast_control(&terminal_opt.a_s_t_control) {
                        ASTControlKind::Attr(a) => attr = a,
                        ASTControlKind::UserTyped(u) => user_type_name = Some(u),
                    }
                }
                let (content, kind) =
                    Self::measure_token_literal(&token_with_states.token_with_states.token_literal);
                Ok(Factor::Terminal(
                    Self::trim_quotes(content),
                    kind,
                    scanner_states,
                    attr,
                    user_type_name,
                ))
            }
            super::parol_grammar_trait::Symbol::ScannerSwitch(scanner_switch) => {
                self.process_scanner_switch(scanner_switch)
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
    fn resolve_scanner(&self, scanner_name: &Token<'_>) -> Result<usize> {
        let context = function_name!();
        self.scanner_configurations
            .iter()
            .position(|s| s.name == scanner_name.text())
            .ok_or_else(|| {
                (ParolParserError::UnknownScanner {
                    context: context.to_owned(),
                    name: scanner_name.text().to_string(),
                    input: scanner_name.location.file_name.to_path_buf(),
                    token: scanner_name.into(),
                })
                .into()
            })
    }

    fn process_scanner_switch(
        &self,
        scanner_switch: &super::parol_grammar_trait::SymbolScannerSwitch,
    ) -> Result<Factor> {
        match &*scanner_switch.scanner_switch {
            super::parol_grammar_trait::ScannerSwitch::PercentScLParenScannerSwitchOptRParen(
                sw,
            ) => match &sw.scanner_switch_opt {
                Some(st) => Ok(Factor::ScannerSwitch(
                    self.resolve_scanner(&st.identifier.identifier)?,
                )),
                None => Ok(Factor::ScannerSwitch(INITIAL_STATE)),
            },
            super::parol_grammar_trait::ScannerSwitch::PercentPushLParenIdentifierRParen(sw) => Ok(
                Factor::ScannerSwitchPush(self.resolve_scanner(&sw.identifier.identifier)?),
            ),
            super::parol_grammar_trait::ScannerSwitch::PercentPopLParenRParen(_) => {
                Ok(Factor::ScannerSwitchPop)
            }
        }
    }

    fn process_user_type_definition(
        &mut self,
        user_type_def: &super::parol_grammar_trait::DeclarationPercentUserUnderscoreTypeIdentifierEquUserTypeName,
    ) {
        self.user_type_definitions.insert(
            user_type_def.identifier.identifier.text().to_string(),
            user_type_def.user_type_name.clone(),
        );
    }

    fn handle_token_alias(
        &mut self,
        lhs_non_terminal: Token<'static>,
        expanded: String,
    ) -> Result<()> {
        if let Some(conflicting_alias) = self.token_aliases.iter().find(|(_, e)| {
            *e == expanded // Is true if conflict found
        }) {
            bail!(ParolParserError::ConflictingTokenAliases {
                first_alias: conflicting_alias.0.text().to_string(),
                second_alias: lhs_non_terminal.text().to_string(),
                expanded,
                input: lhs_non_terminal.location.file_name.to_path_buf(),
                first: (&conflicting_alias.0).into(),
                second: (&lhs_non_terminal).into(),
            })
        }
        self.token_aliases.push((lhs_non_terminal, expanded));
        Ok(())
    }

    fn expanded_token_literal(token_literal: &super::parol_grammar_trait::TokenLiteral) -> String {
        match token_literal {
            TokenLiteral::String(s) => TerminalKind::Legacy
                .expand(ParolGrammar::trim_quotes(s.string.string.text()).as_str()),
            TokenLiteral::RawString(l) => TerminalKind::Raw
                .expand(ParolGrammar::trim_quotes(l.raw_string.raw_string.text()).as_str()),
            TokenLiteral::Regex(r) => {
                TerminalKind::Regex.expand(ParolGrammar::trim_quotes(r.regex.regex.text()).as_str())
            }
        }
    }

    fn check(&self) -> Result<()> {
        let empty_scanners = self
            .scanner_configurations
            .iter()
            .enumerate()
            .skip(1) // Allow INITIAL to be empty to avoid annoyance
            .fold(Vec::new(), |mut acc, (i, e)| {
                if !self.is_used_scanner(i) {
                    acc.push(e.name.clone());
                }
                acc
            });
        if !empty_scanners.is_empty() {
            Err(ParolParserError::EmptyScanners { empty_scanners }.into())
        } else {
            Ok(())
        }
    }

    fn is_used_scanner(&self, scanner_index: usize) -> bool {
        self.productions
            .iter()
            .any(|p| p.rhs.0.iter().any(|a| a.is_used_scanner(scanner_index)))
    }
}

impl Display for ParolGrammar<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        writeln!(f, "title: {:?}", self.title)?;
        writeln!(f, "comment: {:?}", self.comment)?;
        writeln!(f, "start_symbol: {}", self.start_symbol)?;
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

impl Default for &ParolGrammar<'_> {
    fn default() -> Self {
        &DEFAULT_PAROL_GRAMMAR
    }
}

impl<'t> ParolGrammarTrait<'t> for ParolGrammar<'t> {
    /// Semantic action for non-terminal 'Production'
    fn production(&mut self, arg: &super::parol_grammar_trait::Production<'t>) -> Result<()> {
        use super::parol_grammar_trait::{
            Factor, Symbol, SymbolSimpleToken, SymbolTokenWithStates,
        };
        // Only one alternation
        if arg.alternations.alternations_list.is_empty() {
            // Only one factor in the single alternation
            if arg.alternations.alternation.alternation_list.len() == 1 {
                if let Factor::Symbol(symbol) =
                    &*arg.alternations.alternation.alternation_list[0].factor
                {
                    match &*symbol.symbol {
                        // Only applicable for SimpleToken ...
                        Symbol::SimpleToken(SymbolSimpleToken { simple_token }) => {
                            let expanded =
                                ParolGrammar::expanded_token_literal(&simple_token.token_literal);
                            self.handle_token_alias(
                                arg.identifier.identifier.to_owned(),
                                expanded,
                            )?;
                        }
                        // .. and TokenWithStates!
                        Symbol::TokenWithStates(SymbolTokenWithStates { token_with_states }) => {
                            let expanded = ParolGrammar::expanded_token_literal(
                                &token_with_states.token_literal,
                            );
                            self.handle_token_alias(
                                arg.identifier.identifier.to_owned(),
                                expanded,
                            )?;
                        }
                        _ => (),
                    }
                }
            }
        }
        Ok(())
    }

    /// Semantic action for non-terminal 'Parol'
    fn parol(&mut self, parol: &Parol<'t>) -> Result<()> {
        self.process_parol(parol)
    }
}
