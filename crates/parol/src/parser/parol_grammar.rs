use super::parol_grammar_trait::{
    self, ASTControl, AlternationList, Declaration, GrammarDefinition, Parol, ParolGrammarTrait,
    Prolog, PrologList, PrologList0, ScannerDirectives, StartDeclaration, TokenLiteral,
};
use crate::ParolParserError;
use crate::grammar::{Decorate, ProductionAttribute, SymbolAttribute, TerminalKind};
use crate::parser::parol_grammar_trait::ScannerDirectivesPercentOnIdentifierListScannerStateDirectives;
use anyhow::anyhow;

use parol_macros::{bail, parol};

use parol_runtime::Location;
use parol_runtime::{Result, lexer::Token};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use std::collections::BTreeMap;
use std::fmt::{Debug, Display, Error, Formatter, Write};
use std::marker::PhantomData;

pub(crate) const INITIAL_STATE: usize = 0;

/// A user defined type name
#[derive(
    Debug, Clone, Default, Hash, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize, TS,
)]
#[ts(export)]
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
impl TryFrom<&parol_grammar_trait::UserTypeName<'_>> for UserDefinedTypeName {
    type Error = anyhow::Error;
    fn try_from(
        user_type_names: &parol_grammar_trait::UserTypeName<'_>,
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
/// [LookaheadExpression] is part of the [Factor::Terminal] enum variant
///
#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct LookaheadExpression {
    /// If the lookahead operation is positive or negative
    pub is_positive: bool,
    /// The scanned text
    pub pattern: String,
    /// interpretation of the terminals context regarding regular expression meta characters
    pub kind: TerminalKind,
}

impl LookaheadExpression {
    fn new(is_positive: bool, pattern: String, kind: TerminalKind) -> Self {
        Self {
            is_positive,
            pattern,
            kind,
        }
    }

    /// Generate parol's syntax
    pub fn to_par(&self) -> String {
        let delimiter = self.kind.delimiter();
        format!(
            "{} {}{}{}",
            if self.is_positive { "?=" } else { "?!" },
            delimiter,
            self.pattern,
            delimiter
        )
    }
}

impl Display for LookaheadExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(
            f,
            "{} {}{}{}",
            if self.is_positive { "?=" } else { "?!" },
            self.kind.delimiter(),
            self.pattern,
            self.kind.delimiter()
        )
    }
}

impl TryFrom<&parol_grammar_trait::LookAhead<'_>> for LookaheadExpression {
    type Error = anyhow::Error;
    fn try_from(
        lookahead: &parol_grammar_trait::LookAhead<'_>,
    ) -> std::result::Result<Self, Self::Error> {
        let (content, kind) = ParolGrammar::measure_token_literal(&lookahead.token_literal);

        Ok(Self::new(
            matches!(
                lookahead.look_ahead_group,
                parol_grammar_trait::LookAheadGroup::PositiveLookahead(_)
            ),
            ParolGrammar::trim_quotes(content),
            kind,
        ))
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
        /// An optional member name
        Option<String>,
        /// An optional lookahead
        Option<LookaheadExpression>,
    ),
    /// A non-terminal with a symbol attribute an an optional user type name and an optional
    /// member name
    NonTerminal(
        String,
        SymbolAttribute,
        Option<UserDefinedTypeName>,
        Option<String>,
    ),
    /// An identifier, scanner state name
    Identifier(String),
    /// A scanner switch instruction
    ScannerSwitch(usize, Location),
    /// A scanner switch & push instruction
    ScannerSwitchPush(usize, Location),
    /// A scanner switch + pop instruction
    ScannerSwitchPop(Location),
}

impl Factor {
    pub(crate) fn default_non_terminal(non_terminal: String) -> Self {
        Self::NonTerminal(non_terminal, SymbolAttribute::default(), None, None)
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
            Self::Terminal(t, k, s, a, u, m, l) => {
                let mut d = String::new();
                a.decorate(&mut d, &format!("T({})", m.as_ref().unwrap_or(t)))
                    .expect("Failed to decorate terminal!");
                if let Some(user_type) = u {
                    let _ = write!(d, " /* : {user_type} */");
                }
                let delimiter = k.delimiter();
                format!(
                    "<{}>{}{}{}{}",
                    s.iter()
                        .map(|s| format!("{s}"))
                        .collect::<Vec<String>>()
                        .join(", "),
                    delimiter,
                    d,
                    delimiter,
                    if let Some(lookahead) = l {
                        format!(" {}", lookahead.to_par())
                    } else {
                        "".to_string()
                    }
                )
            }
            Self::NonTerminal(n, a, u, m) => {
                let mut buf = String::new();
                a.decorate(&mut buf, &m.as_ref().unwrap_or(n))
                    .expect("Failed to decorate non-terminal!");
                if let Some(user_type) = u {
                    let _ = write!(buf, " /* : {user_type} */");
                }
                buf
            }
            Factor::Identifier(i) => format!("\"{i}\""),
            Self::ScannerSwitch(n, _) => format!("%sc({n})"),
            Self::ScannerSwitchPush(n, _) => format!("%push({n})"),
            Self::ScannerSwitchPop(_) => "%pop()".to_string(),
        }
    }

    fn is_used_scanner(&self, scanner_index: usize) -> bool {
        match self {
            Factor::Terminal(_, _, s, _, _, _, _) => s.contains(&scanner_index),
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
            Self::Group(g) => write!(f, "G({g})"),
            Self::Repeat(r) => write!(f, "R{{{r}}}"),
            Self::Optional(o) => write!(f, "O[{o}]"),
            Self::Terminal(t, k, s, a, u, m, l) => {
                let mut d = String::new();
                let delimiter = k.delimiter();
                a.decorate(&mut d, &format!("T({delimiter}{t}{delimiter})"))?;
                if let Some(member_name) = m {
                    write!(d, "@{member_name}")?;
                }
                if let Some(user_type) = u {
                    write!(d, " : {user_type}")?;
                }
                write!(
                    f,
                    "<{}>{}{}",
                    s.iter()
                        .map(|s| format!("{s}"))
                        .collect::<Vec<String>>()
                        .join(", "),
                    d,
                    if let Some(lookahead) = l {
                        format!(" {}", lookahead.to_par())
                    } else {
                        "".to_string()
                    }
                )
            }
            Self::NonTerminal(n, a, u, m) => {
                let mut s = String::new();
                a.decorate(&mut s, &format!("N({n})"))?;
                if let Some(member_name) = m {
                    write!(s, "@{member_name}")?;
                }
                if let Some(user_type) = u {
                    write!(s, " : {user_type}")?;
                }
                write!(f, "{s}")
            }
            Self::Identifier(n) => write!(f, "Id({n})"),
            Self::ScannerSwitch(n, _) => write!(f, "S({n})"),
            Self::ScannerSwitchPush(n, _) => write!(f, "Push({n})"),
            Self::ScannerSwitchPop(_) => write!(f, "Pop"),
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

    pub(crate) fn insert(&mut self, index: usize, fac: Factor) {
        self.0.insert(index, fac)
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

    fn is_terminal(&self) -> bool {
        self.0.len() == 1 && matches!(self.0[0], Factor::Terminal(..))
    }

    fn terminal(&self) -> Option<(&str, TerminalKind)> {
        if self.is_terminal() {
            match &self.0[0] {
                Factor::Terminal(t, k, _, _, _, _, _) => Some((t, *k)),
                _ => None,
            }
        } else {
            None
        }
    }

    fn has_switch(&self) -> Option<Location> {
        self.0.iter().find_map(|f| match f {
            Factor::ScannerSwitch(_, l)
            | Factor::ScannerSwitchPush(_, l)
            | Factor::ScannerSwitchPop(l) => Some(l.clone()),
            _ => None,
        })
    }
}

impl Display for Alternation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(
            f,
            "Alt({}",
            self.0
                .iter()
                .map(|f| format!("{f}"))
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
                .map(|a| format!("{a}"))
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

    fn has_switch(&self) -> Option<parol_runtime::Location> {
        self.rhs.0.iter().find_map(|a| a.has_switch())
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
                .map(|e| format!("<{e}>"))
                .collect::<Vec<String>>()
                .join(", "),
        }
    }
}

impl Display for ParolGrammarItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match self {
            Self::Prod(p) => write!(f, "{p}"),
            Self::Alts(a) => write!(f, "{a}"),
            Self::Alt(a) => write!(f, "{a}"),
            Self::Fac(t) => write!(f, "{t}"),
            Self::StateList(s) => write!(
                f,
                "SL<{}>",
                s.iter()
                    .map(|e| format!("{e}"))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        }
    }
}

/// ScannerStateSwitch is part of the structure of the grammar representation, more precisely
/// of the description of scanner state handling.
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub enum ScannerStateSwitch {
    /// A scanner state switch
    Switch(String, Location),
    /// A scanner state switch and push
    SwitchPush(String, Location),
    /// A scanner state switch and pop
    SwitchPop(Location),
}

impl TryFrom<&parol_grammar_trait::ScannerStateDirectives<'_>> for ScannerStateSwitch {
    type Error = anyhow::Error;
    fn try_from(
        scanner_state_directives: &parol_grammar_trait::ScannerStateDirectives<'_>,
    ) -> std::result::Result<Self, Self::Error> {
        match scanner_state_directives {
            parol_grammar_trait::ScannerStateDirectives::PercentEnterIdentifier(switch) => {
                Ok(ScannerStateSwitch::Switch(
                    switch.identifier.identifier.text().to_string(),
                    switch.identifier.identifier.location.clone(),
                ))
            }
            parol_grammar_trait::ScannerStateDirectives::PercentPushIdentifier(switch_push) => {
                Ok(ScannerStateSwitch::SwitchPush(
                    switch_push.identifier.identifier.text().to_string(),
                    switch_push.identifier.identifier.location.clone(),
                ))
            }
            parol_grammar_trait::ScannerStateDirectives::PercentPop(pop) => Ok(
                ScannerStateSwitch::SwitchPop(pop.percent_pop.location.clone()),
            ),
        }
    }
}

impl Display for ScannerStateSwitch {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match self {
            ScannerStateSwitch::Switch(s, _) => write!(f, "enter {s}"),
            ScannerStateSwitch::SwitchPush(s, _) => write!(f, "push {s}"),
            ScannerStateSwitch::SwitchPop(_) => write!(f, "pop"),
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
    /// Scanner state transitions
    /// Maps from (token, terminal kind) to scanner state, where the token is identified by its
    /// primary non-terminal name. The scanner state is identified by its name.
    pub transitions: BTreeMap<Token<'static>, ScannerStateSwitch>,
}

impl ScannerConfig {
    pub(crate) fn add_transitions(
        &mut self,
        transitions: &ScannerDirectivesPercentOnIdentifierListScannerStateDirectives<'_>,
    ) {
        // The identifier list contains the first identifier, which is inserted here
        self.transitions.insert(
            transitions.identifier_list.identifier.identifier.to_owned(),
            transitions.scanner_state_directives.clone(),
        );
        // The rest of the identifier list is processed here
        transitions
            .identifier_list
            .identifier_list_list
            .iter()
            .for_each(|i| {
                self.transitions.insert(
                    i.identifier.identifier.to_owned(),
                    transitions.scanner_state_directives.clone(),
                );
            });
    }
}

impl Display for ScannerConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(f, "name: {};", self.name)?;
        write!(f, "line_comments: {:?};", self.line_comments)?;
        write!(f, "block_comments: {:?};", self.block_comments)?;
        write!(f, "auto_newline_off: {};", self.auto_newline_off)?;
        write!(f, "auto_ws_off: {};", self.auto_ws_off)?;
        self.transitions
            .iter()
            .try_for_each(|(k, v)| write!(f, "on {k} {v};"))
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
            transitions: BTreeMap::default(),
        }
    }
}

/// This trait is used to automatically convert the generated type `ScannerState` to our own
/// `ScannerConfig`.
impl TryFrom<&parol_grammar_trait::ScannerState<'_>> for ScannerConfig {
    type Error = anyhow::Error;
    fn try_from(
        scanner_state: &parol_grammar_trait::ScannerState<'_>,
    ) -> std::result::Result<Self, Self::Error> {
        let mut me = Self {
            name: scanner_state.state_name.identifier.text().to_string(),
            ..Default::default()
        };
        for scanner_directive in &scanner_state.scanner_state_list {
            match &scanner_directive.scanner_directives {
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
                ScannerDirectives::PercentOnIdentifierListScannerStateDirectives(
                    scanner_directives_percent_on_identifier_list_scanner_state_directives,
                ) => {
                    me.add_transitions(
                        scanner_directives_percent_on_identifier_list_scanner_state_directives,
                    );
                }
            }
        }
        Ok(me)
    }
}

#[derive(Debug, Clone)]
enum ASTControlKind {
    Attr(SymbolAttribute),
    MemberName(String),
    UserTyped(UserDefinedTypeName),
    MemberNameUserTyped(String, UserDefinedTypeName),
}

/// The type of grammar supported by parol
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub enum GrammarType {
    /// LLK grammar, default
    #[default]
    LLK,
    /// LR(1) grammar, not yet supported
    LALR1,
}

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
///
/// Data structure used to build up a parol::GrammarConfig during parsing.
///
#[derive(Debug, Clone)]
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
    /// Non-terminal type definitions, i.e., user defined types for non-terminals
    pub nt_type_definitions: BTreeMap<String, UserDefinedTypeName>,
    /// Terminal type definitions, i.e., user defined types for terminals
    pub t_type_def: Option<UserDefinedTypeName>,
    /// The grammar type
    pub grammar_type: GrammarType,
    /// Contains information about token aliases:
    /// (LHS identifier as Token to keep location, expanded text)
    token_aliases: Vec<(Token<'static>, String)>,
    // Just to hold the lifetime generated by parol
    phantom: PhantomData<&'t str>,
}

impl ParolGrammar<'_> {
    ///
    /// Constructs a new item
    ///
    pub fn new() -> Self {
        Self::default()
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
        match &declaration.declaration {
            Declaration::PercentTitleString(title_decl) => {
                self.title = Some(Self::trim_quotes(title_decl.string.string.text()))
            }
            Declaration::PercentCommentString(comment_decl) => {
                self.comment = Some(Self::trim_quotes(comment_decl.string.string.text()))
            }
            Declaration::PercentUserUnderscoreTypeIdentifierEquUserTypeName(user_type_def) => {
                self.process_user_type_definition(user_type_def)
            }
            Declaration::PercentNtUnderscoreTypeNtNameEquNtType(nt_type) => {
                self.process_nt_type_definition(nt_type)
            }
            Declaration::PercentTUnderscoreTypeTType(t_type) => {
                // The last TType definition is used
                self.t_type_def = Some(t_type.t_type.clone());
            }
            Declaration::ScannerDirectives(scanner_decl) => {
                self.process_scanner_directive(&scanner_decl.scanner_directives)?
            }
            Declaration::PercentGrammarUnderscoreTypeRawString(grammar_type) => {
                self.process_grammar_type_declaration(&grammar_type.raw_string.raw_string)?
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
            ScannerDirectives::PercentOnIdentifierListScannerStateDirectives(transitions) => {
                self.scanner_configurations[INITIAL_STATE].add_transitions(transitions)
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
            vec![&grammar_definition.production],
            |mut acc, p| {
                acc.push(&p.production);
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
        productions: &[&parol_grammar_trait::Production],
    ) -> Result<()> {
        for prod in productions {
            self.process_production(prod)?;
        }
        for prod in productions {
            if self.is_single_production(prod.identifier.identifier.text()) {
                self.handle_token_alias(prod)?;
            }
        }
        Ok(())
    }

    fn to_alternation_vec<'t>(
        alts: &'t parol_grammar_trait::Alternations<'t>,
    ) -> Vec<&'t parol_grammar_trait::Alternation<'t>> {
        alts.alternations_list
            .iter()
            .fold(vec![&alts.alternation], |mut acc, a| {
                acc.push(&a.alternation);
                acc
            })
    }

    fn process_production(&mut self, prod: &parol_grammar_trait::Production) -> Result<()> {
        let lhs = prod.identifier.identifier.text().to_string();
        let alternations = Self::to_alternation_vec(&prod.alternations);
        let rhs = self.process_alternations(&alternations)?;
        self.productions.push(Production { lhs, rhs });
        Ok(())
    }

    fn process_alternations(
        &mut self,
        alternations: &[&parol_grammar_trait::Alternation],
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
            result.push(self.process_factor(&a.factor)?)
        }
        Ok(result)
    }

    #[named]
    fn process_factor(&mut self, factor: &parol_grammar_trait::Factor) -> Result<Factor> {
        let context = function_name!();
        match factor {
            parol_grammar_trait::Factor::Group(group) => {
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
            parol_grammar_trait::Factor::Repeat(repeat) => {
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
            parol_grammar_trait::Factor::Optional(optional) => {
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
            parol_grammar_trait::Factor::Symbol(symbol) => self.process_symbol(&symbol.symbol),
        }
    }

    fn process_ast_control(&mut self, ast_control: &ASTControl) -> ASTControlKind {
        match ast_control {
            ASTControl::CutOperator(_) => ASTControlKind::Attr(SymbolAttribute::Clipped),
            ASTControl::UserTypeDeclaration(t) => {
                let mut user_type_name = t.user_type_declaration.user_type_name.clone();
                if let Some(defined_type) = self
                    .user_type_definitions
                    .get(&user_type_name.get_module_scoped_name())
                {
                    user_type_name = defined_type.clone();
                }
                ASTControlKind::UserTyped(user_type_name)
            }
            ASTControl::MemberNameASTControlOpt(user_type_decl_opt) => {
                let member_name = user_type_decl_opt
                    .member_name
                    .identifier
                    .identifier
                    .text()
                    .to_string();
                match user_type_decl_opt.a_s_t_control_opt.as_ref() {
                    Some(user_type_decl) => {
                        let mut user_type_name =
                            user_type_decl.user_type_declaration.user_type_name.clone();
                        if let Some(defined_type) = self
                            .user_type_definitions
                            .get(&user_type_name.get_module_scoped_name())
                        {
                            user_type_name = defined_type.clone();
                        }
                        ASTControlKind::MemberNameUserTyped(member_name, user_type_name)
                    }
                    None => ASTControlKind::MemberName(member_name),
                }
            }
        }
    }

    fn measure_token_literal<'a>(literal: &'a TokenLiteral) -> (&'a str, TerminalKind) {
        match literal {
            parol_grammar_trait::TokenLiteral::String(s) => {
                (s.string.string.text(), TerminalKind::Legacy)
            }
            parol_grammar_trait::TokenLiteral::RawString(l) => {
                (l.raw_string.raw_string.text(), TerminalKind::Raw)
            }
            parol_grammar_trait::TokenLiteral::Regex(r) => {
                (r.regex.regex.text(), TerminalKind::Regex)
            }
        }
    }

    fn process_symbol(&mut self, symbol: &parol_grammar_trait::Symbol) -> Result<Factor> {
        match symbol {
            parol_grammar_trait::Symbol::NonTerminal(non_terminal) => {
                let mut attr = SymbolAttribute::None;
                let mut user_type_name = None;
                let mut member_name = None;
                if let Some(non_terminal_opt) = &non_terminal.non_terminal.non_terminal_opt {
                    self.extract_attributes_from_ast_control(
                        &non_terminal_opt.a_s_t_control,
                        &mut attr,
                        &mut user_type_name,
                        &mut member_name,
                    );
                }
                let non_terminal_name = non_terminal
                    .non_terminal
                    .identifier
                    .identifier
                    .text()
                    .to_string();
                if user_type_name.is_none() {
                    // If no local user type is defined, check if a global user type is defined
                    // for this non-terminal and use it.
                    if let Some(defined_type) = self.nt_type_definitions.get(&non_terminal_name) {
                        user_type_name = Some(defined_type.clone());
                    }
                }
                Ok(Factor::NonTerminal(
                    non_terminal_name,
                    attr,
                    user_type_name,
                    member_name,
                ))
            }
            parol_grammar_trait::Symbol::SimpleToken(simple_token) => {
                let mut attr = SymbolAttribute::None;
                let mut user_type_name = None;
                let mut member_name = None;
                if let Some(terminal_opt) = &simple_token.simple_token.simple_token_opt {
                    self.extract_attributes_from_ast_control(
                        &terminal_opt.a_s_t_control,
                        &mut attr,
                        &mut user_type_name,
                        &mut member_name,
                    );
                }
                if user_type_name.is_none() {
                    // If no local user type is defined, check if a global user type is defined
                    // for all terminals and use it.
                    if let Some(defined_type) = self.t_type_def.as_ref() {
                        user_type_name = Some(defined_type.clone());
                    }
                }
                let (content, kind) = Self::measure_token_literal(
                    &simple_token.simple_token.token_expression.token_literal,
                );
                let lookahead = simple_token
                    .simple_token
                    .token_expression
                    .token_expression_opt
                    .as_ref()
                    .map(|l| LookaheadExpression::try_from(&l.look_ahead))
                    .transpose()?;
                Ok(Factor::Terminal(
                    Self::trim_quotes(content),
                    kind,
                    vec![0],
                    attr,
                    user_type_name,
                    member_name,
                    lookahead,
                ))
            }
            parol_grammar_trait::Symbol::TokenWithStates(token_with_states) => {
                let mut scanner_states = self.process_scanner_state_list(
                    &token_with_states.token_with_states.identifier_list,
                )?;
                scanner_states.sort_unstable();
                let mut attr = SymbolAttribute::None;
                let mut user_type_name = None;
                let mut member_name = None;
                if let Some(terminal_opt) =
                    &token_with_states.token_with_states.token_with_states_opt
                {
                    self.extract_attributes_from_ast_control(
                        &terminal_opt.a_s_t_control,
                        &mut attr,
                        &mut user_type_name,
                        &mut member_name,
                    );
                }
                let (content, kind) = Self::measure_token_literal(
                    &token_with_states
                        .token_with_states
                        .token_expression
                        .token_literal,
                );
                let lookahead = token_with_states
                    .token_with_states
                    .token_expression
                    .token_expression_opt
                    .as_ref()
                    .map(|l| LookaheadExpression::try_from(&l.look_ahead))
                    .transpose()?;
                Ok(Factor::Terminal(
                    Self::trim_quotes(content),
                    kind,
                    scanner_states,
                    attr,
                    user_type_name,
                    member_name,
                    lookahead,
                ))
            }
        }
    }

    fn extract_attributes_from_ast_control(
        &mut self,
        ast_control: &parol_grammar_trait::ASTControl<'_>,
        attr: &mut SymbolAttribute,
        user_type_name: &mut Option<UserDefinedTypeName>,
        member_name: &mut Option<String>,
    ) {
        match self.process_ast_control(ast_control) {
            ASTControlKind::Attr(a) => *attr = a,
            ASTControlKind::UserTyped(u) => *user_type_name = Some(u),
            ASTControlKind::MemberName(n) => *member_name = Some(n),
            ASTControlKind::MemberNameUserTyped(member, user_defined_type_name) => {
                *member_name = Some(member);
                *user_type_name = Some(user_defined_type_name);
            }
        }
    }

    #[named]
    fn process_scanner_state_list(
        &mut self,
        state_list: &parol_grammar_trait::IdentifierList,
    ) -> Result<Vec<usize>> {
        let context = function_name!();

        let iter = std::iter::once(&state_list.identifier).chain(
            state_list
                .identifier_list_list
                .iter()
                .map(|s| &s.identifier),
        );

        iter.map(|id| {
            self.resolve_scanner(&id.identifier).ok_or_else(|| {
                ParolParserError::UnknownScanner {
                    context: context.to_owned(),
                    name: id.identifier.text().to_string(),
                    input: id.identifier.location.file_name.to_path_buf(),
                    token: id.identifier.location.clone(),
                }
                .into()
            })
        })
        .collect()
    }
    fn resolve_scanner<T: AsRef<str>>(&self, scanner_name: &T) -> Option<usize> {
        self.scanner_configurations
            .iter()
            .position(|s| s.name == scanner_name.as_ref())
    }

    fn process_user_type_definition(
        &mut self,
        user_type_def: &parol_grammar_trait::DeclarationPercentUserUnderscoreTypeIdentifierEquUserTypeName,
    ) {
        self.user_type_definitions.insert(
            user_type_def.identifier.identifier.text().to_string(),
            user_type_def.user_type_name.clone(),
        );
    }

    fn process_nt_type_definition(
        &mut self,
        user_type_def: &parol_grammar_trait::DeclarationPercentNtUnderscoreTypeNtNameEquNtType,
    ) {
        self.nt_type_definitions.insert(
            user_type_def.nt_name.identifier.text().to_string(),
            user_type_def.nt_type.clone(),
        );
    }

    fn handle_token_alias(&mut self, prod: &parol_grammar_trait::Production) -> Result<()> {
        if !prod.alternations.alternations_list.is_empty()
            || prod.alternations.alternation.alternation_list.len() != 1
        {
            return Ok(());
        }
        if let parol_grammar_trait::Factor::Symbol(sym) =
            &prod.alternations.alternation.alternation_list[0].factor
        {
            let expanded = match &sym.symbol {
                // Only applicable for SimpleToken ...
                parol_grammar_trait::Symbol::SimpleToken(
                    parol_grammar_trait::SymbolSimpleToken { simple_token },
                ) => ParolGrammar::expanded_token_expression(&simple_token.token_expression),
                // .. and TokenWithStates!
                parol_grammar_trait::Symbol::TokenWithStates(
                    parol_grammar_trait::SymbolTokenWithStates { token_with_states },
                ) => ParolGrammar::expanded_token_expression(&token_with_states.token_expression),
                _ => return Ok(()),
            };

            if let Some(conflicting_alias) = self.token_aliases.iter().find(|(_, e)| {
                *e == expanded // Is true if conflict found
            }) {
                bail!(ParolParserError::ConflictingTokenAliases {
                    first_alias: conflicting_alias.0.text().to_string(),
                    second_alias: prod.identifier.identifier.text().to_string(),
                    expanded,
                    input: prod.identifier.identifier.location.file_name.to_path_buf(),
                    first: (&conflicting_alias.0).into(),
                    second: (&prod.identifier.identifier).into(),
                })
            }
            self.token_aliases
                .push((prod.identifier.identifier.to_owned(), expanded));
        }
        Ok(())
    }

    fn expanded_token_expression(
        token_expression: &parol_grammar_trait::TokenExpression,
    ) -> String {
        let mut expanded = Self::expanded_token_literal(&token_expression.token_literal);
        if let Some(lookahead) = token_expression.token_expression_opt.as_ref() {
            expanded.push_str(&format!(
                " {}",
                LookaheadExpression::try_from(&lookahead.look_ahead).unwrap()
            ))
        }
        expanded
    }

    fn expanded_token_literal(token_literal: &parol_grammar_trait::TokenLiteral) -> String {
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
            return Err(ParolParserError::EmptyScanners { empty_scanners }.into());
        }

        self.scanner_configurations
            .iter()
            .enumerate()
            .try_for_each(|(i, s)| self.check_transitions(i, s))
    }

    fn check_transitions(&self, index: usize, s: &ScannerConfig) -> Result<()> {
        if !s.transitions.is_empty()
            && let Some(location) = self.parser_based_scanner_switching_used()
        {
            bail!(ParolParserError::MixedScannerSwitching {
                context: "check_transitions".to_string(),
                input: location.file_name.to_path_buf(),
                location,
            });
        }
        s.transitions.iter().try_for_each(|(k, v)| {
            if !self.is_primary_non_terminal(k) {
                bail!(ParolParserError::InvalidTokenInTransition {
                    context: "check_transitions".to_string(),
                    token: k.text().to_string(),
                    input: k.location.file_name.to_path_buf(),
                    location: k.location.clone(),
                });
            }
            if !self.is_terminal_in_scanner(k, index) {
                bail!(ParolParserError::TokenIsNotInScanner {
                    context: "check_transitions".to_string(),
                    scanner: s.name.clone(),
                    token: k.text().to_string(),
                    input: k.location.file_name.to_path_buf(),
                    location: k.location.clone(),
                });
            }
            match v {
                ScannerStateSwitch::Switch(s, location)
                | ScannerStateSwitch::SwitchPush(s, location) => {
                    if self.resolve_scanner(s).is_none() {
                        bail!(ParolParserError::UnknownScanner {
                            context: "check_transitions".to_string(),
                            name: s.clone(),
                            input: location.file_name.to_path_buf(),
                            token: location.clone(),
                        });
                    }
                }
                ScannerStateSwitch::SwitchPop(_) => (),
            }
            Ok(())
        })
    }

    fn is_used_scanner(&self, scanner_index: usize) -> bool {
        self.productions
            .iter()
            .any(|p| p.rhs.0.iter().any(|a| a.is_used_scanner(scanner_index)))
    }

    fn process_grammar_type_declaration(&mut self, grammar_type: &Token) -> Result<()> {
        let grammar_type_name = grammar_type.text().to_string().to_lowercase();
        if grammar_type_name == "'lalr(1)'" {
            self.grammar_type = GrammarType::LALR1;
        } else if grammar_type_name == "'ll(k)'" {
            self.grammar_type = GrammarType::LLK;
        } else {
            return Err(ParolParserError::UnsupportedGrammarType {
                grammar_type: grammar_type_name,
                input: grammar_type.location.file_name.to_path_buf(),
                token: grammar_type.location.clone(),
            }
            .into());
        }
        Ok(())
    }

    fn is_single_production(&self, lhs: &str) -> bool {
        self.productions.iter().filter(|p| p.lhs == lhs).count() == 1
    }

    fn is_primary_non_terminal(&self, k: &Token<'_>) -> bool {
        self.is_single_production(k.text())
            && self
                .productions
                .iter()
                .any(|p| p.lhs == k.text() && p.rhs.0.len() == 1 && p.rhs.0[0].is_terminal())
    }

    // Check if a terminal is used in the given scanner state
    fn is_terminal_in_scanner(&self, terminal: &Token<'_>, index: usize) -> bool {
        // Get the token text from the primary non-terminal
        let term = self
            .productions
            .iter()
            .find(|p| p.lhs == terminal.text() && p.rhs.0.len() == 1 && p.rhs.0[0].is_terminal())
            .and_then(|p| p.rhs.0[0].terminal());
        if let Some((tx, kind)) = term {
            // Find a terminal in the productions that matches the given terminal and is used in the
            // given scanner state
            self.productions.iter().any(|p| {
                p.rhs.0.iter().any(|a| {
                    if a.0.len() == 1 {
                        if let Factor::Terminal(t, k, _, _, _, _, _) = &a.0[0] {
                            *t == tx && k.behaves_like(kind) && a.is_used_scanner(index)
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                })
            })
        } else {
            false
        }
    }

    /// Check if the grammar uses parser-based scanner switching (i.e. scanner switching in the
    /// grammar productions) is used.
    /// We need to check if also scanner-based scanner switching (i.e. scanner switching in the
    /// scanner directives) is used, because the two cannot be mixed reasonably.
    fn parser_based_scanner_switching_used(&self) -> Option<parol_runtime::Location> {
        // Check if any of the productions uses scanner switching
        self.productions.iter().find_map(|p| p.has_switch())
    }
}

impl Display for ParolGrammar<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        writeln!(f, "title: {:?}", self.title)?;
        writeln!(f, "comment: {:?}", self.comment)?;
        writeln!(f, "start_symbol: {}", self.start_symbol)?;
        writeln!(f, "grammar_type: {:?}", self.grammar_type)?;
        writeln!(
            f,
            "{}",
            self.scanner_configurations
                .iter()
                .map(|s| format!("{s}"))
                .collect::<Vec<String>>()
                .join("\n")
        )?;
        writeln!(
            f,
            "{}",
            self.productions
                .iter()
                .map(|e| format!("{e}"))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

impl Default for ParolGrammar<'_> {
    fn default() -> Self {
        Self {
            productions: Vec::new(),
            title: None,
            comment: None,
            start_symbol: "".to_string(),
            scanner_configurations: vec![ScannerConfig::default()],
            user_type_definitions: BTreeMap::new(),
            nt_type_definitions: BTreeMap::new(),
            t_type_def: None,
            grammar_type: GrammarType::LLK,
            token_aliases: Vec::new(),
            phantom: PhantomData,
        }
    }
}

impl<'t> ParolGrammarTrait<'t> for ParolGrammar<'t> {
    /// Semantic action for non-terminal 'Parol'
    fn parol(&mut self, parol: &Parol<'t>) -> Result<()> {
        self.process_parol(parol)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A test the checks the correct implementation of the `Default` for the `ParolGrammar`
    #[test]
    fn test_default() {
        let pg = ParolGrammar::default();
        assert_eq!(pg.title, None);
        assert_eq!(pg.comment, None);
        assert_eq!(pg.start_symbol, "");
        assert_eq!(pg.scanner_configurations.len(), 1);
        assert_eq!(pg.user_type_definitions.len(), 0);
        assert_eq!(pg.nt_type_definitions.len(), 0);
        assert_eq!(pg.grammar_type, GrammarType::LLK);
        assert_eq!(pg.productions.len(), 0);
    }
}
