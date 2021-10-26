use crate::parser::parol_grammar_trait::ParolGrammarTrait;
use id_tree::Tree;
use log::trace;
use parol_runtime::parser::errors::*;
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
    Terminal(String),
    NonTerminal(String),
}

impl Display for Factor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match self {
            Self::Group(g) => write!(f, "({})", g),
            Self::Repeat(r) => write!(f, "{{{}}}", r),
            Self::Optional(o) => write!(f, "[{}]", o),
            Self::Terminal(t) => write!(f, "T({})", t),
            Self::NonTerminal(n) => write!(f, "N({})", n),
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
pub enum ParolGrammarItem {
    Prod(Production),
    Alts(Alternations),
    Alt(Alternation),
    Fac(Factor),
}

impl Display for ParolGrammarItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match self {
            Self::Prod(p) => write!(f, "{}", p),
            Self::Alts(a) => write!(f, "{}", a),
            Self::Alt(a) => write!(f, "{}", a),
            Self::Fac(t) => write!(f, "{}", t),
        }
    }
}

///
/// Data structure used to build up a parol::GrammarConfig during parsing.
///
#[derive(Debug, Clone, Default)]
pub struct ParolGrammar {
    pub ast_stack: Vec<ParolGrammarItem>,
    pub title: Option<String>,
    pub comment: Option<String>,
    pub start_symbol: String,
    pub line_comments: Vec<String>,
    pub block_comments: Vec<(String, String)>,
    pub auto_newline_off: bool,
}

impl ParolGrammar {
    pub fn new() -> Self {
        ParolGrammar::default()
    }

    #[allow(dead_code)]
    // Use this function for debugging purposes:
    // $env:RUST_LOG="parol::parser=trace"
    //
    // trace!("{}", self.trace_ast_stack(context));
    fn trace_ast_stack(&self, context: &str) -> String {
        format!(
            "Ast stack at {}:\n{}",
            context,
            self.ast_stack
                .iter()
                .rev()
                .map(|s| format!("  {}", s))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

impl Display for ParolGrammar {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        writeln!(f, "title: {:?}", self.title)?;
        writeln!(f, "comment: {:?}", self.comment)?;
        writeln!(f, "start_symbol: {}", self.start_symbol)?;
        writeln!(f, "line_comments: {:?}", self.line_comments)?;
        writeln!(f, "block_comments: {:?}", self.block_comments)?;
        writeln!(
            f,
            "{}",
            self.ast_stack
                .iter()
                .map(|e| format!("{}", e))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

impl ParolGrammarTrait for ParolGrammar {
    /// Semantic action for production 7:
    ///
    /// StartDeclaration: "%start" Identifier;
    ///
    fn start_declaration_7(
        &mut self,
        _percent_start_0: &ParseTreeStackEntry,
        _identifier_1: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "start_declaration_7";
        if let Some(ParolGrammarItem::Fac(Factor::NonTerminal(s))) = self.ast_stack.pop() {
            self.start_symbol = s;
            Ok(())
        } else {
            Err(format!("{}: Expected 'Fac(Factor::NonTerminal)' on TOS.", context).into())
        }
    }

    /// Semantic action for production 8:
    ///
    /// Declaration: "%title" String;
    ///
    fn declaration_8(
        &mut self,
        _percent_title_0: &ParseTreeStackEntry,
        _string_1: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "declaration_8";
        if let Some(ParolGrammarItem::Fac(Factor::Terminal(s))) = self.ast_stack.pop() {
            self.title = Some(s);
            Ok(())
        } else {
            Err(format!("{}: Expected 'Fac(Factor::Terminal)' on TOS.", context).into())
        }
    }

    /// Semantic action for production 9:
    ///
    /// Declaration: "%comment" String;
    ///
    fn declaration_9(
        &mut self,
        _percent_comment_0: &ParseTreeStackEntry,
        _string_1: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "declaration_9";
        if let Some(ParolGrammarItem::Fac(Factor::Terminal(s))) = self.ast_stack.pop() {
            self.comment = Some(s);
            Ok(())
        } else {
            Err(format!("{}: Expected 'Fac(Factor::Terminal)' on TOS.", context).into())
        }
    }

    /// Semantic action for production 10:
    ///
    /// Declaration: "%line_comment" String;
    ///
    fn declaration_10(
        &mut self,
        _percent_line_underscore_comment_0: &ParseTreeStackEntry,
        _string_1: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "declaration_10";
        if let Some(ParolGrammarItem::Fac(Factor::Terminal(s))) = self.ast_stack.pop() {
            self.line_comments.push(s);
            Ok(())
        } else {
            Err(format!("{}: Expected 'Fac(Factor::Terminal)' on TOS.", context).into())
        }
    }

    /// Semantic action for production 11:
    ///
    /// Declaration: "%block_comment" String String;
    ///
    fn declaration_11(
        &mut self,
        _percent_block_underscore_comment_0: &ParseTreeStackEntry,
        _string_1: &ParseTreeStackEntry,
        _string_2: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "declaration_11";
        if let Some(ParolGrammarItem::Fac(Factor::Terminal(s1))) = self.ast_stack.pop() {
            if let Some(ParolGrammarItem::Fac(Factor::Terminal(s2))) = self.ast_stack.pop() {
                self.block_comments.push((s2, s1));
                Ok(())
            } else {
                Err(format!("{}: Expected 'Fac(Factor::Terminal)' on TOS.", context).into())
            }
        } else {
            Err(format!("{}: Expected 'Fac(Factor::Terminal)' on TOS.", context).into())
        }
    }

    /// Semantic action for production 12:
    ///
    /// Declaration: "%auto_newline_off";
    ///
    fn declaration_12(
        &mut self,
        _percent_auto_underscore_newline_underscore_off_0: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let _context = "declaration_12";
        self.auto_newline_off = true;
        Ok(())
    }

    /// Semantic action for production 19:
    ///
    /// Production: Identifier ":" Alternations ";";
    ///
    fn production_19(
        &mut self,
        _identifier_0: &ParseTreeStackEntry,
        _colon_1: &ParseTreeStackEntry,
        _alternations_2: &ParseTreeStackEntry,
        _semicolon_3: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "production_19";
        if let Some(ParolGrammarItem::Alts(mut rhs)) = self.ast_stack.pop() {
            if let Some(ParolGrammarItem::Fac(Factor::NonTerminal(lhs))) = self.ast_stack.pop() {
                rhs.reverse();
                self.ast_stack
                    .push(ParolGrammarItem::Prod(Production::new(lhs, rhs)));
                Ok(())
            } else {
                Err(format!("{}: Expected 'Fac(Factor::NonTerminal)' on TOS.", context).into())
            }
        } else {
            Err(format!("{}: Expected 'Alts' on TOS.", context).into())
        }
    }

    /// Semantic action for production 20:
    ///
    /// Alternations: Alternation AlternationsSuffix;
    ///
    fn alternations_20(
        &mut self,
        _alternation_0: &ParseTreeStackEntry,
        _alternations_suffix_1: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "alternations_20";
        if let Some(ParolGrammarItem::Alts(mut alts)) = self.ast_stack.pop() {
            if let Some(ParolGrammarItem::Alt(mut alt)) = self.ast_stack.pop() {
                alt.reverse();
                alts.push(alt);
                self.ast_stack.push(ParolGrammarItem::Alts(alts));
                Ok(())
            } else {
                Err(format!("{}: Expected 'Alt' on TOS.", context).into())
            }
        } else {
            Err(format!("{}: Expected 'Alts' on TOS.", context).into())
        }
    }

    /// Semantic action for production 22:
    ///
    /// AlternationsSuffix: ;
    ///
    fn alternations_suffix_22(&mut self, _parse_tree: &Tree<ParseTreeType>) -> Result<()> {
        let _context = "alternations_suffix_22";
        self.ast_stack
            .push(ParolGrammarItem::Alts(Alternations::new()));
        Ok(())
    }

    /// Semantic action for production 23:
    ///
    /// AlternationsRest: "\|" Alternation AlternationsRestSuffix;
    ///
    fn alternations_rest_23(
        &mut self,
        _esc_or_0: &ParseTreeStackEntry,
        _alternation_1: &ParseTreeStackEntry,
        _alternations_rest_suffix_2: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "alternations_rest_23";
        if let Some(ParolGrammarItem::Alts(mut alts)) = self.ast_stack.pop() {
            if let Some(ParolGrammarItem::Alt(mut alt)) = self.ast_stack.pop() {
                alt.reverse();
                alts.push(alt);
                self.ast_stack.push(ParolGrammarItem::Alts(alts));
                Ok(())
            } else {
                Err(format!("{}: Expected 'Alt' on TOS.", context).into())
            }
        } else {
            Err(format!("{}: Expected 'Alts' on TOS.", context).into())
        }
    }

    /// Semantic action for production 25:
    ///
    /// AlternationsRestSuffix: ;
    ///
    fn alternations_rest_suffix_25(&mut self, _parse_tree: &Tree<ParseTreeType>) -> Result<()> {
        let _context = "alternations_rest_suffix_25";
        self.ast_stack
            .push(ParolGrammarItem::Alts(Alternations::new()));
        Ok(())
    }

    /// Semantic action for production 27:
    ///
    /// Alternation: ;
    ///
    fn alternation_27(&mut self, _parse_tree: &Tree<ParseTreeType>) -> Result<()> {
        let _context = "alternation_27";
        self.ast_stack
            .push(ParolGrammarItem::Alt(Alternation::new()));
        Ok(())
    }

    /// Semantic action for production 28:
    ///
    /// AlternationRest: Factor AlternationRestSuffix;
    ///
    fn alternation_rest_28(
        &mut self,
        _factor_0: &ParseTreeStackEntry,
        _alternation_rest_suffix_1: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "alternation_rest_28";
        trace!("{}", self.trace_ast_stack(context));
        if let Some(ParolGrammarItem::Alt(mut alt)) = self.ast_stack.pop() {
            if let Some(ParolGrammarItem::Fac(fac)) = self.ast_stack.pop() {
                alt.push(fac);
                self.ast_stack.push(ParolGrammarItem::Alt(alt));
                Ok(())
            } else {
                Err(format!("{}: Expected 'Fac' on TOS.", context).into())
            }
        } else {
            Err(format!("{}: Expected 'Alt' on TOS.", context).into())
        }
    }

    /// Semantic action for production 30:
    ///
    /// AlternationRestSuffix: ;
    ///
    fn alternation_rest_suffix_30(&mut self, _parse_tree: &Tree<ParseTreeType>) -> Result<()> {
        let _context = "alternation_rest_suffix_30";
        self.ast_stack
            .push(ParolGrammarItem::Alt(Alternation::new()));
        Ok(())
    }

    /// Semantic action for production 37:
    ///
    /// Group: "\(" Alternations "\)";
    ///
    fn group_37(
        &mut self,
        _esc_l_paren_0: &ParseTreeStackEntry,
        _alternations_1: &ParseTreeStackEntry,
        _esc_r_paren_2: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "group_37";
        if let Some(ParolGrammarItem::Alts(alts)) = self.ast_stack.pop() {
            self.ast_stack
                .push(ParolGrammarItem::Fac(Factor::Group(alts)));
            Ok(())
        } else {
            Err(format!("{}: Expected 'Alts' on TOS.", context).into())
        }
    }

    /// Semantic action for production 38:
    ///
    /// Optional: "\[" Alternations "\]";
    ///
    fn optional_38(
        &mut self,
        _esc_l_bracket_0: &ParseTreeStackEntry,
        _alternations_1: &ParseTreeStackEntry,
        _esc_r_bracket_2: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "optional_38";
        if let Some(ParolGrammarItem::Alts(alts)) = self.ast_stack.pop() {
            self.ast_stack
                .push(ParolGrammarItem::Fac(Factor::Optional(alts)));
            Ok(())
        } else {
            Err(format!("{}: Expected 'Alts' on TOS.", context).into())
        }
    }

    /// Semantic action for production 39:
    ///
    /// Repeat: "\{" Alternations "\}";
    ///
    fn repeat_39(
        &mut self,
        _esc_l_brace_0: &ParseTreeStackEntry,
        _alternations_1: &ParseTreeStackEntry,
        _esc_r_brace_2: &ParseTreeStackEntry,
        _parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "repeat_39";
        if let Some(ParolGrammarItem::Alts(alts)) = self.ast_stack.pop() {
            self.ast_stack
                .push(ParolGrammarItem::Fac(Factor::Repeat(alts)));
            Ok(())
        } else {
            Err(format!("{}: Expected 'Alts' on TOS.", context).into())
        }
    }

    /// Semantic action for production 40:
    ///
    /// Identifier: "[a-zA-Z_]\w*";
    ///
    fn identifier_40(
        &mut self,
        identifier_0: &ParseTreeStackEntry,
        parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "identifier_40";
        let ast_item = identifier_0.get_ast_type(parse_tree);
        if let ParseTreeType::T(t) = ast_item {
            self.ast_stack
                .push(ParolGrammarItem::Fac(Factor::NonTerminal(t.symbol.clone())));
            Ok(())
        } else {
            Err(format!("{}: Token expected, found {}", context, ast_item).into())
        }
    }

    /// Semantic action for production 41:
    ///
    /// String: "\u{0022}([^\\]|\\.)*?\u{0022}";
    ///
    fn string_41(
        &mut self,
        string_0: &ParseTreeStackEntry,
        parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()> {
        let context = "string_41";
        let ast_item = string_0.get_ast_type(parse_tree);
        if let ParseTreeType::T(t) = ast_item {
            // Trim double quotes here
            let s = t.symbol.clone().trim_matches('"').to_owned();
            self.ast_stack
                .push(ParolGrammarItem::Fac(Factor::Terminal(s)));
            Ok(())
        } else {
            Err(format!("{}: Token expected, found {}", context, ast_item).into())
        }
    }
}
