use super::super::comments::Comments;
use super::super::context::{FormatterContext, context_for_declaration, declaration_delimiter};
use super::helpers::{
    comment_opts_left, comment_opts_left_force_remove, format_comments_before_token,
    format_trailing_comment,
};
use super::last_token::LastToken;
use super::scanner_fmt::format_scanner_directives_with_context;
use super::traits::Fmt;
use crate::parol_ls_grammar_trait::Declaration;

pub(super) fn format_declaration_with_context(
    declaration: &Declaration,
    context: &FormatterContext<'_>,
    comments: Comments,
) -> (String, Comments) {
    match declaration {
        Declaration::PercentTitleString(title) => {
            let (comments_before_token, comments) = format_comments_before_token(
                comments,
                &title.percent_title,
                &comment_opts_left(context.policy()),
            );
            let context = context_for_declaration(context, &comments_before_token);
            let delim = declaration_delimiter(&context);
            let (str, comments) = title.string.txt(context.policy(), comments);
            (
                format!(
                    "{}{}{} {}",
                    comments_before_token, delim, title.percent_title, str
                ),
                comments,
            )
        }
        Declaration::PercentCommentString(comment) => {
            let (comments_before_token, comments) = format_comments_before_token(
                comments,
                &comment.percent_comment,
                &comment_opts_left(context.policy()),
            );
            let context = context_for_declaration(context, &comments_before_token);
            let delim = declaration_delimiter(&context);
            let (str, comments) = comment.string.txt(context.policy(), comments);
            (
                format!(
                    "{}{}{} {}",
                    comments_before_token, delim, comment.percent_comment, str
                ),
                comments,
            )
        }
        Declaration::PercentUserUnderscoreTypeIdentifierEquUserTypeName(user_type) => {
            let (comments_before_token, comments) = format_comments_before_token(
                comments,
                &user_type.percent_user_underscore_type,
                &comment_opts_left_force_remove(context.policy()),
            );
            let context = context_for_declaration(context, &comments_before_token);
            let delim = declaration_delimiter(&context);
            let (alias_name, comments) = user_type.identifier.txt(context.policy(), comments);
            let (orig_type_name, comments) =
                user_type.user_type_name.txt(context.policy(), comments);
            let (following_comment, comments) = format_trailing_comment(
                comments,
                user_type.user_type_name.get_last_token(),
                &comment_opts_left_force_remove(context.policy()),
            );
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
            format_scanner_directives_with_context(
                &scanner_directives.scanner_directives,
                context,
                comments,
            )
        }
        Declaration::PercentGrammarUnderscoreTypeLiteralString(grammar_type) => {
            let (comments_before_token, comments) = format_comments_before_token(
                comments,
                &grammar_type.percent_grammar_underscore_type,
                &comment_opts_left(context.policy()),
            );
            let context = context_for_declaration(context, &comments_before_token);
            let delim = declaration_delimiter(&context);
            let (str, comments) = grammar_type.literal_string.txt(context.policy(), comments);
            (
                format!(
                    "{}{}{} {}",
                    comments_before_token, delim, grammar_type.percent_grammar_underscore_type, str
                ),
                comments,
            )
        }
        Declaration::PercentNtUnderscoreTypeNtNameEquNtType(nt_type) => {
            let (comments_before_token, comments) = format_comments_before_token(
                comments,
                &nt_type.percent_nt_underscore_type,
                &comment_opts_left_force_remove(context.policy()),
            );
            let context = context_for_declaration(context, &comments_before_token);
            let delim = declaration_delimiter(&context);
            let (prod_name, comments) = nt_type.nt_name.txt(context.policy(), comments);
            let (orig_type_name, comments) = nt_type.nt_type.txt(context.policy(), comments);
            let (following_comment, comments) = format_trailing_comment(
                comments,
                nt_type.nt_type.get_last_token(),
                &comment_opts_left_force_remove(context.policy()),
            );
            (
                format!(
                    "{}{}{} {} {} {}{}",
                    comments_before_token,
                    delim,
                    nt_type.percent_nt_underscore_type,
                    prod_name,
                    nt_type.equ,
                    orig_type_name,
                    following_comment,
                ),
                comments,
            )
        }
        Declaration::PercentTUnderscoreTypeTType(t_type) => {
            let (comments_before_token, comments) = format_comments_before_token(
                comments,
                &t_type.percent_t_underscore_type,
                &comment_opts_left_force_remove(context.policy()),
            );
            let context = context_for_declaration(context, &comments_before_token);
            let delim = declaration_delimiter(&context);
            let (user_type_name, comments) = t_type.t_type.txt(context.policy(), comments);
            let (following_comment, comments) = format_trailing_comment(
                comments,
                t_type.t_type.get_last_token(),
                &comment_opts_left_force_remove(context.policy()),
            );
            (
                format!(
                    "{}{}{} {}{}",
                    comments_before_token,
                    delim,
                    t_type.percent_t_underscore_type,
                    user_type_name,
                    following_comment,
                ),
                comments,
            )
        }
    }
}
