use lsp_types::{FormattingOptions, Range, TextEdit};
use std::vec;

use crate::{
    parol_ls_grammar::OwnedToken,
    parol_ls_grammar_trait::{
        ASTControl, Alternation, AlternationList, Alternations, AlternationsList, CutOperator,
        Declaration, DoubleColon, Factor, GrammarDefinition, GrammarDefinitionList, Group,
        Identifier, IdentifierList, IdentifierListList, LiteralString, NonTerminal, NonTerminalOpt,
        Optional, ParolLs, Production, ProductionLHS, Prolog, PrologList, PrologList0, Regex,
        Repeat, ScannerDirectives, ScannerState, ScannerStateList, ScannerSwitch, ScannerSwitchOpt,
        SimpleToken, SimpleTokenOpt, StartDeclaration, Symbol, TokenLiteral, TokenWithStates,
        TokenWithStatesOpt, UserTypeDeclaration, UserTypeName, UserTypeNameList,
    },
    rng::Rng,
    utils::RX_NEW_LINE,
};

use super::{Comments, FmtOptions, Indent, Line, LineEnd, Padding};

// This is the actual start column for each production (alternation) line
const START_LINE_OFFSET: usize = 6;

pub(crate) trait Format {
    fn format(&self, options: &FormattingOptions, comments: Comments) -> Vec<TextEdit>;
}

trait LastToken {
    fn get_last_token(&self) -> &OwnedToken;
}

impl LastToken for UserTypeName {
    fn get_last_token(&self) -> &OwnedToken {
        if self.user_type_name_list.is_empty() {
            &self.identifier.identifier
        } else {
            &self
                .user_type_name_list
                .last()
                .unwrap()
                .identifier
                .identifier
        }
    }
}

impl LastToken for TokenLiteral {
    fn get_last_token(&self) -> &OwnedToken {
        match self {
            TokenLiteral::String(s) => &s.string.string,
            TokenLiteral::LiteralString(l) => &l.literal_string.literal_string,
            TokenLiteral::Regex(r) => &r.regex.regex,
        }
    }
}

impl Format for &ParolLs {
    fn format(&self, options: &FormattingOptions, comments: Comments) -> Vec<TextEdit> {
        // We use the complete document's range for the edit to ensure that the whole document is
        // replaced. This is necessary to avoid problems with comments at the start and the end of
        // the document.
        let range = Rng::new(Range::default()).extend_to_end().0;
        let fmt_options = options.into();
        let (new_text, comments) = self.txt(&fmt_options, comments);
        debug_assert!(comments.is_empty());
        vec![TextEdit { range, new_text }]
    }
}

pub(crate) trait Fmt {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments);
}

impl Fmt for ASTControl {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        match self {
            ASTControl::CutOperator(_) => ("^".to_string(), comments),
            ASTControl::UserTypeDeclaration(ut) => ut.user_type_declaration.txt(options, comments),
        }
    }
}
impl Fmt for Alternation {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
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
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        self.factor.txt(options, comments)
    }
}
impl Fmt for Alternations {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        let (first_alternation_str, comments) = self.alternation.txt(options, comments);
        let (all_alternations_str, comments) = self.alternations_list.iter().fold(
            (first_alternation_str, comments),
            |(mut acc, comments), a| {
                let (comments_before_or, comments) = Comments::format_comments_before(
                    comments,
                    &a.or,
                    &options.clone().with_padding(Padding::Right),
                );

                if Line::ends_with_nl(&acc) && !comments_before_or.is_empty() {
                    // Pull the comment on the line with the alternation.
                    acc.clone_from(&acc.trim_end().to_owned());
                    // Add a space between alternation_str and the line comment.
                    acc.push(' ');
                    acc.push_str(&comments_before_or);
                }

                let (alternations_str, comments) = a.txt(options, comments);
                acc.push_str(&alternations_str);
                (acc, comments)
            },
        );
        let delimiter = ""; //if options.nesting_depth == 0 { "" } else { " " };
        (format!("{}{}", delimiter, all_alternations_str), comments)
    }
}
impl Fmt for AlternationsList {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        let (alternation_str, comments) = self.alternation.txt(options, comments);
        let right_padding = "";
        let left_padding = if options.nesting_depth == 0 {
            "    "
        } else {
            " "
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
    fn txt(&self, _options: &FmtOptions, comments: Comments) -> (String, Comments) {
        (self.cut_operator.text().to_string(), comments)
    }
}
impl Fmt for Declaration {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        let mut delim = "";
        match self {
            Declaration::PercentTitleString(title) => {
                let (comments_before_token, comments) = Comments::format_comments_before(
                    comments,
                    &title.percent_title,
                    &options.clone().with_padding(Padding::Left),
                );
                if comments_before_token.is_empty() || !Line::ends_with_nl(&comments_before_token) {
                    delim = "\n";
                };
                let (str, comments) = title.string.txt(options, comments);
                (
                    format!(
                        "{}{}{} {}",
                        comments_before_token, delim, title.percent_title, str
                    ),
                    comments,
                )
            }
            Declaration::PercentCommentString(comment) => {
                let (comments_before_token, comments) = Comments::format_comments_before(
                    comments,
                    &comment.percent_comment,
                    &options.clone().with_padding(Padding::Left),
                );
                if comments_before_token.is_empty() || !Line::ends_with_nl(&comments_before_token) {
                    delim = "\n";
                };
                let (str, comments) = comment.string.txt(options, comments);
                (
                    format!(
                        "{}{}{} {}",
                        comments_before_token, delim, comment.percent_comment, str
                    ),
                    comments,
                )
            }
            Declaration::PercentUserUnderscoreTypeIdentifierEquUserTypeName(user_type) => {
                // "%user_type" Identifier "=" UserTypeName;
                // %user_type UserType1 = UserDefinedTypeName1 // comment
                let (comments_before_token, comments) = Comments::format_comments_before(
                    comments,
                    &user_type.percent_user_underscore_type,
                    &options
                        .clone()
                        .with_padding(Padding::Left)
                        .with_line_end(LineEnd::ForceRemove),
                );
                let (alias_name, comments) = user_type.identifier.txt(options, comments);
                let (orig_type_name, comments) = user_type.user_type_name.txt(options, comments);
                let (following_comment, comments) =
                    Comments::formatted_immediately_following_comment(
                        comments,
                        user_type.user_type_name.get_last_token(),
                        &options
                            .clone()
                            .with_padding(Padding::Left)
                            .with_line_end(LineEnd::ForceRemove),
                    );
                if comments_before_token.is_empty() || !Line::ends_with_nl(&comments_before_token) {
                    delim = "\n";
                };
                (
                    format!(
                        "{}{}{} {} {} {}{}",
                        comments_before_token,
                        delim,
                        user_type.percent_user_underscore_type,
                        alias_name,
                        user_type.equ,
                        orig_type_name,
                        following_comment,
                    ),
                    comments,
                )
            }
            Declaration::ScannerDirectives(scanner_directives) => {
                handle_scanner_directives(&scanner_directives.scanner_directives, options, comments)
            }
            Declaration::PercentGrammarUnderscoreTypeLiteralString(grammar_type) => {
                let (comments_before_token, comments) = Comments::format_comments_before(
                    comments,
                    &grammar_type.percent_grammar_underscore_type,
                    &options.clone().with_padding(Padding::Left),
                );
                if comments_before_token.is_empty() || !Line::ends_with_nl(&comments_before_token) {
                    delim = "\n";
                };
                let (str, comments) = grammar_type.literal_string.txt(options, comments);
                (
                    format!(
                        "{}{}{} {}",
                        comments_before_token,
                        delim,
                        grammar_type.percent_grammar_underscore_type,
                        str
                    ),
                    comments,
                )
            }
        }
    }
}

impl Fmt for DoubleColon {
    fn txt(&self, _options: &FmtOptions, comments: Comments) -> (String, Comments) {
        (self.double_colon.text().to_string(), comments)
    }
}
impl Fmt for Factor {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
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
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
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
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        self.production.txt(options, comments)
    }
}
impl Fmt for Group {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
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
    fn txt(&self, _options: &FmtOptions, comments: Comments) -> (String, Comments) {
        (self.identifier.text().to_string(), comments)
    }
}
impl Fmt for LiteralString {
    fn txt(&self, _options: &FmtOptions, comments: Comments) -> (String, Comments) {
        (self.literal_string.text().to_string(), comments)
    }
}
impl Fmt for NonTerminal {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        let (ast_control_str, comments) =
            if let Some(non_terminal_opt) = self.non_terminal_opt.as_ref() {
                non_terminal_opt.txt(options, comments)
            } else {
                (String::default(), comments)
            };

        let (comments_before_identifier, comments) = Comments::format_comments_before(
            comments,
            &self.identifier.identifier,
            &options.clone().with_padding(Padding::Right),
        );
        let mut delim = String::new();
        if Line::ends_with_nl(&comments_before_identifier) {
            "      ".clone_into(&mut delim);
        }
        (
            format!(
                "{}{}{}{}",
                comments_before_identifier, delim, self.identifier.identifier, ast_control_str
            ),
            comments,
        )
    }
}
impl Fmt for NonTerminalOpt {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        self.a_s_t_control.txt(options, comments)
    }
}
impl Fmt for Optional {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
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
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
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
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        let (production_l_h_s, comments) = self.production_l_h_s.txt(options, comments);
        let (mut alternations_str, comments) = self.alternations.txt(options, comments);
        let (mut comments_before_semicolon, comments) = Comments::format_comments_before(
            comments,
            &self.semicolon,
            &options.clone().with_padding(Padding::Left),
        );

        let mut semi_nl_opt = "";
        alternations_str.clone_from(&alternations_str.trim_end().to_owned());
        if options.prod_semicolon_on_nl || Line::ends_with_nl(&comments_before_semicolon) {
            if Line::ends_with_nl(&comments_before_semicolon) {
                comments_before_semicolon
                    .clone_from(&comments_before_semicolon.trim_end().to_owned());
            }
            semi_nl_opt = "\n    ";
        }

        let prod_nl_opt = if options.empty_line_after_prod {
            "\n"
        } else {
            ""
        };
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
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        let (comments_before_non_terminal, comments) = Comments::format_comments_before(
            comments,
            &self.identifier.identifier,
            &options.clone().with_line_end(LineEnd::ForceSingleNewline),
        );
        let (comments_before_colon, comments) = Comments::format_comments_before(
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
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
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
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        self.declaration.txt(options, comments)
    }
}
impl Fmt for PrologList0 {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        self.scanner_state.txt(options, comments)
    }
}
impl Fmt for Regex {
    fn txt(&self, _options: &FmtOptions, comments: Comments) -> (String, Comments) {
        (self.regex.text().to_string(), comments)
    }
}
impl Fmt for Repeat {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
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
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        handle_scanner_directives(self, options, comments)
    }
}
impl Fmt for ScannerState {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
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

        let (comments_before_scanner, comments) = Comments::format_comments_before(
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
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        handle_scanner_directives(&self.scanner_directives, options, comments)
    }
}
impl Fmt for ScannerSwitch {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
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
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        self.identifier.txt(options, comments)
    }
}
impl Fmt for SimpleToken {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
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
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        self.a_s_t_control.txt(options, comments)
    }
}
impl Fmt for StartDeclaration {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        let (comments_before_start, comments) = Comments::format_comments_before(
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
impl Fmt for IdentifierList {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        let (identifier, comments) = self.identifier.txt(options, comments);
        let (state_list_list, comments) = self.identifier_list_list.iter().fold(
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
impl Fmt for IdentifierListList {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        let (identifier, comments) = self.identifier.txt(options, comments);
        (format!("{} {}", self.comma, identifier), comments)
    }
}
impl Fmt for crate::parol_ls_grammar_trait::String {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        let (comments_before_string, comments) = Comments::format_comments_before(
            comments,
            &self.string,
            &options.clone().with_padding(Padding::Right),
        );
        (
            format!("{}{}", comments_before_string, self.string.text()),
            comments,
        )
    }
}
impl Fmt for Symbol {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        handle_symbol(self, options, comments)
    }
}
impl Fmt for TokenLiteral {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        match self {
            TokenLiteral::String(s) => s.string.txt(options, comments),
            TokenLiteral::LiteralString(l) => l.literal_string.txt(options, comments),
            TokenLiteral::Regex(r) => r.regex.txt(options, comments),
        }
    }
}
impl Fmt for TokenWithStates {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        let (mut state_list, comments) = self.identifier_list.txt(options, comments);
        let (token_literal, comments) = self.token_literal.txt(options, comments);
        let (token_with_states_opt, comments) =
            if let Some(token_with_states_opt) = self.token_with_states_opt.as_ref() {
                token_with_states_opt.txt(options, comments)
            } else {
                (String::default(), comments)
            };
        state_list.clone_from(&state_list.trim().to_owned());
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
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        self.a_s_t_control.txt(options, comments)
    }
}
impl Fmt for UserTypeDeclaration {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        let (user_type_name, comments) = self.user_type_name.txt(options, comments);
        (format!("{} {}", self.colon, user_type_name), comments)
    }
}
impl Fmt for UserTypeName {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
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
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        let (identifier, comments) = self.identifier.txt(options, comments);
        (
            format!("{}{}", self.double_colon.double_colon, identifier),
            comments,
        )
    }
}

fn handle_scanner_directives(
    scanner_directives: &ScannerDirectives,
    options: &FmtOptions,
    comments: Comments,
) -> (String, Comments) {
    let mut indent = Indent::make_indent(options.nesting_depth);
    match scanner_directives {
        ScannerDirectives::PercentLineUnderscoreCommentTokenLiteral(l) => {
            let (comments_before_token, comments) = Comments::format_comments_before(
                comments,
                &l.percent_line_underscore_comment,
                &options.clone().with_padding(Padding::Left),
            );
            if comments_before_token.is_empty() || !Line::ends_with_nl(&comments_before_token) {
                indent.insert(0, '\n');
            };
            let (comment_str, comments) = l.token_literal.txt(options, comments);
            let (following_comment, comments) = Comments::formatted_immediately_following_comment(
                comments,
                l.token_literal.get_last_token(),
                &options
                    .clone()
                    .with_padding(Padding::Left)
                    .with_line_end(LineEnd::ForceRemove),
            );
            (
                format!(
                    "{}{}{} {}{}",
                    comments_before_token,
                    indent,
                    l.percent_line_underscore_comment,
                    comment_str,
                    following_comment,
                ),
                comments,
            )
        }
        ScannerDirectives::PercentBlockUnderscoreCommentTokenLiteralTokenLiteral(b) => {
            let (comments_before_token, comments) = Comments::format_comments_before(
                comments,
                &b.percent_block_underscore_comment,
                &options.clone().with_padding(Padding::Left),
            );
            let (str1, comments) = b.token_literal.txt(options, comments);
            let (str2, comments) = b.token_literal0.txt(options, comments);
            if comments_before_token.is_empty() || !Line::ends_with_nl(&comments_before_token) {
                indent.insert(0, '\n');
            };
            let (following_comment, comments) = Comments::formatted_immediately_following_comment(
                comments,
                b.token_literal0.get_last_token(),
                &options
                    .clone()
                    .with_padding(Padding::Left)
                    .with_line_end(LineEnd::ForceRemove),
            );
            (
                format!(
                    "{}{}{} {} {}{}",
                    comments_before_token,
                    indent,
                    b.percent_block_underscore_comment,
                    str1,
                    str2,
                    following_comment,
                ),
                comments,
            )
        }

        ScannerDirectives::PercentAutoUnderscoreNewlineUnderscoreOff(n) => {
            let (comments_before_token, comments) = Comments::format_comments_before(
                comments,
                &n.percent_auto_underscore_newline_underscore_off,
                &options.clone().with_padding(Padding::Left),
            );
            if comments_before_token.is_empty() || !Line::ends_with_nl(&comments_before_token) {
                indent.insert(0, '\n');
            };
            let (following_comment, comments) = Comments::formatted_immediately_following_comment(
                comments,
                &n.percent_auto_underscore_newline_underscore_off,
                &options
                    .clone()
                    .with_padding(Padding::Left)
                    .with_line_end(LineEnd::ForceRemove),
            );
            (
                format!(
                    "{}{}{}{}",
                    comments_before_token,
                    indent,
                    n.percent_auto_underscore_newline_underscore_off,
                    following_comment
                ),
                comments,
            )
        }

        ScannerDirectives::PercentAutoUnderscoreWsUnderscoreOff(w) => {
            let (comments_before_token, comments) = comments.format_comments_before(
                &w.percent_auto_underscore_ws_underscore_off,
                &options.clone().with_padding(Padding::Left),
            );
            if comments_before_token.is_empty() || !Line::ends_with_nl(&comments_before_token) {
                indent.insert(0, '\n');
            };
            let (following_comment, comments) = Comments::formatted_immediately_following_comment(
                comments,
                &w.percent_auto_underscore_ws_underscore_off,
                &options
                    .clone()
                    .with_padding(Padding::Left)
                    .with_line_end(LineEnd::ForceRemove),
            );
            (
                format!(
                    "{}{}{}{}",
                    comments_before_token,
                    indent,
                    w.percent_auto_underscore_ws_underscore_off,
                    following_comment
                ),
                comments,
            )
        }
        ScannerDirectives::PercentOnIdentifierListPercentEnterIdentifier(trans) => {
            let (comments_before_token, comments) = Comments::format_comments_before(
                comments,
                &trans.percent_on,
                &options.clone().with_padding(Padding::Left),
            );
            if comments_before_token.is_empty() || !Line::ends_with_nl(&comments_before_token) {
                indent.insert(0, '\n');
            };
            let (following_comment, comments) = Comments::formatted_immediately_following_comment(
                comments,
                &trans.identifier.identifier,
                &options
                    .clone()
                    .with_padding(Padding::Left)
                    .with_line_end(LineEnd::ForceRemove),
            );
            let ident_list = trans
                .identifier_list
                .identifier_list_list
                .iter()
                .fold(
                    vec![trans
                        .identifier_list
                        .identifier
                        .identifier
                        .text()
                        .to_string()],
                    |mut acc, i| {
                        acc.push(i.identifier.identifier.text().to_string());
                        acc
                    },
                )
                .join(", ");
            (
                format!(
                    "{}{}{} {} {} {}{}",
                    comments_before_token,
                    indent,
                    trans.percent_on,
                    ident_list,
                    trans.percent_enter,
                    trans.identifier.identifier.text(),
                    following_comment
                ),
                comments,
            )
        }
    }
}

fn handle_symbol(symbol: &Symbol, options: &FmtOptions, comments: Comments) -> (String, Comments) {
    match symbol {
        Symbol::NonTerminal(n) => n.non_terminal.txt(options, comments),
        Symbol::SimpleToken(t) => t.simple_token.txt(options, comments),
        Symbol::TokenWithStates(t) => t.token_with_states.txt(options, comments),
        Symbol::ScannerSwitch(s) => s.scanner_switch.txt(options, comments),
    }
}

#[cfg(test)]
mod test {
    use std::{ffi::OsStr, fs};

    use parol_runtime::Report;

    use crate::{
        formatting::{fmt_options::Trimming, format::Fmt, FmtOptions, LineEnd, Padding},
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
                padding: Padding::None,
                line_end: LineEnd::Unchanged,
                trimming: Trimming::Unchanged,
                nesting_depth: 0,
            },
            concat!(env!("CARGO_MANIFEST_DIR"), "/data/expected/options_default"),
        ),
        (
            FmtOptions {
                empty_line_after_prod: true,
                prod_semicolon_on_nl: false,
                max_line_length: 100,
                padding: Padding::None,
                line_end: LineEnd::Unchanged,
                trimming: Trimming::Unchanged,
                nesting_depth: 0,
            },
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/data/expected/prod_semicolon_on_nl_false"
            ),
        ),
        (
            FmtOptions {
                empty_line_after_prod: false,
                prod_semicolon_on_nl: true,
                max_line_length: 100,
                padding: Padding::None,
                line_end: LineEnd::Unchanged,
                trimming: Trimming::Unchanged,
                nesting_depth: 0,
            },
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/data/expected/empty_line_after_prod_false"
            ),
        ),
    ];

    #[test]
    // #[ignore = "Not ready yet"]
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

    #[allow(clippy::const_is_empty)]
    fn skip_test(file_name: &OsStr) -> bool {
        SKIP_LIST.contains(&file_name.to_str().unwrap())
            || (!SELECTED_TESTS.is_empty()
                && !SELECTED_TESTS.contains(&file_name.to_str().unwrap()))
    }
}
