use super::parol_grammar_trait::ParolGrammarTrait;
use super::ParolParserError;
use crate::grammar::ProductionAttribute;
use crate::grammar::{Decorate, SymbolAttribute};
use id_tree::Tree;
use log::trace;
use miette::{miette, IntoDiagnostic, Result};
use parol_runtime::errors::FileSource;
use parol_runtime::parser::{ParseTreeStackEntry, ParseTreeType};
use std::fmt::{Debug, Display, Error, Formatter};
use std::path::PathBuf;

lazy_static! {
    /// Used for implementation of trait `Default` for `&ParolGrammar`.
    static ref DEFAULT_PAROL_GRAMMAR: ParolGrammar =
        ParolGrammar::default();
}

// To rebuild the parser sources from scratch use the command build_parsers.ps1

// Test run:
// parol -f .\src\parser\parol-grammar.par -v

///
/// [Factor] is part of the structure of the grammar representation
///
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
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
}

impl Display for Factor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match self {
            Self::Group(g) => write!(f, "({})", g),
            Self::Repeat(r) => write!(f, "{{{}}}", r),
            Self::Optional(o) => write!(f, "[{}]", o),
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

///
/// An Alternation is a sequence of factors.
/// Valid operation on Alternation is "|".
///
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
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
}

///
/// [Alternations] is part of the structure of the grammar representation
///
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Alternations(pub Vec<Alternation>);

impl Alternations {
    pub(crate) fn new() -> Self {
        Self(Vec::new())
    }

    pub(crate) fn insert(&mut self, alt: Alternation) {
        self.0.insert(0, alt)
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
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Production {
    /// Left-hand side non-terminal
    pub lhs: String,
    /// Right-hand side
    pub rhs: Alternations,
}

impl Production {
    fn new(lhs: String, rhs: Alternations) -> Self {
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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

///
/// [ScannerConfig] is part of the structure of the grammar representation
///
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParolGrammar {
    /// The parsed items are pushed onto the item_stack.
    pub item_stack: Vec<ParolGrammarItem>,
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
}

impl Default for ParolGrammar {
    fn default() -> Self {
        Self {
            item_stack: Vec::new(),
            title: None,
            comment: None,
            start_symbol: String::default(),
            scanner_configurations: vec![ScannerConfig::default()],
            current_scanner: ScannerConfig::default(),
            file_name: PathBuf::default(),
        }
    }
}

impl Default for &ParolGrammar {
    fn default() -> Self {
        &DEFAULT_PAROL_GRAMMAR
    }
}

impl ParolGrammar {
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
            self.item_stack
                .iter()
                .rev()
                .map(|s| format!("  {}", s))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }

    fn handle_scanner_state(
        &mut self,
        context: &str,
        identifier_0: &ParseTreeStackEntry,
        parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let l = self.pop(context);
        let s = self.pop(context);

        match (&l, &s) {
            (
                Some(ParolGrammarItem::StateList(l)),
                Some(ParolGrammarItem::Fac(Factor::Identifier(s))),
            ) => {
                if let Some(scanner_state) = self
                    .scanner_configurations
                    .iter()
                    .position(|sc| sc.name == *s)
                {
                    let mut l = l.clone();
                    l.push(scanner_state);
                    self.push(ParolGrammarItem::StateList(l), context);
                    trace!("{}", self.trace_item_stack(context));
                    Ok(())
                } else if let ParseTreeStackEntry::Id(node_id) = identifier_0 {
                    // We need to navigate to the one and only child of the Identifier
                    // non-terminal to access the actual token.
                    let child = parse_tree
                        .get(node_id)
                        .and_then(|node_ref| parse_tree.get(&node_ref.children()[0]))
                        .into_diagnostic()?;
                    Err(miette!(ParolParserError::UnknownScanner {
                        context: context.to_owned(),
                        name: s.clone(),
                        input: FileSource::try_new(self.file_name.clone())?.into(),
                        token: child.data().token()?.into()
                    }))
                } else {
                    Err(miette!("{}: Unknown scanner name '{}'", context, s))
                }
            }
            _ => Err(miette!(
                "{}: Expected [StateList, Factor::Identifier] on TOS, found [{:?}, {:?}]",
                context,
                l,
                s
            )),
        }
    }

    fn push(&mut self, item: ParolGrammarItem, context: &str) {
        trace!("push   {}: {}", context, item);
        self.item_stack.push(item)
    }

    fn pop(&mut self, context: &str) -> Option<ParolGrammarItem> {
        if !self.item_stack.is_empty() {
            let item = self.item_stack.pop();
            if let Some(ref item) = item {
                trace!("pop    {}: {}", context, item);
            }
            item
        } else {
            None
        }
    }
}

impl Display for ParolGrammar {
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
            self.item_stack
                .iter()
                .map(|e| format!("{}", e))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

impl ParolGrammarTrait for ParolGrammar {
    ///
    /// Information provided by parser
    ///
    fn init(&mut self, file_name: &std::path::Path) {
        self.file_name = file_name.into();
    }

    /// Semantic action for production 2:
    ///
    /// StartDeclaration: "%start" Identifier;
    ///
    fn start_declaration_2(
        &mut self,
        _end_of_input_0: &ParseTreeStackEntry,
        _identifier_1: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "start_declaration_2";
        if let Some(ParolGrammarItem::Fac(Factor::Identifier(s))) = self.pop(context) {
            self.start_symbol = s;
            Ok(())
        } else {
            Err(miette!(
                "{}: Expected 'Fac(Factor::NonTerminal)' on TOS.",
                context
            ))
        }
    }

    /// Semantic action for production 4:
    ///
    /// Declarations: ;
    ///
    fn declarations_4(&mut self, _parse_tree: &Tree<ParseTreeType>) -> Result<()> {
        let context = "declarations_4";
        trace!("{}", self.trace_item_stack(context));
        self.scanner_configurations[0] = self.current_scanner.clone();
        self.current_scanner = ScannerConfig::default();
        Ok(())
    }

    /// Semantic action for production 5:
    ///
    /// Declaration: "%title" String;
    ///
    fn declaration_5(
        &mut self,
        _percent_title_0: &ParseTreeStackEntry,
        _string_1: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "declaration_5";
        if let Some(ParolGrammarItem::Fac(Factor::Terminal(s, _))) = self.pop(context) {
            self.title = Some(s);
            Ok(())
        } else {
            Err(miette!(
                "{}: Expected 'Fac(Factor::Terminal)' on TOS.",
                context
            ))
        }
    }

    /// Semantic action for production 6:
    ///
    /// Declaration: "%comment" String;
    ///
    fn declaration_6(
        &mut self,
        _percent_comment_0: &ParseTreeStackEntry,
        _string_1: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "declaration_6";
        if let Some(ParolGrammarItem::Fac(Factor::Terminal(s, _))) = self.pop(context) {
            self.comment = Some(s);
            Ok(())
        } else {
            Err(miette!(
                "{}: Expected 'Fac(Factor::Terminal)' on TOS.",
                context
            ))
        }
    }

    /// Semantic action for production 8:
    ///
    /// Declaration: "%line_comment" String;
    ///
    fn scanner_directives_8(
        &mut self,
        _percent_line_underscore_comment_0: &ParseTreeStackEntry,
        _string_1: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "scanner_directives_8";
        if let Some(ParolGrammarItem::Fac(Factor::Terminal(s, _))) = self.pop(context) {
            self.current_scanner.line_comments.push(s);
            Ok(())
        } else {
            Err(miette!(
                "{}: Expected 'Fac(Factor::Terminal)' on TOS.",
                context
            ))
        }
    }

    /// Semantic action for production 9:
    ///
    /// ScannerDirectives: "%block_comment" String String;
    ///
    fn scanner_directives_9(
        &mut self,
        _percent_block_underscore_comment_0: &ParseTreeStackEntry,
        _string_1: &ParseTreeStackEntry,
        _string_2: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "scanner_directives_9";
        if let Some(ParolGrammarItem::Fac(Factor::Terminal(s1, _))) = self.pop(context) {
            if let Some(ParolGrammarItem::Fac(Factor::Terminal(s2, _))) = self.pop(context) {
                self.current_scanner.block_comments.push((s2, s1));
                Ok(())
            } else {
                Err(miette!(
                    "{}: Expected 'Fac(Factor::Terminal)' on TOS.",
                    context
                ))
            }
        } else {
            Err(miette!(
                "{}: Expected 'Fac(Factor::Terminal)' on TOS.",
                context
            ))
        }
    }

    /// Semantic action for production 10:
    ///
    /// ScannerDirectives: "%auto_newline_off";
    ///
    fn scanner_directives_10(
        &mut self,
        _percent_auto_underscore_newline_underscore_off_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let _context = "scanner_directives_10";
        self.current_scanner.auto_newline_off = true;
        Ok(())
    }

    /// Semantic action for production 11:
    ///
    /// ScannerDirectives: "%auto_ws_off";
    ///
    fn scanner_directives_11(
        &mut self,
        _percent_auto_underscore_ws_underscore_off_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let _context = "scanner_directives_11";
        self.current_scanner.auto_ws_off = true;
        Ok(())
    }

    /// Semantic action for production 17:
    ///
    /// Production: Identifier ":" Alternations ";";
    ///
    fn production_17(
        &mut self,
        _identifier_0: &ParseTreeStackEntry,
        _colon_1: &ParseTreeStackEntry,
        _alternations_2: &ParseTreeStackEntry,
        _semicolon_3: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "production_17";
        if let Some(ParolGrammarItem::Alts(rhs)) = self.pop(context) {
            if let Some(ParolGrammarItem::Fac(Factor::Identifier(lhs))) = self.pop(context) {
                self.push(ParolGrammarItem::Prod(Production::new(lhs, rhs)), context);
                Ok(())
            } else {
                Err(miette!(
                    "{}: Expected 'Fac(Factor::Identifier)' on TOS.",
                    context
                ))
            }
        } else {
            Err(miette!("{}: Expected 'Alts' on TOS.", context))
        }
    }

    /// Semantic action for production 18:
    ///
    /// Alternations: Alternation AlternationsList;
    ///
    fn alternations_18(
        &mut self,
        _alternation_0: &ParseTreeStackEntry,
        _alternations_list_1: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "alternations_18";
        if let Some(ParolGrammarItem::Alts(mut alts)) = self.pop(context) {
            if let Some(ParolGrammarItem::Alt(alt)) = self.pop(context) {
                alts.insert(alt);
                self.push(ParolGrammarItem::Alts(alts), context);
                Ok(())
            } else {
                Err(miette!("{}: Expected 'Alt' on TOS.", context))
            }
        } else {
            Err(miette!("{}: Expected 'Alts' on TOS.", context))
        }
    }

    /// Semantic action for production 19:
    ///
    /// AlternationsList: "\|" Alternation AlternationsList;
    ///
    fn alternations_list_19(
        &mut self,
        _or_0: &ParseTreeStackEntry,
        _alternation_1: &ParseTreeStackEntry,
        _alternations_list_2: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "alternations_list_19";
        if let Some(ParolGrammarItem::Alts(mut alts)) = self.pop(context) {
            if let Some(ParolGrammarItem::Alt(alt)) = self.pop(context) {
                alts.insert(alt);
                self.push(ParolGrammarItem::Alts(alts), context);
                Ok(())
            } else {
                Err(miette!("{}: Expected 'Alt' on TOS.", context))
            }
        } else {
            Err(miette!("{}: Expected 'Alts' on TOS.", context))
        }
    }

    /// Semantic action for production 20:
    ///
    /// AlternationsList: ;
    ///
    fn alternations_list_20(&mut self, _parse_tree: &Tree<ParseTreeType>) -> Result<()> {
        let context = "alternations_list_20";
        self.push(ParolGrammarItem::Alts(Alternations::new()), context);
        Ok(())
    }

    /// Semantic action for production 22:
    ///
    /// AlternationList: Factor AlternationList;
    ///
    fn alternation_list_22(
        &mut self,
        _factor_0: &ParseTreeStackEntry,
        _alternation_list_1: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "alternation_list_22";
        //trace!("{}", self.trace_item_stack(context));
        if let Some(ParolGrammarItem::Alt(mut alt)) = self.pop(context) {
            if let Some(ParolGrammarItem::Fac(fac)) = self.pop(context) {
                alt.insert(fac);
                self.push(ParolGrammarItem::Alt(alt), context);
                Ok(())
            } else {
                Err(miette!("{}: Expected 'Fac' on TOS.", context))
            }
        } else {
            Err(miette!("{}: Expected 'Alt' on TOS.", context))
        }
    }

    /// Semantic action for production 23:
    ///
    /// AlternationList: ;
    ///
    fn alternation_list_23(&mut self, _parse_tree: &Tree<ParseTreeType>) -> Result<()> {
        let context = "alternation_list_23";
        self.push(ParolGrammarItem::Alt(Alternation::new()), context);
        Ok(())
    }

    /// Semantic action for production 28:
    ///
    /// Symbol: Identifier;
    ///
    fn symbol_28(
        &mut self,
        _identifier_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "symbol_28";
        if let Some(ParolGrammarItem::Fac(Factor::Identifier(nt))) = self.pop(context) {
            self.push(
                ParolGrammarItem::Fac(Factor::NonTerminal(nt, SymbolAttribute::default())),
                context,
            );
            Ok(())
        } else {
            Err(miette!(
                "{}: Expected 'Fac(Factor::Identifier)' on TOS.",
                context
            ))
        }
    }

    /// Semantic action for production 33:
    ///
    /// TokenWithStates: "<" StateList ">" String;
    ///
    fn token_with_states_33(
        &mut self,
        _l_t_0: &ParseTreeStackEntry,
        _state_list_1: &ParseTreeStackEntry,
        _g_t_2: &ParseTreeStackEntry,
        _string_3: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "token_with_states_33";
        trace!("{}", self.trace_item_stack(context));
        if let Some(ParolGrammarItem::Fac(Factor::Terminal(s, _))) = self.pop(context) {
            if let Some(ParolGrammarItem::StateList(sc)) = self.pop(context) {
                self.push(ParolGrammarItem::Fac(Factor::Terminal(s, sc)), context);
                Ok(())
            } else {
                Err(miette!("{}: Expected 'StateList' on TOS.", context))
            }
        } else {
            Err(miette!("{}: Expected 'Factor::Terminal' on TOS.", context))
        }
    }

    /// Semantic action for production 34:
    ///
    /// Group: "\(" Alternations "\)";
    ///
    fn group_34(
        &mut self,
        l_paren_0: &ParseTreeStackEntry,
        _alternations_1: &ParseTreeStackEntry,
        r_paren_2: &ParseTreeStackEntry,
        parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "group_34";
        trace!("{}", self.trace_item_stack(context));
        if let Some(ParolGrammarItem::Alts(alts)) = self.pop(context) {
            if alts.0.is_empty() || (alts.0.len() == 1 && alts.0[0].0.is_empty()) {
                Err(miette!(ParolParserError::EmptyGroup {
                    context: context.to_owned(),
                    input: FileSource::try_new(self.file_name.clone())?.into(),
                    start: l_paren_0.token(parse_tree)?.into(),
                    end: r_paren_2.token(parse_tree)?.into(),
                }))
            } else {
                self.push(ParolGrammarItem::Fac(Factor::Group(alts)), context);
                Ok(())
            }
        } else {
            Err(miette!("{}: Expected 'Alts' on TOS.", context))
        }
    }

    /// Semantic action for production 35:
    ///
    /// Optional: "\[" Alternations "\]";
    ///
    fn optional_35(
        &mut self,
        l_bracket_0: &ParseTreeStackEntry,
        _alternations_1: &ParseTreeStackEntry,
        r_bracket_2: &ParseTreeStackEntry,
        parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "optional_35";
        trace!("{}", self.trace_item_stack(context));
        if let Some(ParolGrammarItem::Alts(alts)) = self.pop(context) {
            if alts.0.is_empty() || (alts.0.len() == 1 && alts.0[0].0.is_empty()) {
                Err(miette!(ParolParserError::EmptyOptional {
                    context: context.to_owned(),
                    input: FileSource::try_new(self.file_name.clone())?.into(),
                    start: l_bracket_0.token(parse_tree)?.into(),
                    end: r_bracket_2.token(parse_tree)?.into(),
                }))
            } else {
                self.push(ParolGrammarItem::Fac(Factor::Optional(alts)), context);
                Ok(())
            }
        } else {
            Err(miette!("{}: Expected 'Alts' on TOS.", context))
        }
    }

    /// Semantic action for production 36:
    ///
    /// Repeat: "\{" Alternations "\}";
    ///
    fn repeat_36(
        &mut self,
        l_brace_0: &ParseTreeStackEntry,
        _alternations_1: &ParseTreeStackEntry,
        r_brace_2: &ParseTreeStackEntry,
        parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "repeat_36";
        trace!("{}", self.trace_item_stack(context));
        if let Some(ParolGrammarItem::Alts(alts)) = self.pop(context) {
            if alts.0.is_empty() || (alts.0.len() == 1 && alts.0[0].0.is_empty()) {
                Err(miette!(ParolParserError::EmptyRepetition {
                    context: context.to_owned(),
                    input: FileSource::try_new(self.file_name.clone())?.into(),
                    start: l_brace_0.token(parse_tree)?.into(),
                    end: r_brace_2.token(parse_tree)?.into(),
                }))
            } else {
                self.push(ParolGrammarItem::Fac(Factor::Repeat(alts)), context);
                Ok(())
            }
        } else {
            Err(miette!("{}: Expected 'Alts' on TOS.", context))
        }
    }

    /// Semantic action for production 37:
    ///
    /// Identifier: "[a-zA-Z_]\w*";
    ///
    fn identifier_37(
        &mut self,
        identifier_0: &ParseTreeStackEntry,
        parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "identifier_37";
        let parse_tree_item = identifier_0.get_parse_tree_type(parse_tree);
        if let ParseTreeType::T(t) = parse_tree_item {
            self.push(
                ParolGrammarItem::Fac(Factor::Identifier(t.symbol.to_owned())),
                context,
            );
            Ok(())
        } else {
            Err(miette!(
                "{}: Token expected, found {}",
                context,
                parse_tree_item
            ))
        }
    }

    /// Semantic action for production 38:
    ///
    /// String: "\u{0022}([^\\]|\\.)*?\u{0022}";
    ///
    fn string_38(
        &mut self,
        string_0: &ParseTreeStackEntry,
        parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "string_38";
        let parse_tree_item = string_0.get_parse_tree_type(parse_tree);
        if let ParseTreeType::T(t) = parse_tree_item {
            // Trim double quotes here
            let s = t.symbol.trim_matches('"').to_owned();
            self.push(ParolGrammarItem::Fac(Factor::Terminal(s, vec![0])), context);
            Ok(())
        } else {
            Err(miette!(
                "{}: Token expected, found {}",
                context,
                parse_tree_item
            ))
        }
    }

    /// Semantic action for production 39:
    ///
    /// ScannerState: "%scanner" Identifier "\{" ScannerStateList "\}";
    ///
    fn scanner_state_39(
        &mut self,
        _percent_scanner_0: &ParseTreeStackEntry,
        _identifier_1: &ParseTreeStackEntry,
        _l_brace_2: &ParseTreeStackEntry,
        _scanner_state_list_3: &ParseTreeStackEntry,
        _r_brace_4: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "scanner_state_39";
        trace!("{}", self.trace_item_stack(context));
        if let Some(ParolGrammarItem::Fac(Factor::Identifier(n))) = self.pop(context) {
            trace!("{}", self);
            self.current_scanner.name = n;
            self.scanner_configurations
                .push(self.current_scanner.clone());
            self.current_scanner = ScannerConfig::default();
            trace!("{}", self);
            Ok(())
        } else {
            Err(miette!(
                "{}: Expected 'Factor::NonTerminal' on TOS.",
                context
            ))
        }
    }

    /// Semantic action for production 42:
    ///
    /// StateList: Identifier StateListList;
    ///
    fn state_list_42(
        &mut self,
        identifier_0: &ParseTreeStackEntry,
        _state_list_list_1: &ParseTreeStackEntry,
        parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "state_list_42";
        trace!("{}", self.trace_item_stack(context));
        self.handle_scanner_state(context, identifier_0, parse_tree)
    }

    /// Semantic action for production 43:
    ///
    /// StateListRest: "," Identifier StateListRest;
    ///
    fn state_list_rest_43(
        &mut self,
        _comma_0: &ParseTreeStackEntry,
        identifier_1: &ParseTreeStackEntry,
        _state_list_rest_2: &ParseTreeStackEntry,
        parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "state_list_rest_43";
        trace!("{}", self.trace_item_stack(context));
        self.handle_scanner_state(context, identifier_1, parse_tree)
    }

    /// Semantic action for production 44:
    ///
    /// StateListRest: ;
    ///
    fn state_list_rest_44(&mut self, _parse_tree: &Tree<ParseTreeType>) -> Result<()> {
        let context = "state_list_rest_44";
        // Start with an empty state list
        self.push(ParolGrammarItem::StateList(vec![]), context);
        Ok(())
    }

    /// Semantic action for production 45:
    ///
    /// ScannerSwitch: "%sc" "\(" ScannerNameOpt "\)";
    ///
    fn scanner_switch_45(
        &mut self,
        _percent_sc_0: &ParseTreeStackEntry,
        _l_paren_1: &ParseTreeStackEntry,
        _scanner_name_opt_2: &ParseTreeStackEntry,
        _r_paren_3: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "scanner_switch_45";
        if let Some(ParolGrammarItem::Fac(Factor::Identifier(s))) = self.pop(context) {
            if let Some(scanner_state) = self
                .scanner_configurations
                .iter()
                .position(|sc| sc.name == *s)
            {
                self.push(
                    ParolGrammarItem::Fac(Factor::ScannerSwitch(scanner_state)),
                    context,
                );
                trace!("{}", self.trace_item_stack(context));
                Ok(())
            } else {
                Err(miette!("{}: Unknown scanner name '{}'", context, s))
            }
        } else {
            Err(miette!(
                "{}: Expected 'Fac(Factor::NonTerminal)' on TOS.",
                context
            ))
        }
    }

    /// Semantic action for production 46:
    ///
    /// ScannerSwitch: "%push" "\(" Identifier "\)";
    ///
    fn scanner_switch_46(
        &mut self,
        _percent_push_0: &ParseTreeStackEntry,
        _l_paren_1: &ParseTreeStackEntry,
        _identifier_2: &ParseTreeStackEntry,
        _r_paren_3: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "scanner_switch_46";
        if let Some(ParolGrammarItem::Fac(Factor::Identifier(s))) = self.pop(context) {
            if let Some(scanner_state) = self
                .scanner_configurations
                .iter()
                .position(|sc| sc.name == *s)
            {
                self.push(
                    ParolGrammarItem::Fac(Factor::ScannerSwitchPush(scanner_state)),
                    context,
                );
                trace!("{}", self.trace_item_stack(context));
                Ok(())
            } else {
                Err(miette!("{}: Unknown scanner name '{}'", context, s))
            }
        } else {
            Err(miette!(
                "{}: Expected 'Fac(Factor::NonTerminal)' on TOS.",
                context
            ))
        }
    }

    /// Semantic action for production 47:
    ///
    /// ScannerSwitch: "%pop" "\(" "\)";
    ///
    fn scanner_switch_47(
        &mut self,
        _percent_pop_0: &ParseTreeStackEntry,
        _l_paren_1: &ParseTreeStackEntry,
        _r_paren_2: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "scanner_switch_47";
        self.push(ParolGrammarItem::Fac(Factor::ScannerSwitchPop), context);
        trace!("{}", self.trace_item_stack(context));
        Ok(())
    }

    /// Semantic action for production 49:
    ///
    /// ScannerNameOpt: ;
    ///
    fn scanner_name_opt_49(&mut self, _parse_tree: &Tree<ParseTreeType>) -> Result<()> {
        let context = "scanner_name_opt_49";
        self.push(
            ParolGrammarItem::Fac(Factor::Identifier("INITIAL".to_string())),
            context,
        );
        Ok(())
    }
}
