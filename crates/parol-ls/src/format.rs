use std::collections::VecDeque;

use lsp_types::{FormattingOptions, FormattingProperty, TextEdit};
use once_cell::sync::Lazy;

use crate::{
    parol_ls_grammar::OwnedToken,
    parol_ls_grammar_trait::{
        ASTControl, Alternation, AlternationList, Alternations, AlternationsList, CutOperator,
        Declaration, DoubleColon, Factor, GrammarDefinition, GrammarDefinitionList, Group,
        Identifier, LiteralString, NonTerminal, NonTerminalOpt, Optional, ParolLs, Production,
        ProductionLHS, Prolog, PrologList, PrologList0, Regex, Repeat, ScannerDirectives,
        ScannerState, ScannerStateList, ScannerSwitch, ScannerSwitchOpt, SimpleToken,
        SimpleTokenOpt, StartDeclaration, StateList, StateListList, Symbol, TokenLiteral,
        TokenWithStates, TokenWithStatesOpt, UserTypeDeclaration, UserTypeName, UserTypeNameList,
    },
    rng::Rng,
    utils::RX_NEW_LINE,
};

// This is the actual start column for each production (alternation) line
const START_LINE_OFFSET: usize = 6;

pub(crate) static RX_NEW_LINES_AFTER_LINE_COMMENT: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"//.*(\r?\n)+$")
        .expect("error parsing regex: RX_NEW_LINES_AFTER_LINE_COMMENT")
});

#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[allow(dead_code)]
enum Padding {
    #[default]
    None,
    Left,
    Right,
    Both,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[allow(dead_code)]
enum LineEnd {
    #[default]
    Unchanged,
    ForceAdd,
    ForceSingleNewline,
    ForceRemove,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[allow(dead_code)]
enum Trimming {
    #[default]
    Unchanged,
    TrimLeft,
    TrimRight,
    Trim,
}

#[allow(unused)]
#[derive(Debug, Clone, Default)]
struct FmtOptions {
    padding: Padding,
    line_end: LineEnd,
    trimming: Trimming,
    nesting_depth: u16,

    /// Add an empty line after each production
    empty_line_after_prod: bool,

    /// Place the semicolon after each production on a new line
    prod_semicolon_on_nl: bool,

    /// Maximum number of characters per line
    max_line_length: usize,
}

#[allow(unused)]
impl FmtOptions {
    fn new() -> Self {
        FmtOptions {
            empty_line_after_prod: true,
            prod_semicolon_on_nl: true,
            max_line_length: 100,
            ..Default::default()
        }
    }
    fn with_padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self
    }
    fn with_line_end(mut self, line_end: LineEnd) -> Self {
        self.line_end = line_end;
        self
    }
    fn with_trimming(mut self, trimming: Trimming) -> Self {
        self.trimming = trimming;
        self
    }
    fn next_depth(mut self) -> Self {
        self.nesting_depth += 1;
        self
    }
}

macro_rules! add_boolean_formatting_option {
    ($self:ident, $options:ident, $option_name:ident, $default:literal) => {
        $self.$option_name = if let Some(&FormattingProperty::Bool(val)) = $options
            .properties
            .get(concat!("formatting.", stringify!($option_name)))
        {
            val
        } else {
            $default
        };
        eprintln!(
            concat!("FmtOptions: ", stringify!($option_name), ": {}"),
            $self.$option_name
        );
    };
}

macro_rules! add_number_formatting_option {
    ($self:ident, $options:ident, $option_name:ident, $default:literal) => {
        $self.$option_name = if let Some(&FormattingProperty::Number(val)) = $options
            .properties
            .get(concat!("formatting.", stringify!($option_name)))
        {
            val as usize
        } else {
            $default
        };
        eprintln!(
            concat!("FmtOptions: ", stringify!($option_name), ": {}"),
            $self.$option_name
        );
    };
}

struct Line;

impl Line {
    fn ends_with_space(line: &str) -> bool {
        line.ends_with(' ')
    }

    fn ends_with_nl(line: &str) -> bool {
        line.ends_with(|c| c == '\n' || c == '\r')
    }

    fn ends_with_nls_after_line_comment(line: &str) -> bool {
        RX_NEW_LINES_AFTER_LINE_COMMENT.is_match(line)
    }
}

struct Comments;

impl Comments {
    fn split_comments(
        mut comments: VecDeque<OwnedToken>,
        at: Rng,
    ) -> (VecDeque<OwnedToken>, VecDeque<OwnedToken>) {
        let mut left = VecDeque::new();
        while let Some(comment) = comments.front() {
            let rng = Into::<Rng>::into(comment);
            if rng.comes_before(&at) {
                left.push_back(comments.pop_front().unwrap());
            } else {
                break;
            }
        }
        (left, comments)
    }

    fn handle_comments_before(
        comments: VecDeque<OwnedToken>,
        at: &OwnedToken,
        options: &FmtOptions,
    ) -> (String, VecDeque<OwnedToken>) {
        let (comments_before_token, comments) = Comments::split_comments(comments, at.into());
        let comments_before_scanner_str = handle_comments(options, comments_before_token);
        (comments_before_scanner_str, comments)
    }
}

impl From<&FormattingOptions> for FmtOptions {
    fn from(options: &FormattingOptions) -> Self {
        let mut me = Self::new();
        add_boolean_formatting_option!(me, options, empty_line_after_prod, true);
        add_boolean_formatting_option!(me, options, prod_semicolon_on_nl, true);
        add_number_formatting_option!(me, options, max_line_length, 100);
        me
    }
}

pub(crate) trait Format {
    fn format(&self, options: &FormattingOptions, comments: VecDeque<OwnedToken>) -> Vec<TextEdit>;
}

impl Format for &ParolLs {
    fn format(&self, options: &FormattingOptions, comments: VecDeque<OwnedToken>) -> Vec<TextEdit> {
        let range = <&ParolLs as Into<Rng>>::into(*self).0;
        let fmt_options = options.into();
        let (new_text, comments) = self.txt(&fmt_options, comments);
        debug_assert!(comments.is_empty());
        vec![TextEdit { range, new_text }]
    }
}

trait Fmt {
    fn txt(
        &self,
        options: &FmtOptions,
        comments: VecDeque<OwnedToken>,
    ) -> (String, VecDeque<OwnedToken>);
}

impl Fmt for ASTControl {
    fn txt(
        &self,
        options: &FmtOptions,
        comments: VecDeque<OwnedToken>,
    ) -> (String, VecDeque<OwnedToken>) {
        match self {
            ASTControl::CutOperator(_) => ("^".to_string(), comments),
            ASTControl::UserTypeDeclaration(ut) => ut.user_type_declaration.txt(options, comments),
        }
    }
}
impl Fmt for Alternation {
    fn txt(
        &self,
        options: &FmtOptions,
        comments: VecDeque<OwnedToken>,
    ) -> (String, VecDeque<OwnedToken>) {
        let next_option = options.clone().next_depth();
        let (mut alternation_str, comments) = self.alternation_list.iter().fold(
            (String::new(), comments),
            |(mut acc, comments), e| {
                let (mut next_part, comments) = e.txt(&next_option, comments);
                if options.nesting_depth == 0 {
                    // We do the line length control only at top level (i.e. at production level)
                    let lines: Vec<&str> = RX_NEW_LINE.split(&acc).collect();
                    if lines.len() > 1 && lines.last().unwrap().is_empty() {
                        acc.push_str("      ");
                    } else if lines.len() > 1 {
                        if lines.last().unwrap().len() + next_part.len() > options.max_line_length {
                            acc.push_str("\n      ");
                        }
                    } else if START_LINE_OFFSET + acc.len() + next_part.len()
                        > options.max_line_length
                    {
                        acc.push_str("\n      ");
                    }
                }
                if !acc.is_empty() && !Line::ends_with_nl(&acc) && !Line::ends_with_space(&acc) {
                    acc.push(' ');
                }
                acc.extend(next_part.drain(..));
                (acc, comments)
            },
        );
        if options.nesting_depth == 0 && !Line::ends_with_nl(&alternation_str) {
            alternation_str.push('\n');
        }
        (alternation_str, comments)
    }
}
impl Fmt for AlternationList {
    fn txt(
        &self,
        options: &FmtOptions,
        comments: VecDeque<OwnedToken>,
    ) -> (String, VecDeque<OwnedToken>) {
        self.factor.txt(options, comments)
    }
}
impl Fmt for Alternations {
    fn txt(
        &self,
        options: &FmtOptions,
        comments: VecDeque<OwnedToken>,
    ) -> (String, VecDeque<OwnedToken>) {
        let (alternation_str, comments) = self.alternation.txt(options, comments);
        let delimiter = if options.nesting_depth == 0
            || Line::ends_with_nls_after_line_comment(&alternation_str)
        {
            ""
        } else {
            " "
        };
        let (str, comments) = self.alternations_list.iter().fold(
            (String::new(), comments),
            |(mut acc, comments), a| {
                let (alternations_str, comments) = a.txt(options, comments);
                acc.push_str(&alternations_str);
                (acc, comments)
            },
        );

        (format!("{}{}{}", alternation_str, delimiter, str), comments)
    }
}
impl Fmt for AlternationsList {
    fn txt(
        &self,
        options: &FmtOptions,
        comments: VecDeque<OwnedToken>,
    ) -> (String, VecDeque<OwnedToken>) {
        let (alternation_str, comments) = self.alternation.txt(options, comments);
        let right_padding = if options.nesting_depth == 0 {
            // Normally we add a newline after each top level alternation except the newline is
            // already attached (usually due to line comment handling)
            if Line::ends_with_nls_after_line_comment(&alternation_str) {
                ""
            } else if !Line::ends_with_nl(&alternation_str) {
                "\n"
            } else {
                ""
            }
        } else {
            // In levels other than the top level we concat alternations with a single whitespace
            if Line::ends_with_space(&alternation_str) {
                ""
            } else {
                " "
            }
        };
        let left_padding = if options.nesting_depth == 0 {
            "    "
        } else {
            ""
        };
        (
            format!(
                "{}{} {}{}",
                left_padding, self.or, alternation_str, right_padding,
            ),
            comments,
        )
    }
}
impl Fmt for CutOperator {
    fn txt(
        &self,
        _options: &FmtOptions,
        comments: VecDeque<OwnedToken>,
    ) -> (String, VecDeque<OwnedToken>) {
        (self.cut_operator.text().to_string(), comments)
    }
}
impl Fmt for Declaration {
    fn txt(
        &self,
        options: &FmtOptions,
        comments: VecDeque<OwnedToken>,
    ) -> (String, VecDeque<OwnedToken>) {
        match self {
            Declaration::PercentTitleString(title) => {
                let (str, comments) = title.string.txt(options, comments);
                (format!("\n{} {}", title.percent_title, str), comments)
            }
            Declaration::PercentCommentString(comment) => {
                let (str, comments) = comment.string.txt(options, comments);
                (format!("\n{} {}", comment.percent_comment, str), comments)
            }
            Declaration::PercentUserUnderscoreTypeIdentifierEquUserTypeName(user_type) => {
                let (str1, comments) = user_type.identifier.txt(options, comments);
                let (str2, comments) = user_type.user_type_name.txt(options, comments);
                (
                    format!(
                        "\n{} {} {} {}",
                        user_type.percent_user_underscore_type, str1, user_type.equ, str2,
                    ),
                    comments,
                )
            }
            Declaration::ScannerDirectives(scanner_directives) => {
                handle_scanner_directives(&scanner_directives.scanner_directives, options, comments)
            }
        }
    }
}

impl Fmt for DoubleColon {
    fn txt(
        &self,
        _options: &FmtOptions,
        comments: VecDeque<OwnedToken>,
    ) -> (String, VecDeque<OwnedToken>) {
        (self.double_colon.text().to_string(), comments)
    }
}
impl Fmt for Factor {
    fn txt(
        &self,
        options: &FmtOptions,
        comments: VecDeque<OwnedToken>,
    ) -> (String, VecDeque<OwnedToken>) {
        let next_depth_option = options.clone().next_depth();
        match self {
            Factor::Group(g) => g.group.txt(&next_depth_option, comments),
            Factor::Repeat(r) => r.repeat.txt(&next_depth_option, comments),
            Factor::Optional(o) => o.optional.txt(&next_depth_option, comments),
            Factor::Symbol(s) => handle_symbol(&s.symbol, options, comments),
        }
    }
}
impl Fmt for GrammarDefinition {
    fn txt(
        &self,
        options: &FmtOptions,
        comments: VecDeque<OwnedToken>,
    ) -> (String, VecDeque<OwnedToken>) {
        let (production_str, comments) = self.production.txt(options, comments);
        let (grammar_definition_list_str, comments) = self.grammar_definition_list.iter().fold(
            (String::new(), comments),
            |(mut acc, comments), p| {
                let (str, comments) = p.txt(options, comments);
                acc.push_str(&str);
                (acc, comments)
            },
        );

        (
            format!(
                "\n{}\n{}{}",
                self.percent_percent, production_str, grammar_definition_list_str
            ),
            comments,
        )
    }
}
impl Fmt for GrammarDefinitionList {
    fn txt(
        &self,
        options: &FmtOptions,
        comments: VecDeque<OwnedToken>,
    ) -> (String, VecDeque<OwnedToken>) {
        self.production.txt(options, comments)
    }
}
impl Fmt for Group {
    fn txt(
        &self,
        options: &FmtOptions,
        comments: VecDeque<OwnedToken>,
    ) -> (String, VecDeque<OwnedToken>) {
        let (alternations_str, comments) = self.alternations.txt(options, comments);
        let sep = if Line::ends_with_space(&alternations_str) {
            ""
        } else {
            " "
        };
        (
            format!(
                "{} {}{}{}",
                self.l_paren, alternations_str, sep, self.r_paren,
            ),
            comments,
        )
    }
}

impl Fmt for Identifier {
    fn txt(
        &self,
        _options: &FmtOptions,
        comments: VecDeque<OwnedToken>,
    ) -> (String, VecDeque<OwnedToken>) {
        (self.identifier.text().to_string(), comments)
    }
}
impl Fmt for LiteralString {
    fn txt(
        &self,
        _options: &FmtOptions,
        comments: VecDeque<OwnedToken>,
    ) -> (String, VecDeque<OwnedToken>) {
        (self.literal_string.text().to_string(), comments)
    }
}
impl Fmt for NonTerminal {
    fn txt(
        &self,
        options: &FmtOptions,
        comments: VecDeque<OwnedToken>,
    ) -> (String, VecDeque<OwnedToken>) {
        let (str, comments) = if let Some(non_terminal_opt) = self.non_terminal_opt.as_ref() {
            non_terminal_opt.txt(options, comments)
        } else {
            (String::default(), comments)
        };
        (format!("{}{}", self.identifier.identifier, str), comments)
    }
}
impl Fmt for NonTerminalOpt {
    fn txt(
        &self,
        options: &FmtOptions,
        comments: VecDeque<OwnedToken>,
    ) -> (String, VecDeque<OwnedToken>) {
        self.a_s_t_control.txt(options, comments)
    }
}
impl Fmt for Optional {
    fn txt(
        &self,
        options: &FmtOptions,
        comments: VecDeque<OwnedToken>,
    ) -> (String, VecDeque<OwnedToken>) {
        let (alternations_str, comments) = self.alternations.txt(options, comments);
        let sep = if Line::ends_with_space(&alternations_str) {
            ""
        } else {
            " "
        };
        (
            format!(
                "{} {}{}{}",
                self.l_bracket, alternations_str, sep, self.r_bracket,
            ),
            comments,
        )
    }
}
impl Fmt for ParolLs {
    fn txt(
        &self,
        options: &FmtOptions,
        comments: VecDeque<OwnedToken>,
    ) -> (String, VecDeque<OwnedToken>) {
        let (prolog, comments) = self.prolog.txt(options, comments);
        let nl_opt = if Line::ends_with_nl(&prolog) {
            ""
        } else {
            "\n"
        };
        let (grammar_definition, comments) = self.grammar_definition.txt(options, comments);
        (
            format!("{}{}{}", prolog, nl_opt, grammar_definition),
            comments,
        )
    }
}
impl Fmt for Production {
    fn txt(
        &self,
        options: &FmtOptions,
        comments: VecDeque<OwnedToken>,
    ) -> (String, VecDeque<OwnedToken>) {
        let (production_l_h_s, comments) = self.production_l_h_s.txt(options, comments);
        let (mut alternations_str, comments) = self.alternations.txt(options, comments);
        let (semi_nl_opt, alternations_str) = if options.prod_semicolon_on_nl
            || Line::ends_with_nls_after_line_comment(&alternations_str)
        {
            ("\n    ", alternations_str.trim().to_owned())
        } else {
            if Line::ends_with_nls_after_line_comment(&alternations_str) {
                // This indicates a line comment at the end of the alternation.
                // We correct the formatting here to have the semicolon correctly indented.
                alternations_str.push_str("    ");
            } else {
                alternations_str = alternations_str.trim_end().to_owned();
            }
            ("", alternations_str)
        };

        let prod_nl_opt = if options.empty_line_after_prod {
            "\n"
        } else {
            ""
        };
        let (comments_before_semicolon, comments) = Comments::handle_comments_before(
            comments,
            &self.semicolon,
            &options.clone().with_padding(Padding::Left),
        );
        (
            format!(
                "{} {}{}{}{}{}",
                production_l_h_s,
                alternations_str,
                comments_before_semicolon,
                semi_nl_opt,
                self.semicolon,
                prod_nl_opt,
            ),
            comments,
        )
    }
}
impl Fmt for ProductionLHS {
    fn txt(
        &self,
        options: &FmtOptions,
        comments: VecDeque<OwnedToken>,
    ) -> (String, VecDeque<OwnedToken>) {
        let (comments_before_non_terminal, comments) = Comments::handle_comments_before(
            comments,
            &self.identifier.identifier,
            &options.clone().with_line_end(LineEnd::ForceSingleNewline),
        );
        let (comments_before_colon, comments) = Comments::handle_comments_before(
            comments,
            &self.colon,
            &options.clone().with_padding(Padding::Left),
        );
        if self.identifier.identifier.text().len() + comments_before_colon.len() < 5 {
            let padding = " ".repeat(4 - self.identifier.identifier.text().len());
            (
                format!(
                    "\n{}{}{}{}{}",
                    comments_before_non_terminal,
                    self.identifier.identifier,
                    padding,
                    comments_before_colon,
                    self.colon
                ),
                comments,
            )
        } else {
            (
                format!(
                    "\n{}{}{}\n    {}",
                    comments_before_non_terminal,
                    self.identifier.identifier,
                    comments_before_colon,
                    self.colon
                ),
                comments,
            )
        }
    }
}
impl Fmt for Prolog {
    fn txt(
        &self,
        options: &FmtOptions,
        comments: VecDeque<OwnedToken>,
    ) -> (String, VecDeque<OwnedToken>) {
        let (start_declaration, comments) = self.start_declaration.txt(options, comments);
        let (prolog_list, comments) =
            self.prolog_list
                .iter()
                .fold((String::new(), comments), |(mut acc, comments), p| {
                    let (pro_str, comments) = p.txt(options, comments);
                    acc.push_str(&pro_str);
                    (acc, comments)
                });
        let (prolog_list0, comments) =
            self.prolog_list0
                .iter()
                .fold((String::new(), comments), |(mut acc, comments), p| {
                    let (pro_str, comments) = p.txt(options, comments);
                    acc.push_str(&pro_str);
                    (acc, comments)
                });
        (
            format!("{}{}\n{}", start_declaration, prolog_list, prolog_list0),
            comments,
        )
    }
}
impl Fmt for PrologList {
    fn txt(
        &self,
        options: &FmtOptions,
        comments: VecDeque<OwnedToken>,
    ) -> (String, VecDeque<OwnedToken>) {
        self.declaration.txt(options, comments)
    }
}
impl Fmt for PrologList0 {
    fn txt(
        &self,
        options: &FmtOptions,
        comments: VecDeque<OwnedToken>,
    ) -> (String, VecDeque<OwnedToken>) {
        self.scanner_state.txt(options, comments)
    }
}
impl Fmt for Regex {
    fn txt(
        &self,
        _options: &FmtOptions,
        comments: VecDeque<OwnedToken>,
    ) -> (String, VecDeque<OwnedToken>) {
        (self.regex.text().to_string(), comments)
    }
}
impl Fmt for Repeat {
    fn txt(
        &self,
        options: &FmtOptions,
        comments: VecDeque<OwnedToken>,
    ) -> (String, VecDeque<OwnedToken>) {
        let (alternations_str, comments) = self.alternations.txt(options, comments);
        let sep = if Line::ends_with_space(&alternations_str) {
            ""
        } else {
            " "
        };
        (
            format!(
                "{} {}{}{}",
                self.l_brace, alternations_str, sep, self.r_brace,
            ),
            comments,
        )
    }
}
impl Fmt for ScannerDirectives {
    fn txt(
        &self,
        options: &FmtOptions,
        comments: VecDeque<OwnedToken>,
    ) -> (String, VecDeque<OwnedToken>) {
        handle_scanner_directives(self, options, comments)
    }
}
impl Fmt for ScannerState {
    fn txt(
        &self,
        options: &FmtOptions,
        comments: VecDeque<OwnedToken>,
    ) -> (String, VecDeque<OwnedToken>) {
        let nl_before_closing_brace = if self.scanner_state_list.is_empty() {
            ""
        } else {
            "\n"
        };
        let (identifier, comments) = self.identifier.txt(options, comments);
        let inner_options = options.clone().next_depth();
        let (scanner_state_list, comments) = self.scanner_state_list.iter().fold(
            (String::new(), comments),
            |(mut acc, comments), s| {
                let (s_str, comments) = s.txt(&inner_options, comments);
                acc.push_str(&s_str);
                (acc, comments)
            },
        );

        let (comments_before_scanner, comments) = Comments::handle_comments_before(
            comments,
            &self.percent_scanner,
            &options.clone().with_line_end(LineEnd::ForceSingleNewline),
        );
        (
            format!(
                "{}\n{} {} {}{}{}{}",
                comments_before_scanner,
                self.percent_scanner,
                identifier,
                self.l_brace,
                scanner_state_list,
                nl_before_closing_brace,
                self.r_brace,
            ),
            comments,
        )
    }
}
impl Fmt for ScannerStateList {
    fn txt(
        &self,
        options: &FmtOptions,
        comments: VecDeque<OwnedToken>,
    ) -> (String, VecDeque<OwnedToken>) {
        handle_scanner_directives(&self.scanner_directives, options, comments)
    }
}
impl Fmt for ScannerSwitch {
    fn txt(
        &self,
        options: &FmtOptions,
        comments: VecDeque<OwnedToken>,
    ) -> (String, VecDeque<OwnedToken>) {
        match self {
            ScannerSwitch::PercentScLParenScannerSwitchOptRParen(sc) => {
                let (scanner_switch_opt, comments) =
                    if let Some(scanner_switch_opt) = sc.scanner_switch_opt.as_ref() {
                        scanner_switch_opt.txt(options, comments)
                    } else {
                        (String::default(), comments)
                    };
                (
                    format!(
                        "{}{}{}{}",
                        sc.percent_sc, sc.l_paren, scanner_switch_opt, sc.r_paren,
                    ),
                    comments,
                )
            }
            ScannerSwitch::PercentPushLParenIdentifierRParen(push) => {
                let (identifier, comments) = push.identifier.txt(options, comments);
                (
                    format!(
                        "{}{}{}{}",
                        push.percent_push, push.l_paren, identifier, push.r_paren,
                    ),
                    comments,
                )
            }
            ScannerSwitch::PercentPopLParenRParen(pop) => (
                format!("{}{}{}", pop.percent_pop, pop.l_paren, pop.r_paren,),
                comments,
            ),
        }
    }
}
impl Fmt for ScannerSwitchOpt {
    fn txt(
        &self,
        options: &FmtOptions,
        comments: VecDeque<OwnedToken>,
    ) -> (String, VecDeque<OwnedToken>) {
        self.identifier.txt(options, comments)
    }
}
impl Fmt for SimpleToken {
    fn txt(
        &self,
        options: &FmtOptions,
        comments: VecDeque<OwnedToken>,
    ) -> (String, VecDeque<OwnedToken>) {
        let (token_literal, comments) = self.token_literal.txt(options, comments);
        let (simple_token_opt, comments) =
            if let Some(simple_token_opt) = self.simple_token_opt.as_ref() {
                simple_token_opt.txt(options, comments)
            } else {
                (String::default(), comments)
            };
        (format!("{}{}", token_literal, simple_token_opt), comments)
    }
}
impl Fmt for SimpleTokenOpt {
    fn txt(
        &self,
        options: &FmtOptions,
        comments: VecDeque<OwnedToken>,
    ) -> (String, VecDeque<OwnedToken>) {
        self.a_s_t_control.txt(options, comments)
    }
}
impl Fmt for StartDeclaration {
    fn txt(
        &self,
        options: &FmtOptions,
        comments: VecDeque<OwnedToken>,
    ) -> (String, VecDeque<OwnedToken>) {
        let (comments_before_start, comments) = Comments::handle_comments_before(
            comments,
            &self.percent_start,
            &options.clone().with_line_end(LineEnd::ForceSingleNewline),
        );

        let (identifier, comments) = self.identifier.txt(options, comments);
        (
            format!(
                "{}{} {}",
                comments_before_start, self.percent_start, identifier,
            ),
            comments,
        )
    }
}
impl Fmt for StateList {
    fn txt(
        &self,
        options: &FmtOptions,
        comments: VecDeque<OwnedToken>,
    ) -> (String, VecDeque<OwnedToken>) {
        let (identifier, comments) = self.identifier.txt(options, comments);
        let (state_list_list, comments) = self.state_list_list.iter().fold(
            (String::new(), comments),
            |(mut acc, comments), s| {
                let (s_str, comments) = s.txt(options, comments);
                acc.push_str(&s_str);
                (acc, comments)
            },
        );
        (format!("{}{}", identifier, state_list_list,), comments)
    }
}
impl Fmt for StateListList {
    fn txt(
        &self,
        options: &FmtOptions,
        comments: VecDeque<OwnedToken>,
    ) -> (String, VecDeque<OwnedToken>) {
        let (identifier, comments) = self.identifier.txt(options, comments);
        (format!("{} {}", self.comma, identifier), comments)
    }
}
impl Fmt for crate::parol_ls_grammar_trait::String {
    fn txt(
        &self,
        options: &FmtOptions,
        comments: VecDeque<OwnedToken>,
    ) -> (String, VecDeque<OwnedToken>) {
        let (comments_before_string, comments) = Comments::handle_comments_before(
            comments,
            &self.string,
            &options.clone().with_padding(Padding::Right),
        );
        (
            format!(
                "{}{}",
                comments_before_string,
                self.string.text().to_string()
            ),
            comments,
        )
    }
}
impl Fmt for Symbol {
    fn txt(
        &self,
        options: &FmtOptions,
        comments: VecDeque<OwnedToken>,
    ) -> (String, VecDeque<OwnedToken>) {
        handle_symbol(self, options, comments)
    }
}
impl Fmt for TokenLiteral {
    fn txt(
        &self,
        options: &FmtOptions,
        comments: VecDeque<OwnedToken>,
    ) -> (String, VecDeque<OwnedToken>) {
        match self {
            TokenLiteral::String(s) => s.string.txt(options, comments),
            TokenLiteral::LiteralString(l) => l.literal_string.txt(options, comments),
            TokenLiteral::Regex(r) => r.regex.txt(options, comments),
        }
    }
}
impl Fmt for TokenWithStates {
    fn txt(
        &self,
        options: &FmtOptions,
        comments: VecDeque<OwnedToken>,
    ) -> (String, VecDeque<OwnedToken>) {
        let (mut state_list, comments) = self.state_list.txt(options, comments);
        let (token_literal, comments) = self.token_literal.txt(options, comments);
        let (token_with_states_opt, comments) =
            if let Some(token_with_states_opt) = self.token_with_states_opt.as_ref() {
                token_with_states_opt.txt(options, comments)
            } else {
                (String::default(), comments)
            };
        state_list = state_list.trim().to_owned();
        (
            format!(
                "{}{}{}{}{}",
                self.l_t, state_list, self.g_t, token_literal, token_with_states_opt
            ),
            comments,
        )
    }
}
impl Fmt for TokenWithStatesOpt {
    fn txt(
        &self,
        options: &FmtOptions,
        comments: VecDeque<OwnedToken>,
    ) -> (String, VecDeque<OwnedToken>) {
        self.a_s_t_control.txt(options, comments)
    }
}
impl Fmt for UserTypeDeclaration {
    fn txt(
        &self,
        options: &FmtOptions,
        comments: VecDeque<OwnedToken>,
    ) -> (String, VecDeque<OwnedToken>) {
        let (user_type_name, comments) = self.user_type_name.txt(options, comments);
        (format!("{} {}", self.colon, user_type_name), comments)
    }
}
impl Fmt for UserTypeName {
    fn txt(
        &self,
        options: &FmtOptions,
        comments: VecDeque<OwnedToken>,
    ) -> (String, VecDeque<OwnedToken>) {
        let (identifier, comments) = self.identifier.txt(options, comments);
        let (user_type_name_list, comments) = self.user_type_name_list.iter().fold(
            (String::new(), comments),
            |(mut acc, comments), u| {
                let (u_str, comments) = u.txt(options, comments);
                acc.push_str(&u_str);
                (acc, comments)
            },
        );
        (format!("{}{}", identifier, user_type_name_list,), comments)
    }
}
impl Fmt for UserTypeNameList {
    fn txt(
        &self,
        options: &FmtOptions,
        comments: VecDeque<OwnedToken>,
    ) -> (String, VecDeque<OwnedToken>) {
        let (identifier, comments) = self.identifier.txt(options, comments);
        (
            format!("{}{}", self.double_colon.double_colon, identifier,),
            comments,
        )
    }
}

fn handle_scanner_directives(
    scanner_directives: &ScannerDirectives,
    options: &FmtOptions,
    comments: VecDeque<OwnedToken>,
) -> (String, VecDeque<OwnedToken>) {
    let indent = make_indent(options.nesting_depth);
    match scanner_directives {
        ScannerDirectives::PercentLineUnderscoreCommentTokenLiteral(l) => {
            let (str, comments) = l.token_literal.txt(options, comments);
            (
                format!("\n{}{} {}", indent, l.percent_line_underscore_comment, str,),
                comments,
            )
        }
        ScannerDirectives::PercentBlockUnderscoreCommentTokenLiteralTokenLiteral(b) => {
            let (str1, comments) = b.token_literal.txt(options, comments);
            let (str2, comments) = b.token_literal0.txt(options, comments);
            (
                format!(
                    "\n{}{} {} {}",
                    indent, b.percent_block_underscore_comment, str1, str2,
                ),
                comments,
            )
        }

        ScannerDirectives::PercentAutoUnderscoreNewlineUnderscoreOff(n) => (
            format!(
                "\n{}{}",
                indent, n.percent_auto_underscore_newline_underscore_off
            ),
            comments,
        ),

        ScannerDirectives::PercentAutoUnderscoreWsUnderscoreOff(w) => (
            format!(
                "\n{}{}",
                indent, w.percent_auto_underscore_ws_underscore_off
            ),
            comments,
        ),
    }
}

fn handle_symbol(
    symbol: &Symbol,
    options: &FmtOptions,
    comments: VecDeque<OwnedToken>,
) -> (String, VecDeque<OwnedToken>) {
    match symbol {
        Symbol::NonTerminal(n) => n.non_terminal.txt(options, comments),
        Symbol::SimpleToken(t) => t.simple_token.txt(options, comments),
        Symbol::TokenWithStates(t) => t.token_with_states.txt(options, comments),
        Symbol::ScannerSwitch(s) => s.scanner_switch.txt(options, comments),
    }
}

fn handle_comments(options: &FmtOptions, comments: VecDeque<OwnedToken>) -> String {
    let comments_str = comments.iter().fold(String::new(), |mut acc, c| {
        acc.push_str(c.text());
        acc
    });
    if comments_str.is_empty() {
        comments_str
    } else {
        let options = if let Some(cmt) = comments.iter().last() {
            if cmt.text().starts_with("//") {
                options.clone().with_line_end(LineEnd::ForceSingleNewline)
            } else {
                options.clone()
            }
        } else {
            options.clone()
        };
        apply_formatting(comments_str, &options)
    }
}

fn apply_formatting(line: String, options: &FmtOptions) -> String {
    let line = match options.trimming {
        Trimming::Unchanged => line,
        Trimming::TrimLeft => line.trim_start().to_string(),
        Trimming::TrimRight => line.trim_end().to_string(),
        Trimming::Trim => line.trim().to_string(),
    };
    let line = match options.line_end {
        LineEnd::Unchanged => line,
        LineEnd::ForceAdd => {
            if line.is_empty() {
                line
            } else {
                line + "\n"
            }
        }
        LineEnd::ForceRemove => line
            .trim_end_matches(|c| c == '\r' || c == '\n')
            .to_string(),
        LineEnd::ForceSingleNewline => {
            let mut trimmed = line.trim_matches(|c| c == '\r' || c == '\n').to_string();
            trimmed.push('\n');
            trimmed
        }
    };
    if !line.is_empty() {
        match options.padding {
            Padding::None => line,
            Padding::Left => format!(" {}", line),
            Padding::Right => format!("{} ", line),
            Padding::Both => format!(" {} ", line),
        }
    } else {
        line
    }
}

#[allow(unused)]
fn make_indent(depth: u16) -> String {
    let mut indent = String::with_capacity((depth as usize) * 4);
    indent.extend("    ".repeat(depth as usize).drain(..));
    indent
}

#[cfg(test)]
mod test {
    use std::{ffi::OsStr, fs};

    use parol_runtime::Report;

    use crate::{
        format::{make_indent, Fmt, FmtOptions},
        parol_ls_grammar::ParolLsGrammar,
        parol_ls_parser::parse,
        utils::RX_NEW_LINE,
    };

    struct LsErrorReporter;
    impl Report for LsErrorReporter {}

    const INPUT_FOLDER: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/data/input");
    const ACTUAL_FOLDER: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/data/actual");

    // Use this to skip certain tests if they are not ready yet
    const SKIP_LIST: &[&str] = &[]; //&["complex1.par"];

    // Use this if you only want to debug a view tests
    const SELECTED_TESTS: &[&str] = &[]; //&["single_group.par"];

    const TEST_DATA: &[(FmtOptions, &str)] = &[
        (
            FmtOptions {
                empty_line_after_prod: true,
                prod_semicolon_on_nl: true,
                max_line_length: 100,
                padding: super::Padding::None,
                line_end: super::LineEnd::Unchanged,
                trimming: super::Trimming::Unchanged,
                nesting_depth: 0,
            },
            concat!(env!("CARGO_MANIFEST_DIR"), "/data/expected/options_default"),
        ),
        // (
        //     FmtOptions {
        //         empty_line_after_prod: true,
        //         prod_semicolon_on_nl: false,
        //         max_line_length: 100,
        //         padding: super::Padding::None,
        //         line_end: super::LineEnd::Unchanged,
        //         trimming: super::Trimming::Unchanged,
        //         nesting_depth: 0,
        //     },
        //     concat!(
        //         env!("CARGO_MANIFEST_DIR"),
        //         "/data/expected/prod_semicolon_on_nl_false"
        //     ),
        // ),
    ];

    #[test]
    fn test_make_indent() {
        assert_eq!(String::from(""), make_indent(0));
        let options = FmtOptions::new();
        assert_eq!(String::from(""), make_indent(options.nesting_depth));
        assert_eq!(
            String::from("    "),
            make_indent(options.next_depth().nesting_depth)
        );
    }

    #[test]
    fn test_formatting() {
        let mut error_count = 0;
        let mut tests_run = 0;

        for (fmt_options, expected_folder) in TEST_DATA {
            eprintln!("from folder {INPUT_FOLDER}:");
            for entry in std::path::Path::new(INPUT_FOLDER)
                .read_dir()
                .unwrap()
                .flatten()
            {
                if skip_test(&entry.file_name()) {
                    continue;
                }
                if entry.path().extension().unwrap().to_str().unwrap() == "par" {
                    eprintln!("\nParsing {}...", entry.path().display());
                    if !process_single_file(
                        entry.file_name().as_os_str(),
                        fmt_options,
                        expected_folder,
                    ) {
                        error_count += 1;
                    }
                    tests_run += 1;
                }
            }
        }
        eprintln!("Found {error_count} formatting error(s) in {tests_run} tests.");
        assert_eq!(0, error_count);
    }

    fn process_single_file(
        file_name: &OsStr,
        fmt_options: &FmtOptions,
        expected_folder: &str,
    ) -> bool {
        let mut input_file = std::path::PathBuf::from(INPUT_FOLDER);
        input_file.push(file_name);
        let input_grammar = fs::read_to_string(input_file.clone()).unwrap();
        let mut grammar = ParolLsGrammar::new();

        if let Err(e) = parse(&input_grammar, input_file.clone(), &mut grammar) {
            LsErrorReporter::report_error(&e, input_file).unwrap();
            panic!("Parsing failed!")
        } else {
            // We generate the new formatting by calling Fmt::txt()
            let (formatted_grammar, _comments) = grammar
                .grammar
                .unwrap()
                .txt(fmt_options, grammar.comments.clone());
            // assert!(comments.is_empty());

            let mut expected_file = std::path::PathBuf::from(expected_folder);

            // Only to support debugging we write out the currently generated source
            let mut actual_file = std::path::PathBuf::from(ACTUAL_FOLDER);
            let expected_sub_folder = expected_file.iter().last().unwrap();
            actual_file.push(expected_sub_folder);
            fs::DirBuilder::new()
                .recursive(true)
                .create(actual_file.clone())
                .unwrap();

            actual_file.push(file_name);
            fs::write(actual_file, formatted_grammar.clone()).unwrap();

            // Read the fixed expectation file into a string
            expected_file.push(file_name);
            eprintln!("expected_file: '{}'", expected_file.display());
            let expected_format = fs::read_to_string(expected_file).unwrap();

            // Compare result with expectation
            let expected_format = RX_NEW_LINE.replace_all(&expected_format, "\n");
            let formatted_grammar = RX_NEW_LINE.replace_all(&formatted_grammar, "\n");

            if expected_format != formatted_grammar {
                eprintln!("=====================================================");
                eprintln!("expecting:\n'{expected_format}'");
                eprintln!("-----------------------------------------------------");
                eprintln!("received:\n'{formatted_grammar}'");
                eprintln!("=====================================================");
                false
            } else {
                true
            }
        }
    }

    fn skip_test(file_name: &OsStr) -> bool {
        SKIP_LIST.contains(&file_name.to_str().unwrap())
            || (!SELECTED_TESTS.is_empty()
                && !SELECTED_TESTS.contains(&file_name.to_str().unwrap()))
    }
}
