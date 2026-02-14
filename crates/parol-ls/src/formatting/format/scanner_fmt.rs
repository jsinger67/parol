use super::helpers::{
    comment_opts_left, comment_opts_left_force_remove, format_comments_before_token,
    format_trailing_comment,
};
use super::last_token::LastToken;
use super::{
    Comments, Fmt, FormatterContext, Indent, ScannerStateDirectives, context_for_scanner_directive,
    scanner_directive_indent,
};
use crate::parol_ls_grammar_trait::ScannerDirectives;

pub(super) fn format_scanner_directives_with_context(
    scanner_directives: &ScannerDirectives,
    context: &FormatterContext<'_>,
    comments: Comments,
) -> (String, Comments) {
    let base_indent = Indent::make_indent(context.policy().nesting_depth);
    match scanner_directives {
        ScannerDirectives::PercentLineUnderscoreCommentTokenLiteral(l) => {
            let (comments_before_token, comments) = format_comments_before_token(
                comments,
                &l.percent_line_underscore_comment,
                &comment_opts_left(context.policy()),
            );
            let context = context_for_scanner_directive(context, &comments_before_token);
            let indent = scanner_directive_indent(&base_indent, &context);
            let (comment_str, comments) = l.token_literal.txt(context.policy(), comments);
            let (following_comment, comments) = format_trailing_comment(
                comments,
                l.token_literal.get_last_token(),
                &comment_opts_left_force_remove(context.policy()),
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
        ScannerDirectives::PercentAllowUnderscoreUnmatched(a) => {
            let (comments_before_token, comments) = format_comments_before_token(
                comments,
                &a.percent_allow_underscore_unmatched,
                &comment_opts_left(context.policy()),
            );
            let context = context_for_scanner_directive(context, &comments_before_token);
            let indent = scanner_directive_indent(&base_indent, &context);
            let (following_comment, comments) = format_trailing_comment(
                comments,
                &a.percent_allow_underscore_unmatched,
                &comment_opts_left_force_remove(context.policy()),
            );
            (
                format!(
                    "{}{}{}{}",
                    comments_before_token,
                    indent,
                    a.percent_allow_underscore_unmatched,
                    following_comment
                ),
                comments,
            )
        }
        ScannerDirectives::PercentBlockUnderscoreCommentTokenLiteralTokenLiteral(b) => {
            let (comments_before_token, comments) = format_comments_before_token(
                comments,
                &b.percent_block_underscore_comment,
                &comment_opts_left(context.policy()),
            );
            let (str1, comments) = b.token_literal.txt(context.policy(), comments);
            let (str2, comments) = b.token_literal0.txt(context.policy(), comments);
            let context = context_for_scanner_directive(context, &comments_before_token);
            let indent = scanner_directive_indent(&base_indent, &context);
            let (following_comment, comments) = format_trailing_comment(
                comments,
                b.token_literal0.get_last_token(),
                &comment_opts_left_force_remove(context.policy()),
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
            let (comments_before_token, comments) = format_comments_before_token(
                comments,
                &n.percent_auto_underscore_newline_underscore_off,
                &comment_opts_left(context.policy()),
            );
            let context = context_for_scanner_directive(context, &comments_before_token);
            let indent = scanner_directive_indent(&base_indent, &context);
            let (following_comment, comments) = format_trailing_comment(
                comments,
                &n.percent_auto_underscore_newline_underscore_off,
                &comment_opts_left_force_remove(context.policy()),
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
            let (comments_before_token, comments) = format_comments_before_token(
                comments,
                &w.percent_auto_underscore_ws_underscore_off,
                &comment_opts_left(context.policy()),
            );
            let context = context_for_scanner_directive(context, &comments_before_token);
            let indent = scanner_directive_indent(&base_indent, &context);
            let (following_comment, comments) = format_trailing_comment(
                comments,
                &w.percent_auto_underscore_ws_underscore_off,
                &comment_opts_left_force_remove(context.policy()),
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
        ScannerDirectives::PercentOnIdentifierListScannerStateDirectives(trans) => {
            let (comments_before_token, comments) = format_comments_before_token(
                comments,
                &trans.percent_on,
                &comment_opts_left(context.policy()),
            );
            let context = context_for_scanner_directive(context, &comments_before_token);
            let indent = scanner_directive_indent(&base_indent, &context);
            let (following_comment, comments) = format_trailing_comment(
                comments,
                trans.get_last_token(),
                &comment_opts_left_force_remove(context.policy()),
            );
            let ident_list = trans
                .identifier_list
                .identifier_list_list
                .iter()
                .fold(
                    vec![
                        trans
                            .identifier_list
                            .identifier
                            .identifier
                            .text()
                            .to_string(),
                    ],
                    |mut acc, i| {
                        acc.push(i.identifier.identifier.text().to_string());
                        acc
                    },
                )
                .join(", ");
            let (text, comments) = match &trans.scanner_state_directives {
                ScannerStateDirectives::PercentEnterIdentifier(
                    scanner_state_directives_percent_enter_identifier,
                ) => {
                    let (ident, comments) = scanner_state_directives_percent_enter_identifier
                        .identifier
                        .txt(context.policy(), comments);
                    (format!("%enter {ident};",), comments)
                }
                ScannerStateDirectives::PercentPushIdentifier(
                    scanner_state_directives_percent_push_identifier,
                ) => {
                    let (ident, comments) = scanner_state_directives_percent_push_identifier
                        .identifier
                        .txt(context.policy(), comments);
                    (format!("%push {ident}",), comments)
                }
                ScannerStateDirectives::PercentPop(_) => ("%pop".to_string(), comments),
            };
            (
                format!(
                    "{}{}{} {} {}{}",
                    comments_before_token,
                    indent,
                    trans.percent_on,
                    ident_list,
                    text,
                    following_comment
                ),
                comments,
            )
        }
    }
}
