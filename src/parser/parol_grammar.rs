use crate::parser::parol_grammar_trait::ParolGrammarTrait;
use id_tree::Tree;
use log::trace;
use parol_runtime::errors::*;
use parol_runtime::parser::{ParseTreeStackEntry, ParseTreeType};
use std::fmt::{Debug, Display, Error, Formatter};

// To rebuild the parser sources from scratch use the command build_parsers.ps1

// Test run:
// cargo run --bin parol -- -f .\src\parser\parol-grammar.par -v

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Factor {
    Group(Alternations),
    Repeat(Alternations),
    Optional(Alternations),
    Terminal(String, Vec<usize>),
    NonTerminal(String),
    ScannerSwitch(usize),
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
            Self::NonTerminal(n) => write!(f, "N({})", n),
            Self::ScannerSwitch(n) => write!(f, "S({})", n),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Alternation(pub Vec<Factor>);

impl Display for Alternation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(
            f,
            "Alt({})",
            self.0
                .iter()
                .map(|f| format!("{}", f))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl Alternation {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn push(&mut self, fac: Factor) {
        self.0.push(fac)
    }

    fn reverse(&mut self) {
        self.0.reverse()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Alternations(pub Vec<Alternation>);

impl Alternations {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn push(&mut self, alt: Alternation) {
        self.0.push(alt)
    }

    fn reverse(&mut self) {
        self.0.reverse()
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

#[derive(Debug, Clone)]
pub struct Production {
    pub lhs: String,
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

#[derive(Debug, Clone)]
pub struct ScannerConfig {
    pub name: String,
    pub line_comments: Vec<String>,
    pub block_comments: Vec<(String, String)>,
    pub auto_newline_off: bool,
    pub auto_ws_off: bool,
}

#[derive(Debug, Clone)]
pub enum ParolGrammarItem {
    Prod(Production),
    Alts(Alternations),
    Alt(Alternation),
    Fac(Factor),
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

///
/// Data structure used to build up a parol::GrammarConfig during parsing.
///
#[derive(Debug, Clone)]
pub struct ParolGrammar {
    pub item_stack: Vec<ParolGrammarItem>,
    pub title: Option<String>,
    pub comment: Option<String>,
    pub start_symbol: String,
    pub scanner_configurations: Vec<ScannerConfig>,
    current_scanner: ScannerConfig,
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
        }
    }
}

impl ParolGrammar {
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

    fn handle_scanner_state(&mut self, context: &str) -> Result<()> {
        let l = self.pop(context);
        let s = self.pop(context);

        match (&l, &s) {
            (
                Some(ParolGrammarItem::StateList(l)),
                Some(ParolGrammarItem::Fac(Factor::NonTerminal(s))),
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
                } else {
                    Err(format!("{}: Unknown scanner name '{}'", context, s).into())
                }
            }
            _ => Err(format!(
                "{}: Expected [StateList, Factor::NonTerminal] on TOS, found [{:?}, {:?}]",
                context, l, s
            )
            .into()),
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
        writeln!(f, "current_scanner: {}", self.current_scanner)?;
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
        if let Some(ParolGrammarItem::Fac(Factor::NonTerminal(s))) = self.pop(context) {
            self.start_symbol = s;
            Ok(())
        } else {
            Err(format!("{}: Expected 'Fac(Factor::NonTerminal)' on TOS.", context).into())
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
            Err(format!("{}: Expected 'Fac(Factor::Terminal)' on TOS.", context).into())
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
            Err(format!("{}: Expected 'Fac(Factor::Terminal)' on TOS.", context).into())
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
            Err(format!("{}: Expected 'Fac(Factor::Terminal)' on TOS.", context).into())
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
                Err(format!("{}: Expected 'Fac(Factor::Terminal)' on TOS.", context).into())
            }
        } else {
            Err(format!("{}: Expected 'Fac(Factor::Terminal)' on TOS.", context).into())
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

    /// Semantic action for production 20:
    ///
    /// Production: Identifier ":" Alternations ";";
    ///
    fn production_20(
        &mut self,
        _identifier_0: &ParseTreeStackEntry,
        _colon_1: &ParseTreeStackEntry,
        _alternations_2: &ParseTreeStackEntry,
        _semicolon_3: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "production_20";
        if let Some(ParolGrammarItem::Alts(mut rhs)) = self.pop(context) {
            if let Some(ParolGrammarItem::Fac(Factor::NonTerminal(lhs))) = self.pop(context) {
                rhs.reverse();
                self.item_stack
                    .push(ParolGrammarItem::Prod(Production::new(lhs, rhs)));
                Ok(())
            } else {
                Err(format!("{}: Expected 'Fac(Factor::NonTerminal)' on TOS.", context).into())
            }
        } else {
            Err(format!("{}: Expected 'Alts' on TOS.", context).into())
        }
    }

    /// Semantic action for production 21:
    ///
    /// Alternations: Alternation AlternationsSuffix;
    ///
    fn alternations_21(
        &mut self,
        _alternation_0: &ParseTreeStackEntry,
        _alternations_suffix_1: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "alternations_21";
        if let Some(ParolGrammarItem::Alts(mut alts)) = self.pop(context) {
            if let Some(ParolGrammarItem::Alt(mut alt)) = self.pop(context) {
                alt.reverse();
                alts.push(alt);
                self.push(ParolGrammarItem::Alts(alts), context);
                Ok(())
            } else {
                Err(format!("{}: Expected 'Alt' on TOS.", context).into())
            }
        } else {
            Err(format!("{}: Expected 'Alts' on TOS.", context).into())
        }
    }

    /// Semantic action for production 23:
    ///
    /// AlternationsSuffix: ;
    ///
    fn alternations_suffix_23(&mut self, _parse_tree: &Tree<ParseTreeType>) -> Result<()> {
        let _context = "alternations_suffix_23";
        self.item_stack
            .push(ParolGrammarItem::Alts(Alternations::new()));
        Ok(())
    }

    /// Semantic action for production 24:
    ///
    /// AlternationsRest: "\|" Alternation AlternationsRestSuffix;
    ///
    fn alternations_rest_24(
        &mut self,
        _or_0: &ParseTreeStackEntry,
        _alternation_1: &ParseTreeStackEntry,
        _alternations_rest_suffix_2: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "alternations_rest_24";
        if let Some(ParolGrammarItem::Alts(mut alts)) = self.pop(context) {
            if let Some(ParolGrammarItem::Alt(mut alt)) = self.pop(context) {
                alt.reverse();
                alts.push(alt);
                self.push(ParolGrammarItem::Alts(alts), context);
                Ok(())
            } else {
                Err(format!("{}: Expected 'Alt' on TOS.", context).into())
            }
        } else {
            Err(format!("{}: Expected 'Alts' on TOS.", context).into())
        }
    }

    /// Semantic action for production 26:
    ///
    /// AlternationsRestSuffix: ;
    ///
    fn alternations_rest_suffix_26(&mut self, _parse_tree: &Tree<ParseTreeType>) -> Result<()> {
        let _context = "alternations_rest_suffix_26";
        self.item_stack
            .push(ParolGrammarItem::Alts(Alternations::new()));
        Ok(())
    }

    /// Semantic action for production 28:
    ///
    /// Alternation: ;
    ///
    fn alternation_28(&mut self, _parse_tree: &Tree<ParseTreeType>) -> Result<()> {
        let _context = "alternation_28";
        self.item_stack
            .push(ParolGrammarItem::Alt(Alternation::new()));
        Ok(())
    }

    /// Semantic action for production 29:
    ///
    /// AlternationRest: Factor AlternationRestSuffix;
    ///
    fn alternation_rest_29(
        &mut self,
        _factor_0: &ParseTreeStackEntry,
        _alternation_rest_suffix_1: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "alternation_rest_29";
        //trace!("{}", self.trace_item_stack(context));
        if let Some(ParolGrammarItem::Alt(mut alt)) = self.pop(context) {
            if let Some(ParolGrammarItem::Fac(fac)) = self.pop(context) {
                alt.push(fac);
                self.push(ParolGrammarItem::Alt(alt), context);
                Ok(())
            } else {
                Err(format!("{}: Expected 'Fac' on TOS.", context).into())
            }
        } else {
            Err(format!("{}: Expected 'Alt' on TOS.", context).into())
        }
    }

    /// Semantic action for production 31:
    ///
    /// AlternationRestSuffix: ;
    ///
    fn alternation_rest_suffix_31(&mut self, _parse_tree: &Tree<ParseTreeType>) -> Result<()> {
        let _context = "alternation_rest_suffix_31";
        self.item_stack
            .push(ParolGrammarItem::Alt(Alternation::new()));
        Ok(())
    }

    /// Semantic action for production 41:
    ///
    /// TokenWithStates: "<" Identifier ">" String;
    ///
    fn token_with_states_41(
        &mut self,
        _l_t_0: &ParseTreeStackEntry,
        _state_list_1: &ParseTreeStackEntry,
        _g_t_2: &ParseTreeStackEntry,
        _string_3: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "token_with_state_41";
        trace!("{}", self.trace_item_stack(context));
        if let Some(ParolGrammarItem::Fac(Factor::Terminal(s, _))) = self.pop(context) {
            if let Some(ParolGrammarItem::StateList(sc)) = self.pop(context) {
                self.item_stack
                    .push(ParolGrammarItem::Fac(Factor::Terminal(s, sc)));
                Ok(())
            } else {
                Err(format!("{}: Expected 'StateList' on TOS.", context).into())
            }
        } else {
            Err(format!("{}: Expected 'Factor::Terminal' on TOS.", context).into())
        }
    }

    /// Semantic action for production 42:
    ///
    /// Group: "\(" Alternations "\)";
    ///
    fn group_42(
        &mut self,
        _l_paren_0: &ParseTreeStackEntry,
        _alternations_1: &ParseTreeStackEntry,
        _r_paren_2: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "group_42";
        if let Some(ParolGrammarItem::Alts(alts)) = self.pop(context) {
            self.item_stack
                .push(ParolGrammarItem::Fac(Factor::Group(alts)));
            Ok(())
        } else {
            Err(format!("{}: Expected 'Alts' on TOS.", context).into())
        }
    }

    /// Semantic action for production 43:
    ///
    /// Optional: "\[" Alternations "\]";
    ///
    fn optional_43(
        &mut self,
        _l_bracket_0: &ParseTreeStackEntry,
        _alternations_1: &ParseTreeStackEntry,
        _r_bracket_2: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "optional_43";
        if let Some(ParolGrammarItem::Alts(alts)) = self.pop(context) {
            self.item_stack
                .push(ParolGrammarItem::Fac(Factor::Optional(alts)));
            Ok(())
        } else {
            Err(format!("{}: Expected 'Alts' on TOS.", context).into())
        }
    }

    /// Semantic action for production 44:
    ///
    /// Repeat: "\{" Alternations "\}";
    ///
    fn repeat_44(
        &mut self,
        _l_brace_0: &ParseTreeStackEntry,
        _alternations_1: &ParseTreeStackEntry,
        _r_brace_2: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "repeat_44";
        if let Some(ParolGrammarItem::Alts(alts)) = self.pop(context) {
            self.item_stack
                .push(ParolGrammarItem::Fac(Factor::Repeat(alts)));
            Ok(())
        } else {
            Err(format!("{}: Expected 'Alts' on TOS.", context).into())
        }
    }

    /// Semantic action for production 45:
    ///
    /// Identifier: "[a-zA-Z_]\w*";
    ///
    fn identifier_45(
        &mut self,
        identifier_0: &ParseTreeStackEntry,
        parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "identifier_45";
        let parse_tree_item = identifier_0.get_parse_tree_type(parse_tree);
        if let ParseTreeType::T(t) = parse_tree_item {
            self.item_stack
                .push(ParolGrammarItem::Fac(Factor::NonTerminal(t.symbol.clone())));
            Ok(())
        } else {
            Err(format!("{}: Token expected, found {}", context, parse_tree_item).into())
        }
    }

    /// Semantic action for production 46:
    ///
    /// String: "\u{0022}([^\\]|\\.)*?\u{0022}";
    ///
    fn string_46(
        &mut self,
        string_0: &ParseTreeStackEntry,
        parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "string_46";
        let parse_tree_item = string_0.get_parse_tree_type(parse_tree);
        if let ParseTreeType::T(t) = parse_tree_item {
            // Trim double quotes here
            let s = t.symbol.clone().trim_matches('"').to_owned();
            self.item_stack
                .push(ParolGrammarItem::Fac(Factor::Terminal(s, vec![0])));
            Ok(())
        } else {
            Err(format!("{}: Token expected, found {}", context, parse_tree_item).into())
        }
    }

    /// Semantic action for production 48:
    ///
    /// ScannerStateSuffix: ScannerStateRest "\}";
    ///
    fn scanner_state_suffix_48(
        &mut self,
        _scanner_state_rest_0: &ParseTreeStackEntry,
        _r_brace_1: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "scanner_state_suffix_48";
        trace!("{}", self.trace_item_stack(context));
        if let Some(ParolGrammarItem::Fac(Factor::NonTerminal(n))) = self.pop(context) {
            trace!("{}", self);
            self.current_scanner.name = n;
            self.scanner_configurations
                .push(self.current_scanner.clone());
            self.current_scanner = ScannerConfig::default();
            trace!("{}", self);
            Ok(())
        } else {
            Err(format!("{}: Expected 'Factor::NonTerminal' on TOS.", context).into())
        }
    }

    /// Semantic action for production 49:
    ///
    /// ScannerStateSuffix: "\}";
    ///
    fn scanner_state_suffix_49(
        &mut self,
        _r_brace_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "scanner_state_suffix_49";
        trace!("{}", self.trace_item_stack(context));
        if let Some(ParolGrammarItem::Fac(Factor::NonTerminal(n))) = self.pop(context) {
            trace!("{}", self);
            self.current_scanner.name = n;
            self.scanner_configurations
                .push(self.current_scanner.clone());
            self.current_scanner = ScannerConfig::default();
            trace!("{}", self);
            Ok(())
        } else {
            Err(format!("{}: Expected 'Factor::NonTerminal' on TOS.", context).into())
        }
    }

    /// Semantic action for production 53:
    ///
    /// StateList: Identifier StateListRest;
    ///
    fn state_list_53(
        &mut self,
        _identifier_0: &ParseTreeStackEntry,
        _state_list_rest_1: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "state_list_53";
        trace!("{}", self.trace_item_stack(context));
        self.handle_scanner_state(context)
    }

    /// Semantic action for production 54:
    ///
    /// StateListRest: "," Identifier StateListRest;
    ///
    fn state_list_rest_54(
        &mut self,
        _comma_0: &ParseTreeStackEntry,
        _identifier_1: &ParseTreeStackEntry,
        _state_list_rest_2: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "state_list_rest_54";
        trace!("{}", self.trace_item_stack(context));
        self.handle_scanner_state(context)
    }

    /// Semantic action for production 55:
    ///
    /// StateListRest: ;
    ///
    fn state_list_rest_55(&mut self, _parse_tree: &Tree<ParseTreeType>) -> Result<()> {
        let context = "state_list_rest_55";
        // Start with an empty state list
        self.push(ParolGrammarItem::StateList(vec![]), context);
        Ok(())
    }

    /// Semantic action for production 56:
    ///
    /// ScannerSwitch: "%sc" "\(" ScannerNameOpt "\)";
    ///
    fn scanner_switch_56(
        &mut self,
        _percent_sc_0: &ParseTreeStackEntry,
        _l_paren_1: &ParseTreeStackEntry,
        _scanner_name_opt_2: &ParseTreeStackEntry,
        _r_paren_3: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "scanner_switch_56";
        if let Some(ParolGrammarItem::Fac(Factor::NonTerminal(s))) = self.pop(context) {
            if let Some(scanner_state) = self
                .scanner_configurations
                .iter()
                .position(|sc| sc.name == *s)
            {
                self.item_stack
                    .push(ParolGrammarItem::Fac(Factor::ScannerSwitch(scanner_state)));
                trace!("{}", self.trace_item_stack(context));
                Ok(())
            } else {
                Err(format!("{}: Unknown scanner name '{}'", context, s).into())
            }
        } else {
            Err(format!("{}: Expected 'Fac(Factor::NonTerminal)' on TOS.", context).into())
        }
    }

    /// Semantic action for production 58:
    ///
    /// ScannerNameOpt: ;
    ///
    fn scanner_name_opt_58(&mut self, _parse_tree: &Tree<ParseTreeType>) -> Result<()> {
        self.item_stack
            .push(ParolGrammarItem::Fac(Factor::NonTerminal(
                "INITIAL".to_string(),
            )));
        Ok(())
    }
}
