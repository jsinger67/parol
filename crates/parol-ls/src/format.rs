use lsp_types::{FormattingOptions, FormattingProperty, TextEdit};
use once_cell::sync::Lazy;

use crate::{
    parol_ls_grammar_trait::{
        ASTControl, Alternation, AlternationList, Alternations, AlternationsList, BlockComment,
        Comment, Comments, CommentsList, CutOperator, Declaration, DoubleColon, Factor,
        GrammarDefinition, GrammarDefinitionList, Group, Identifier, LineComment, LiteralString,
        NonTerminal, NonTerminalOpt, Optional, ParolLs, Production, ProductionLHS, Prolog,
        PrologList, PrologList0, Regex, Repeat, ScannerDirectives, ScannerState, ScannerStateList,
        ScannerSwitch, ScannerSwitchOpt, SimpleToken, SimpleTokenOpt, StartDeclaration, StateList,
        StateListList, Symbol, TokenLiteral, TokenWithStates, TokenWithStatesOpt,
        UserTypeDeclaration, UserTypeName, UserTypeNameList,
    },
    rng::Rng,
    utils::RX_NEW_LINE,
};

// This is the actual start column for each production (alternation) line
const START_LINE_OFFSET: usize = 6;

pub(crate) static RX_DOUBLE_NEW_LINE: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"\r?\n\r?\n$").expect("error parsing regex: RX_DOUBLE_NEW_LINE")
});

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
    fn ends_with_double_nl(line: &str) -> bool {
        RX_DOUBLE_NEW_LINE.is_match(line)
    }

    // fn ends_with_nl_wo_line_comment(line: &str) -> bool {
    //     Self::ends_with_nl(line) && !RX_NEW_LINES_AFTER_LINE_COMMENT.is_match(line)
    // }

    fn ends_with_nls_after_line_comment(line: &str) -> bool {
        RX_NEW_LINES_AFTER_LINE_COMMENT.is_match(line)
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
    fn format(&self, options: &FormattingOptions) -> Vec<TextEdit>;
}

impl Format for &ParolLs {
    fn format(&self, options: &FormattingOptions) -> Vec<TextEdit> {
        let range = <&ParolLs as Into<Rng>>::into(*self).0;
        let fmt_options = options.into();
        let new_text = self.txt(&fmt_options);
        vec![TextEdit { range, new_text }]
    }
}

trait Fmt {
    fn txt(&self, options: &FmtOptions) -> String;
}

impl Fmt for ASTControl {
    fn txt(&self, options: &FmtOptions) -> String {
        match self {
            ASTControl::CutOperator(_) => "^".to_string(),
            ASTControl::UserTypeDeclaration(ut) => ut.user_type_declaration.txt(options),
        }
    }
}
impl Fmt for Alternation {
    fn txt(&self, options: &FmtOptions) -> String {
        let next_option = options.clone().next_depth();
        let mut alternation_str = self
            .alternation_list
            .iter()
            .fold(String::new(), |mut acc, e| {
                let mut next_part = e.txt(&next_option);
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
                acc
            });
        if options.nesting_depth == 0 && !Line::ends_with_nl(&alternation_str) {
            alternation_str.push('\n');
        }
        alternation_str
    }
}
impl Fmt for AlternationList {
    fn txt(&self, options: &FmtOptions) -> String {
        let comment_options = options
            .clone()
            .with_line_end(LineEnd::ForceRemove)
            .with_padding(Padding::Left)
            .with_trimming(Trimming::TrimRight);
        format!(
            "{}{}",
            self.factor.txt(options),
            handle_comments(&self.comments, &comment_options)
        )
    }
}
impl Fmt for Alternations {
    fn txt(&self, options: &FmtOptions) -> String {
        let alternation_str = self.alternation.txt(options);
        let delimiter = if options.nesting_depth == 0
            || Line::ends_with_nls_after_line_comment(&alternation_str)
        {
            ""
        } else {
            " "
        };
        format!(
            "{}{}{}",
            alternation_str,
            delimiter,
            self.alternations_list
                .iter()
                .fold(String::new(), |mut acc, a| {
                    let alternations_str = &a.txt(options);
                    acc.push_str(alternations_str);
                    acc
                })
        )
    }
}
impl Fmt for AlternationsList {
    fn txt(&self, options: &FmtOptions) -> String {
        let alternation_str = self.alternation.txt(options);
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
        format!(
            "{}{}{} {}{}",
            left_padding,
            self.or,
            handle_comments(&self.comments, options),
            alternation_str,
            right_padding,
        )
    }
}
impl Fmt for BlockComment {
    fn txt(&self, _options: &FmtOptions) -> String {
        self.block_comment.text().to_string()
    }
}
impl Fmt for Comment {
    fn txt(&self, _options: &FmtOptions) -> String {
        match self {
            Comment::LineComment(l) => l.line_comment.line_comment.text().to_string(),
            Comment::BlockComment(b) => b.block_comment.block_comment.text().to_string(),
        }
    }
}
impl Fmt for CommentsList {
    fn txt(&self, options: &FmtOptions) -> String {
        self.comment.txt(options)
    }
}
impl Fmt for CutOperator {
    fn txt(&self, _options: &FmtOptions) -> String {
        self.cut_operator.text().to_string()
    }
}
impl Fmt for Declaration {
    fn txt(&self, options: &FmtOptions) -> String {
        let comment_options = options
            .clone()
            .with_padding(Padding::Left)
            .with_trimming(Trimming::TrimRight);
        match self {
            Declaration::PercentTitleStringComments(title) => format!(
                "{} {}{}\n",
                title.percent_title,
                title.string.txt(options),
                handle_comments(&title.comments, &comment_options),
            ),
            Declaration::PercentCommentStringComments(comment) => format!(
                "{} {}{}\n",
                comment.percent_comment,
                comment.string.txt(options),
                handle_comments(&comment.comments, &comment_options),
            ),
            Declaration::PercentUserUnderscoreTypeIdentifierEquUserTypeNameComments(user_type) => {
                format!(
                    "{} {} {} {}{}\n",
                    user_type.percent_user_underscore_type,
                    user_type.identifier.txt(options),
                    user_type.equ,
                    user_type.user_type_name.txt(options),
                    handle_comments(&user_type.comments, &comment_options),
                )
            }
            Declaration::ScannerDirectives(scanner_directives) => {
                handle_scanner_directives(&scanner_directives.scanner_directives, options)
            }
        }
    }
}

impl Fmt for DoubleColon {
    fn txt(&self, _options: &FmtOptions) -> String {
        self.double_colon.text().to_string()
    }
}
impl Fmt for Factor {
    fn txt(&self, options: &FmtOptions) -> String {
        let next_depth_option = options.clone().next_depth();
        match self {
            Factor::Group(g) => g.group.txt(&next_depth_option),
            Factor::Repeat(r) => r.repeat.txt(&next_depth_option),
            Factor::Optional(o) => o.optional.txt(&next_depth_option),
            Factor::Symbol(s) => handle_symbol(&s.symbol, options),
        }
    }
}
impl Fmt for GrammarDefinition {
    fn txt(&self, options: &FmtOptions) -> String {
        format!(
            "{}\n{}{}",
            self.percent_percent,
            self.production.txt(options),
            self.grammar_definition_list
                .iter()
                .fold(String::new(), |mut acc, p| {
                    acc.push_str(&p.txt(options));
                    acc
                })
        )
    }
}
impl Fmt for GrammarDefinitionList {
    fn txt(&self, options: &FmtOptions) -> String {
        self.production.txt(options)
    }
}
impl Fmt for Group {
    fn txt(&self, options: &FmtOptions) -> String {
        let alternations_str = self.alternations.txt(options);
        let sep = if Line::ends_with_space(&alternations_str) {
            ""
        } else {
            " "
        };
        format!(
            "{} {}{}{}",
            self.l_paren, alternations_str, sep, self.r_paren,
        )
    }
}

impl Fmt for Identifier {
    fn txt(&self, _options: &FmtOptions) -> String {
        self.identifier.text().to_string()
    }
}
impl Fmt for LineComment {
    fn txt(&self, _options: &FmtOptions) -> String {
        self.line_comment.text().to_string()
    }
}
impl Fmt for LiteralString {
    fn txt(&self, _options: &FmtOptions) -> String {
        self.literal_string.text().to_string()
    }
}
impl Fmt for NonTerminal {
    fn txt(&self, options: &FmtOptions) -> String {
        format!(
            "{}{}",
            self.identifier.identifier,
            self.non_terminal_opt
                .as_ref()
                .map_or(String::default(), |a| { a.txt(options) })
        )
    }
}
impl Fmt for NonTerminalOpt {
    fn txt(&self, options: &FmtOptions) -> String {
        self.a_s_t_control.txt(options)
    }
}
impl Fmt for Optional {
    fn txt(&self, options: &FmtOptions) -> String {
        let alternations_str = self.alternations.txt(options);
        let sep = if Line::ends_with_space(&alternations_str) {
            ""
        } else {
            " "
        };
        format!(
            "{} {}{}{}",
            self.l_bracket, alternations_str, sep, self.r_bracket,
        )
    }
}
impl Fmt for ParolLs {
    fn txt(&self, options: &FmtOptions) -> String {
        let prolog = self.prolog.txt(options);
        let nl_opt = if Line::ends_with_double_nl(&prolog) {
            ""
        } else {
            "\n"
        };
        format!(
            "{}{}{}",
            prolog,
            nl_opt,
            self.grammar_definition.txt(options),
        )
    }
}
impl Fmt for Production {
    fn txt(&self, options: &FmtOptions) -> String {
        let mut alternations_str = self.alternations.txt(options);
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
        format!(
            "{} {}{}{}{}",
            self.production_l_h_s.txt(options),
            alternations_str,
            semi_nl_opt,
            self.semicolon,
            prod_nl_opt,
        )
    }
}
impl Fmt for ProductionLHS {
    fn txt(&self, options: &FmtOptions) -> String {
        let comment_options_both = options
            .clone()
            .with_padding(Padding::Left)
            .with_trimming(Trimming::TrimRight);
        let comment_options_right = options
            .clone()
            .with_line_end(LineEnd::ForceAdd)
            .with_trimming(Trimming::TrimRight);
        if self.identifier.identifier.text().len() < 5 && self.comments0.comments_list.is_empty() {
            let padding = " ".repeat(4 - self.identifier.identifier.text().len());
            format!(
                "\n{}{}{}{}",
                handle_comments(&self.comments, &comment_options_right),
                self.identifier.identifier,
                padding,
                self.colon,
            )
        } else {
            format!(
                "\n{}{}{}\n    {}",
                handle_comments(&self.comments, &comment_options_right),
                self.identifier.identifier,
                handle_comments(&self.comments0, &comment_options_both),
                self.colon,
            )
        }
    }
}
impl Fmt for Prolog {
    fn txt(&self, options: &FmtOptions) -> String {
        format!(
            "{}{}\n{}",
            self.start_declaration.txt(options),
            self.prolog_list.iter().fold(String::new(), |mut acc, p| {
                acc.push_str(&p.txt(options));
                acc
            }),
            self.prolog_list0.iter().fold(String::new(), |mut acc, p| {
                acc.push_str(&p.txt(options));
                acc
            })
        )
    }
}
impl Fmt for PrologList {
    fn txt(&self, options: &FmtOptions) -> String {
        self.declaration.txt(options)
    }
}
impl Fmt for PrologList0 {
    fn txt(&self, options: &FmtOptions) -> String {
        self.scanner_state.txt(options)
    }
}
impl Fmt for Regex {
    fn txt(&self, _options: &FmtOptions) -> String {
        self.regex.text().to_string()
    }
}
impl Fmt for Repeat {
    fn txt(&self, options: &FmtOptions) -> String {
        let alternations_str = self.alternations.txt(options);
        let sep = if Line::ends_with_space(&alternations_str) {
            ""
        } else {
            " "
        };
        format!(
            "{} {}{}{}",
            self.l_brace, alternations_str, sep, self.r_brace,
        )
    }
}
impl Fmt for ScannerDirectives {
    fn txt(&self, options: &FmtOptions) -> String {
        handle_scanner_directives(self, options)
    }
}
impl Fmt for ScannerState {
    fn txt(&self, options: &FmtOptions) -> String {
        format!(
            "\n{} {} {}\n{}{}\n",
            self.percent_scanner,
            self.identifier.txt(options),
            self.l_brace,
            self.scanner_state_list
                .iter()
                .fold(String::new(), |mut acc, s| {
                    acc.push_str("    ");
                    acc.push_str(&s.txt(options));
                    acc
                }),
            self.r_brace,
        )
    }
}
impl Fmt for ScannerStateList {
    fn txt(&self, options: &FmtOptions) -> String {
        handle_scanner_directives(&self.scanner_directives, options)
    }
}
impl Fmt for ScannerSwitch {
    fn txt(&self, options: &FmtOptions) -> String {
        match self {
            ScannerSwitch::PercentScLParenScannerSwitchOptRParen(sc) => format!(
                "{}{}{}{}",
                sc.percent_sc,
                sc.l_paren,
                sc.scanner_switch_opt
                    .as_ref()
                    .map_or(String::default(), |s| { s.txt(options) }),
                sc.r_paren,
            ),
            ScannerSwitch::PercentPushLParenIdentifierRParen(push) => format!(
                "{}{}{}{}",
                push.percent_push,
                push.l_paren,
                push.identifier.txt(options),
                push.r_paren,
            ),
            ScannerSwitch::PercentPopLParenRParen(pop) => {
                format!("{}{}{}", pop.percent_pop, pop.l_paren, pop.r_paren,)
            }
        }
    }
}
impl Fmt for ScannerSwitchOpt {
    fn txt(&self, options: &FmtOptions) -> String {
        self.identifier.txt(options)
    }
}
impl Fmt for SimpleToken {
    fn txt(&self, options: &FmtOptions) -> String {
        format!(
            "{}{}",
            self.token_literal.txt(options),
            self.simple_token_opt
                .as_ref()
                .map_or(String::default(), |s| { s.txt(options) })
        )
    }
}
impl Fmt for SimpleTokenOpt {
    fn txt(&self, options: &FmtOptions) -> String {
        self.a_s_t_control.txt(options)
    }
}
impl Fmt for StartDeclaration {
    fn txt(&self, options: &FmtOptions) -> String {
        let comment_options_left = options.clone().with_padding(Padding::Left);
        let comment_options_right = options.clone().with_padding(Padding::Right);
        format!(
            "{}{} {}{}\n",
            handle_comments(&self.comments, &comment_options_right),
            self.percent_start,
            self.identifier.txt(options),
            handle_comments(&self.comments0, &comment_options_left),
        )
    }
}
impl Fmt for StateList {
    fn txt(&self, options: &FmtOptions) -> String {
        format!(
            "{}{}",
            self.identifier.txt(options),
            self.state_list_list
                .iter()
                .fold(String::new(), |mut acc, s| {
                    acc.push_str(&s.txt(options));
                    acc
                }),
        )
    }
}
impl Fmt for StateListList {
    fn txt(&self, options: &FmtOptions) -> String {
        format!("{} {}", self.comma, self.identifier.txt(options),)
    }
}
impl Fmt for crate::parol_ls_grammar_trait::String {
    fn txt(&self, _options: &FmtOptions) -> String {
        self.string.text().to_string()
    }
}
impl Fmt for Symbol {
    fn txt(&self, options: &FmtOptions) -> String {
        handle_symbol(self, options)
    }
}
impl Fmt for TokenLiteral {
    fn txt(&self, options: &FmtOptions) -> String {
        match self {
            TokenLiteral::String(s) => s.string.txt(options),
            TokenLiteral::LiteralString(l) => l.literal_string.txt(options),
            TokenLiteral::Regex(r) => r.regex.txt(options),
        }
    }
}
impl Fmt for TokenWithStates {
    fn txt(&self, options: &FmtOptions) -> String {
        format!(
            "{}{}{}{}{}",
            self.l_t,
            self.state_list.txt(options).trim(),
            self.g_t,
            self.token_literal.txt(options),
            self.token_with_states_opt
                .as_ref()
                .map_or(String::default(), |a| { a.txt(options) }),
        )
    }
}
impl Fmt for TokenWithStatesOpt {
    fn txt(&self, options: &FmtOptions) -> String {
        self.a_s_t_control.txt(options)
    }
}
impl Fmt for UserTypeDeclaration {
    fn txt(&self, options: &FmtOptions) -> String {
        format!("{} {}", self.colon, self.user_type_name.txt(options),)
    }
}
impl Fmt for UserTypeName {
    fn txt(&self, options: &FmtOptions) -> String {
        format!(
            "{}{}",
            self.identifier.txt(options),
            self.user_type_name_list
                .iter()
                .fold(String::new(), |mut acc, u| {
                    acc.push_str(&u.txt(options));
                    acc
                }),
        )
    }
}
impl Fmt for UserTypeNameList {
    fn txt(&self, options: &FmtOptions) -> String {
        format!(
            "{}{}",
            self.double_colon.double_colon,
            self.identifier.txt(options),
        )
    }
}

fn handle_scanner_directives(
    scanner_directives: &ScannerDirectives,
    options: &FmtOptions,
) -> String {
    let comment_options = options.clone().with_padding(Padding::Left);
    match scanner_directives {
        ScannerDirectives::PercentLineUnderscoreCommentTokenLiteralComments(l) => format!(
            "{} {}{}\n",
            l.percent_line_underscore_comment,
            l.token_literal.txt(options),
            handle_comments(&l.comments, &comment_options),
        ),
        ScannerDirectives::PercentBlockUnderscoreCommentTokenLiteralTokenLiteralComments(b) => {
            format!(
                "{} {} {}{}\n",
                b.percent_block_underscore_comment,
                b.token_literal.txt(options),
                b.token_literal0.txt(options),
                handle_comments(&b.comments, &comment_options),
            )
        }

        ScannerDirectives::PercentAutoUnderscoreNewlineUnderscoreOffComments(n) => format!(
            "{}{}\n",
            n.percent_auto_underscore_newline_underscore_off,
            handle_comments(&n.comments, &comment_options),
        ),

        ScannerDirectives::PercentAutoUnderscoreWsUnderscoreOffComments(w) => format!(
            "{}{}\n",
            w.percent_auto_underscore_ws_underscore_off,
            handle_comments(&w.comments, &comment_options),
        ),
    }
}

fn handle_symbol(symbol: &Symbol, options: &FmtOptions) -> String {
    match symbol {
        Symbol::NonTerminal(n) => n.non_terminal.txt(options),
        Symbol::SimpleToken(t) => t.simple_token.txt(options),
        Symbol::TokenWithStates(t) => t.token_with_states.txt(options),
        Symbol::ScannerSwitch(s) => s.scanner_switch.txt(options),
    }
}

fn handle_comments(comments: &Comments, options: &FmtOptions) -> String {
    let comments_str = comments
        .comments_list
        .iter()
        .fold(String::new(), |mut acc, c| {
            acc.push_str(&c.txt(options));
            acc
        });
    if comments_str.is_empty() {
        comments_str
    } else {
        let options = if let Some(cmt) = comments.comments_list.last() {
            if let Comment::LineComment(_) = &*cmt.comment {
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
    let mut indent = String::with_capacity((depth as usize + 1) * 4);
    indent.extend("    ".repeat(depth as usize + 1).drain(..));
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
        (
            FmtOptions {
                empty_line_after_prod: true,
                prod_semicolon_on_nl: false,
                max_line_length: 100,
                padding: super::Padding::None,
                line_end: super::LineEnd::Unchanged,
                trimming: super::Trimming::Unchanged,
                nesting_depth: 0,
            },
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/data/expected/prod_semicolon_on_nl_false"
            ),
        ),
    ];

    #[test]
    fn test_make_indent() {
        assert_eq!(String::from("    "), make_indent(0));
        let options = FmtOptions::new();
        assert_eq!(String::from("    "), make_indent(options.nesting_depth));
        assert_eq!(
            String::from("        "),
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
            let formatted_grammar = grammar.grammar.unwrap().txt(fmt_options);

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
