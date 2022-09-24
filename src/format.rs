use lsp_types::{FormattingOptions, TextEdit};

use crate::{
    parol_ls_grammar_trait::{
        ASTControl, Alternation, AlternationList, Alternations, AlternationsList, BlockComment,
        Comments, CommentsList, CommentsListGroup, CutOperator, Declaration, DoubleColon, Factor,
        GrammarDefinition, GrammarDefinitionList, Group, Identifier, LineComment, NonTerminal,
        NonTerminalOpt, Optional, ParolLs, Production, ProductionLHS, Prolog, PrologList,
        PrologList0, Repeat, ScannerDirectives, ScannerState, ScannerStateList, ScannerSwitch,
        ScannerSwitchOpt, SimpleToken, SimpleTokenOpt, StartDeclaration, StateList, StateListList,
        Symbol, TokenWithStates, TokenWithStatesOpt, UserTypeDeclaration, UserTypeName,
        UserTypeNameList,
    },
    rng::Rng,
};

#[allow(dead_code)]
enum Padding {
    None,
    Left,
    Right,
    Both,
}

pub(crate) trait Format {
    fn format(&self, options: &FormattingOptions) -> Vec<TextEdit>;
}

impl Format for &ParolLs {
    fn format(&self, options: &FormattingOptions) -> Vec<TextEdit> {
        let range = <&ParolLs as Into<Rng>>::into(*self).0;
        let new_text = self.txt(options);
        vec![TextEdit { range, new_text }]
    }
}

trait Fmt {
    fn txt(&self, options: &FormattingOptions) -> String;
}

impl Fmt for ASTControl {
    fn txt(&self, options: &FormattingOptions) -> String {
        match self {
            ASTControl::ASTControl0(_) => "^".to_string(),
            ASTControl::ASTControl1(ut) => ut.user_type_declaration.txt(options),
        }
    }
}
impl Fmt for Alternation {
    fn txt(&self, options: &FormattingOptions) -> String {
        self.alternation_list
            .iter()
            .map(|a| a.txt(options))
            .collect::<Vec<String>>()
            .join(" ")
    }
}
impl Fmt for AlternationList {
    fn txt(&self, options: &FormattingOptions) -> String {
        format!(
            "{}{}",
            self.factor.txt(options),
            handle_comments(&*self.comments, Padding::Left, options)
        )
    }
}
impl Fmt for Alternations {
    fn txt(&self, options: &FormattingOptions) -> String {
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
    fn txt(&self, options: &FormattingOptions) -> String {
        format!(
            "\n    {} {}{}",
            self.or.symbol,
            handle_comments(&*self.comments, Padding::Both, options),
            self.alternation.txt(options)
        )
    }
}
impl Fmt for BlockComment {
    fn txt(&self, _options: &FormattingOptions) -> String {
        self.block_comment.symbol.clone()
    }
}
impl Fmt for CommentsList {
    fn txt(&self, options: &FormattingOptions) -> String {
        self.comments_list_group.txt(options)
    }
}
impl Fmt for CommentsListGroup {
    fn txt(&self, options: &FormattingOptions) -> String {
        match self {
            CommentsListGroup::CommentsListGroup0(l) => l.line_comment.txt(options),
            CommentsListGroup::CommentsListGroup1(b) => b.block_comment.txt(options),
        }
    }
}
impl Fmt for CutOperator {
    fn txt(&self, _options: &FormattingOptions) -> String {
        self.cut_operator.symbol.clone()
    }
}
impl Fmt for Declaration {
    fn txt(&self, options: &FormattingOptions) -> String {
        match self {
            Declaration::Declaration0(title) => format!(
                "{} {}{}\n",
                title.percent_title.symbol,
                title.string.txt(options),
                handle_comments(&*title.comments, Padding::Left, options),
            ),
            Declaration::Declaration1(comment) => format!(
                "{} {}{}\n",
                comment.percent_comment.symbol,
                comment.string.txt(options),
                handle_comments(&*comment.comments, Padding::Left, options),
            ),
            Declaration::Declaration2(user_type) => format!(
                "{} {} {} {}{}\n",
                user_type.percent_user_underscore_type.symbol,
                user_type.identifier.txt(options),
                user_type.equ.symbol,
                user_type.user_type_name.txt(options),
                handle_comments(&*user_type.comments, Padding::Left, options),
            ),
            Declaration::Declaration3(scanner_directives) => {
                handle_scanner_directives(&*scanner_directives.scanner_directives, options)
            }
        }
    }
}

impl Fmt for DoubleColon {
    fn txt(&self, _options: &FormattingOptions) -> String {
        self.double_colon.symbol.clone()
    }
}
impl Fmt for Factor {
    fn txt(&self, options: &FormattingOptions) -> String {
        match self {
            Factor::Factor0(g) => g.group.txt(options),
            Factor::Factor1(r) => r.repeat.txt(options),
            Factor::Factor2(o) => o.optional.txt(options),
            Factor::Factor3(s) => handle_symbol(&*s.symbol, options),
        }
    }
}
impl Fmt for GrammarDefinition {
    fn txt(&self, options: &FormattingOptions) -> String {
        format!(
            "{}{}{}",
            self.percent_percent.symbol,
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
    fn txt(&self, options: &FormattingOptions) -> String {
        self.production.txt(options)
    }
}
impl Fmt for Group {
    fn txt(&self, options: &FormattingOptions) -> String {
        format!(
            "{} {}{}\n",
            self.l_paren.symbol,
            self.alternations.txt(options),
            self.r_paren.symbol,
        )
    }
}
impl Fmt for Identifier {
    fn txt(&self, _options: &FormattingOptions) -> String {
        self.identifier.symbol.clone()
    }
}
impl Fmt for LineComment {
    fn txt(&self, _options: &FormattingOptions) -> String {
        self.line_comment.symbol.clone()
    }
}
impl Fmt for NonTerminal {
    fn txt(&self, options: &FormattingOptions) -> String {
        format!(
            "{}{}",
            self.identifier.identifier.symbol,
            self.non_terminal_opt
                .as_ref()
                .map_or(String::default(), |a| { a.txt(options) })
        )
    }
}
impl Fmt for NonTerminalOpt {
    fn txt(&self, options: &FormattingOptions) -> String {
        self.a_s_t_control.txt(options)
    }
}
impl Fmt for Optional {
    fn txt(&self, options: &FormattingOptions) -> String {
        format!(
            "{} {}{}",
            self.l_bracket.symbol,
            self.alternations.txt(options),
            self.r_bracket.symbol,
        )
    }
}
impl Fmt for ParolLs {
    fn txt(&self, options: &FormattingOptions) -> String {
        format!(
            "{}\n{}",
            self.prolog.txt(options),
            self.grammar_definition.txt(options),
        )
    }
}
impl Fmt for Production {
    fn txt(&self, options: &FormattingOptions) -> String {
        format!(
            "\n{}{}\n    {}",
            self.production_l_h_s.txt(options),
            self.alternations.txt(options).trim(),
            self.semicolon.symbol,
        )
    }
}
impl Fmt for ProductionLHS {
    fn txt(&self, options: &FormattingOptions) -> String {
        format!(
            "\n{}{}{}\n    {} ",
            handle_comments(&*self.comments, Padding::Right, options),
            self.identifier.identifier.symbol,
            handle_comments(&*self.comments0, Padding::Both, options),
            self.colon.symbol,
        )
    }
}
impl Fmt for Prolog {
    fn txt(&self, options: &FormattingOptions) -> String {
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
    fn txt(&self, options: &FormattingOptions) -> String {
        self.declaration.txt(options)
    }
}
impl Fmt for PrologList0 {
    fn txt(&self, options: &FormattingOptions) -> String {
        self.scanner_state.txt(options)
    }
}
impl Fmt for Repeat {
    fn txt(&self, options: &FormattingOptions) -> String {
        format!(
            "{} {}{}",
            self.l_brace.symbol,
            self.alternations.txt(options),
            self.r_brace.symbol,
        )
    }
}
impl Fmt for ScannerDirectives {
    fn txt(&self, options: &FormattingOptions) -> String {
        handle_scanner_directives(self, options)
    }
}
impl Fmt for ScannerState {
    fn txt(&self, options: &FormattingOptions) -> String {
        format!(
            "{} {} {}\n{} {}\n",
            self.percent_scanner.symbol,
            self.identifier.txt(options),
            self.l_brace.symbol,
            self.scanner_state_list
                .iter()
                .fold(String::new(), |mut acc, s| {
                    acc.push_str("    ");
                    acc.push_str(&s.txt(options));
                    acc
                }),
            self.r_brace.symbol,
        )
    }
}
impl Fmt for ScannerStateList {
    fn txt(&self, options: &FormattingOptions) -> String {
        handle_scanner_directives(&self.scanner_directives, options)
    }
}
impl Fmt for ScannerSwitch {
    fn txt(&self, options: &FormattingOptions) -> String {
        match self {
            ScannerSwitch::ScannerSwitch0(sc) => format!(
                "{}{}{}{}",
                sc.percent_sc.symbol,
                sc.l_paren.symbol,
                sc.scanner_switch_opt
                    .as_ref()
                    .map_or(String::default(), |s| { s.txt(options) }),
                sc.r_paren.symbol,
            ),
            ScannerSwitch::ScannerSwitch1(push) => format!(
                "{}{}{}{}",
                push.percent_push.symbol,
                push.l_paren.symbol,
                push.identifier.txt(options),
                push.r_paren.symbol,
            ),
            ScannerSwitch::ScannerSwitch2(pop) => format!(
                "{}{}{}",
                pop.percent_pop.symbol, pop.l_paren.symbol, pop.r_paren.symbol,
            ),
        }
    }
}
impl Fmt for ScannerSwitchOpt {
    fn txt(&self, options: &FormattingOptions) -> String {
        self.identifier.txt(options)
    }
}
impl Fmt for SimpleToken {
    fn txt(&self, options: &FormattingOptions) -> String {
        format!(
            "{}{}",
            self.string.string.symbol,
            self.simple_token_opt
                .as_ref()
                .map_or(String::default(), |s| { s.txt(options) })
        )
    }
}
impl Fmt for SimpleTokenOpt {
    fn txt(&self, options: &FormattingOptions) -> String {
        self.a_s_t_control.txt(options)
    }
}
impl Fmt for StartDeclaration {
    fn txt(&self, options: &FormattingOptions) -> String {
        format!(
            "{}{} {}{}\n",
            handle_comments(&*self.comments, Padding::Right, options),
            self.percent_start.symbol,
            self.identifier.txt(options),
            handle_comments(&*self.comments0, Padding::Left, options),
        )
    }
}
impl Fmt for StateList {
    fn txt(&self, options: &FormattingOptions) -> String {
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
    fn txt(&self, options: &FormattingOptions) -> String {
        format!("{} {}", self.comma.symbol, self.identifier.txt(options),)
    }
}
impl Fmt for crate::parol_ls_grammar_trait::String {
    fn txt(&self, _options: &FormattingOptions) -> String {
        self.string.symbol.clone()
    }
}
impl Fmt for Symbol {
    fn txt(&self, options: &FormattingOptions) -> String {
        handle_symbol(self, options)
    }
}
impl Fmt for TokenWithStates {
    fn txt(&self, options: &FormattingOptions) -> String {
        format!(
            "{}{}{}{}{}",
            self.l_t.symbol,
            self.state_list.txt(options).trim(),
            self.g_t.symbol,
            self.string.txt(options),
            self.token_with_states_opt
                .as_ref()
                .map_or(String::default(), |a| { a.txt(options) }),
        )
    }
}
impl Fmt for TokenWithStatesOpt {
    fn txt(&self, options: &FormattingOptions) -> String {
        self.a_s_t_control.txt(options)
    }
}
impl Fmt for UserTypeDeclaration {
    fn txt(&self, options: &FormattingOptions) -> String {
        format!("{} {}", self.colon.symbol, self.user_type_name.txt(options),)
    }
}
impl Fmt for UserTypeName {
    fn txt(&self, options: &FormattingOptions) -> String {
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
    fn txt(&self, options: &FormattingOptions) -> String {
        format!(
            "{}{}",
            self.double_colon.double_colon.symbol,
            self.identifier.txt(options),
        )
    }
}

fn handle_scanner_directives(
    scanner_directives: &ScannerDirectives,
    options: &FormattingOptions,
) -> String {
    match scanner_directives {
        ScannerDirectives::ScannerDirectives0(l) => format!(
            "{} {}{}\n",
            l.percent_line_underscore_comment.symbol,
            l.string.txt(options),
            handle_comments(&*l.comments, Padding::Left, options),
        ),
        ScannerDirectives::ScannerDirectives1(b) => format!(
            "{} {} {}{}\n",
            b.percent_block_underscore_comment.symbol,
            b.string.txt(options),
            b.string0.txt(options),
            handle_comments(&*b.comments, Padding::Left, options),
        ),
        ScannerDirectives::ScannerDirectives2(n) => format!(
            "{}{}\n",
            n.percent_auto_underscore_newline_underscore_off.symbol,
            handle_comments(&*n.comments, Padding::Left, options),
        ),
        ScannerDirectives::ScannerDirectives3(w) => format!(
            "{}{}\n",
            w.percent_auto_underscore_ws_underscore_off.symbol,
            handle_comments(&*w.comments, Padding::Left, options),
        ),
    }
}

fn handle_symbol(symbol: &Symbol, options: &FormattingOptions) -> String {
    match symbol {
        Symbol::Symbol0(n) => n.non_terminal.txt(options),
        Symbol::Symbol1(t) => t.simple_token.txt(options),
        Symbol::Symbol2(t) => t.token_with_states.txt(options),
        Symbol::Symbol3(s) => s.scanner_switch.txt(options),
    }
}

fn handle_comments(comments: &Comments, padding: Padding, options: &FormattingOptions) -> String {
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
        match padding {
            Padding::None => comments,
            Padding::Left => format!(" {}", comments),
            Padding::Right => format!("{} ", comments),
            Padding::Both => format!(" {} ", comments),
        }
    }
}
