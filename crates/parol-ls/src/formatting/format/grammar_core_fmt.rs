use crate::{
    parol_ls_grammar_trait::{
        ASTControl, Alternation, AlternationList, Alternations, AlternationsList, CutOperator,
        Factor, GrammarDefinition, GrammarDefinitionList, Group, LookAhead, NonTerminal,
        NonTerminalOpt, Optional, ParolLs, Production, ProductionLHS, Repeat, Symbol,
    },
    utils::RX_NEW_LINE,
};

use super::super::comments::Comments;
use super::super::fmt_options::FmtOptions;
use super::super::{context::FormatterContext, line::Line};
use super::dispatch::handle_symbol;
use super::helpers::{comment_opts_right, format_comments_before_token};
use super::production_fmt::{format_production_lhs_with_context, format_production_with_context};
use super::traits::Fmt;

const START_LINE_OFFSET: usize = 6;

fn normalize_to_single_line(text: &str) -> String {
    text.replace("\n    | ", " | ")
        .replace("\n      ", " ")
        .replace('\n', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn rhs_exceeds_max_line_length(options: &FmtOptions, rhs: &str) -> bool {
    START_LINE_OFFSET + rhs.len() > options.max_line_length
}

impl Fmt for ASTControl {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        match self {
            ASTControl::CutOperator(_) => ("^".to_string(), comments),
            ASTControl::UserTypeDeclaration(ut) => ut.user_type_declaration.txt(options, comments),
            ASTControl::MemberNameASTControlOpt(member_with_user_type_opt) => {
                let (member_name, comments) = member_with_user_type_opt
                    .member_name
                    .identifier
                    .txt(options, comments);
                let (ast_control_opt, comments) = if let Some(ast_control_opt) =
                    member_with_user_type_opt.a_s_t_control_opt.as_ref()
                {
                    ast_control_opt.user_type_declaration.txt(options, comments)
                } else {
                    (String::default(), comments)
                };
                (format!("@{member_name}{ast_control_opt}"), comments)
            }
        }
    }
}

impl Fmt for Alternation {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        let split_top_level_alternatives = options.nesting_depth <= 1;
        let next_option = options.clone().next_depth();
        let (mut alternation_str, comments) = self.alternation_list.iter().fold(
            (String::new(), comments),
            |(mut acc, comments), e| {
                let (mut next_part, comments) = e.txt(&next_option, comments);
                if split_top_level_alternatives && !acc.is_empty() {
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
        if split_top_level_alternatives && !Line::ends_with_nl(&alternation_str) {
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
                let (comments_before_or, comments) =
                    format_comments_before_token(comments, &a.or, &comment_opts_right(options));

                if Line::ends_with_nl(&acc) && !comments_before_or.is_empty() {
                    acc.clone_from(&acc.trim_end().to_owned());
                    acc.push(' ');
                    acc.push_str(&comments_before_or);
                }

                let (alternations_str, comments) = a.txt(options, comments);
                acc.push_str(&alternations_str);
                (acc, comments)
            },
        );
        if options.nesting_depth <= 1
            && !all_alternations_str.contains("//")
            && !all_alternations_str.contains("/*")
        {
            let single_line_rhs = normalize_to_single_line(&all_alternations_str);
            if !rhs_exceeds_max_line_length(options, &single_line_rhs) {
                return (single_line_rhs, comments);
            }
        }
        (all_alternations_str, comments)
    }
}

impl Fmt for AlternationsList {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        let (alternation_str, comments) = self.alternation.txt(options, comments);
        let right_padding = "";
        let left_padding = if options.nesting_depth <= 1 {
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
        let single_line_alternations = normalize_to_single_line(&alternations_str);
        let group_rhs = format!("( {single_line_alternations} )");
        let split_group_alternatives = options.nesting_depth == 2
            && !self.alternations.alternations_list.is_empty()
            && !alternations_str.contains('\n')
            && rhs_exceeds_max_line_length(options, &group_rhs);
        let alternations_str = if split_group_alternatives {
            alternations_str.replace(" | ", "\n    | ")
        } else {
            alternations_str
        };
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

impl Fmt for LookAhead {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        let (look_ahead_group_str, comments) = self.look_ahead_group.txt(options, comments);
        let (token_literal_str, comments) = self.token_literal.txt(options, comments);
        (
            format!("{look_ahead_group_str}{token_literal_str}"),
            comments,
        )
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

        let (comments_before_identifier, comments) = format_comments_before_token(
            comments,
            &self.identifier.identifier,
            &comment_opts_right(options),
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
        let single_line_alternations = normalize_to_single_line(&alternations_str);
        let optional_rhs = format!("[ {single_line_alternations} ]");
        let split_optional_alternatives = options.nesting_depth == 2
            && !self.alternations.alternations_list.is_empty()
            && !alternations_str.contains('\n')
            && rhs_exceeds_max_line_length(options, &optional_rhs);
        let alternations_str = if split_optional_alternatives {
            alternations_str.replace(" | ", "\n    | ")
        } else {
            alternations_str
        };
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
        (format!("{prolog}{nl_opt}{grammar_definition}"), comments)
    }
}

impl Fmt for Production {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        let context = FormatterContext::new(options);
        format_production_with_context(self, &context, comments)
    }
}

impl Fmt for ProductionLHS {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        let context = FormatterContext::new(options);
        format_production_lhs_with_context(self, &context, comments)
    }
}

impl Fmt for Repeat {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        let (alternations_str, comments) = self.alternations.txt(options, comments);
        let single_line_alternations = normalize_to_single_line(&alternations_str);
        let repeat_rhs = format!("{{ {single_line_alternations} }}");
        let split_repeat_alternatives = options.nesting_depth == 2
            && !self.alternations.alternations_list.is_empty()
            && !alternations_str.contains('\n')
            && rhs_exceeds_max_line_length(options, &repeat_rhs);
        let alternations_str = if split_repeat_alternatives {
            alternations_str.replace(" | ", "\n    | ")
        } else {
            alternations_str
        };
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

impl Fmt for Symbol {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        handle_symbol(self, options, comments)
    }
}
