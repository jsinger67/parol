use lsp_types::{FormattingOptions, TextEdit};

use crate::{
    parol_ls_grammar_trait::{
        ASTControl, Alternation, AlternationList, Alternations, AlternationsList, BlockComment,
        Comments, CommentsList, CommentsListGroup, CutOperator, Declaration, DoubleColon, Factor,
        GrammarDefinition, GrammarDefinitionList, Group, Identifier, LineComment, LiteralString,
        NonTerminal, NonTerminalOpt, Optional, ParolLs, Production, ProductionLHS, Prolog,
        PrologList, PrologList0, Regex, Repeat, ScannerDirectives, ScannerState, ScannerStateList,
        ScannerSwitch, ScannerSwitchOpt, SimpleToken, SimpleTokenOpt, StartDeclaration, StateList,
        StateListList, Symbol, TokenLiteral, TokenWithStates, TokenWithStatesOpt,
        UserTypeDeclaration, UserTypeName, UserTypeNameList,
    },
    rng::Rng,
};

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

#[derive(Debug, Default, Clone)]
struct FmtOptions {
    padding: Padding,
    line_end: LineEnd,
    trimming: Trimming,
}

impl FmtOptions {
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
}

impl From<&FormattingOptions> for FmtOptions {
    fn from(_: &FormattingOptions) -> Self {
        // TODO: Modify if necessary
        Self::default()
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
            ASTControl::ASTControl0(_) => "^".to_string(),
            ASTControl::ASTControl1(ut) => ut.user_type_declaration.txt(options),
        }
    }
}
impl Fmt for Alternation {
    fn txt(&self, options: &FmtOptions) -> String {
        self.alternation_list
            .iter()
            .map(|a| a.txt(options))
            .collect::<Vec<String>>()
            .join(" ")
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
        format!(
            "{} {}",
            self.alternation.txt(options),
            self.alternations_list
                .iter()
                .fold(String::new(), |mut acc, a| {
                    acc.push_str(&a.txt(options));
                    acc
                })
        )
    }
}
impl Fmt for AlternationsList {
    fn txt(&self, options: &FmtOptions) -> String {
        let comment_options = options.clone().with_padding(Padding::Both);
        format!(
            "\n    {} {}{}",
            self.or,
            handle_comments(&self.comments, &comment_options),
            self.alternation.txt(options)
        )
    }
}
impl Fmt for BlockComment {
    fn txt(&self, _options: &FmtOptions) -> String {
        self.block_comment.text().to_string()
    }
}
impl Fmt for CommentsList {
    fn txt(&self, options: &FmtOptions) -> String {
        self.comments_list_group.txt(options)
    }
}
impl Fmt for CommentsListGroup {
    fn txt(&self, options: &FmtOptions) -> String {
        match self {
            CommentsListGroup::CommentsListGroup0(l) => l.line_comment.txt(options),
            CommentsListGroup::CommentsListGroup1(b) => b.block_comment.txt(options),
        }
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
            Declaration::Declaration0(title) => format!(
                "{} {}{}\n",
                title.percent_title,
                title.string.txt(options),
                handle_comments(&title.comments, &comment_options),
            ),
            Declaration::Declaration1(comment) => format!(
                "{} {}{}\n",
                comment.percent_comment,
                comment.string.txt(options),
                handle_comments(&comment.comments, &comment_options),
            ),
            Declaration::Declaration2(user_type) => format!(
                "{} {} {} {}{}\n",
                user_type.percent_user_underscore_type,
                user_type.identifier.txt(options),
                user_type.equ,
                user_type.user_type_name.txt(options),
                handle_comments(&user_type.comments, &comment_options),
            ),
            Declaration::Declaration3(scanner_directives) => {
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
        match self {
            Factor::Factor0(g) => g.group.txt(options),
            Factor::Factor1(r) => r.repeat.txt(options),
            Factor::Factor2(o) => o.optional.txt(options),
            Factor::Factor3(s) => handle_symbol(&s.symbol, options),
        }
    }
}
impl Fmt for GrammarDefinition {
    fn txt(&self, options: &FmtOptions) -> String {
        format!(
            "{}{}{}",
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
        format!(
            "{} {}{}\n",
            self.l_paren,
            self.alternations.txt(options),
            self.r_paren,
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
        format!(
            "{} {}{}",
            self.l_bracket,
            self.alternations.txt(options),
            self.r_bracket,
        )
    }
}
impl Fmt for ParolLs {
    fn txt(&self, options: &FmtOptions) -> String {
        format!(
            "{}\n{}",
            self.prolog.txt(options),
            self.grammar_definition.txt(options),
        )
    }
}
impl Fmt for Production {
    fn txt(&self, options: &FmtOptions) -> String {
        format!(
            "\n{} {}\n    {}",
            self.production_l_h_s.txt(options),
            self.alternations.txt(options).trim(),
            self.semicolon,
        )
    }
}
impl Fmt for ProductionLHS {
    fn txt(&self, options: &FmtOptions) -> String {
        let comment_options_both = options
            .clone()
            .with_padding(Padding::Both)
            .with_trimming(Trimming::TrimRight);
        let comment_options_right = options
            .clone()
            .with_line_end(LineEnd::ForceAdd)
            .with_trimming(Trimming::TrimRight);
        format!(
            "\n{}{}{}\n    {}",
            handle_comments(&self.comments, &comment_options_right),
            self.identifier.identifier,
            handle_comments(&self.comments0, &comment_options_both),
            self.colon,
        )
    }
}
impl Fmt for Prolog {
    fn txt(&self, options: &FmtOptions) -> String {
        format!(
            "{}{}{}",
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
        format!(
            "{} {}{}",
            self.l_brace,
            self.alternations.txt(options),
            self.r_brace,
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
            "{} {} {}\n{} {}\n",
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
            ScannerSwitch::ScannerSwitch0(sc) => format!(
                "{}{}{}{}",
                sc.percent_sc,
                sc.l_paren,
                sc.scanner_switch_opt
                    .as_ref()
                    .map_or(String::default(), |s| { s.txt(options) }),
                sc.r_paren,
            ),
            ScannerSwitch::ScannerSwitch1(push) => format!(
                "{}{}{}{}",
                push.percent_push,
                push.l_paren,
                push.identifier.txt(options),
                push.r_paren,
            ),
            ScannerSwitch::ScannerSwitch2(pop) => {
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
            TokenLiteral::TokenLiteral0(s) => s.string.txt(options),
            TokenLiteral::TokenLiteral1(l) => l.literal_string.txt(options),
            TokenLiteral::TokenLiteral2(r) => r.regex.txt(options),
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
        ScannerDirectives::ScannerDirectives0(l) => format!(
            "{} {}{}\n",
            l.percent_line_underscore_comment,
            l.token_literal.txt(options),
            handle_comments(&l.comments, &comment_options),
        ),
        ScannerDirectives::ScannerDirectives1(b) => format!(
            "{} {} {}{}\n",
            b.percent_block_underscore_comment,
            b.token_literal.txt(options),
            b.token_literal0.txt(options),
            handle_comments(&b.comments, &comment_options),
        ),

        ScannerDirectives::ScannerDirectives2(n) => format!(
            "{}{}\n",
            n.percent_auto_underscore_newline_underscore_off,
            handle_comments(&n.comments, &comment_options),
        ),

        ScannerDirectives::ScannerDirectives3(w) => format!(
            "{}{}\n",
            w.percent_auto_underscore_ws_underscore_off,
            handle_comments(&w.comments, &comment_options),
        ),
    }
}

fn handle_symbol(symbol: &Symbol, options: &FmtOptions) -> String {
    match symbol {
        Symbol::Symbol0(n) => n.non_terminal.txt(options),
        Symbol::Symbol1(t) => t.simple_token.txt(options),
        Symbol::Symbol2(t) => t.token_with_states.txt(options),
        Symbol::Symbol3(s) => s.scanner_switch.txt(options),
    }
}

fn handle_comments(comments: &Comments, options: &FmtOptions) -> String {
    let comments = comments
        .comments_list
        .iter()
        .fold(String::new(), |mut acc, c| {
            acc.push_str(&c.txt(options));
            acc
        });
    if comments.is_empty() {
        comments
    } else {
        apply_formatting(comments, options)
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
    };
    match options.padding {
        Padding::None => line,
        Padding::Left => format!(" {}", line),
        Padding::Right => format!("{} ", line),
        Padding::Both => format!(" {} ", line),
    }
}
