use super::super::{
    comments::Comments,
    context::{FormatterContext, semicolon_starts_on_new_line},
    indent::Indent,
    line::Line,
    fmt_options::IndentStyle,
};
use super::helpers::{
    comment_opts_force_single_newline, comment_opts_left, format_comments_before_token,
};
use super::traits::Fmt;
use crate::parol_ls_grammar_trait::{Production, ProductionLHS};

pub(super) fn format_production_with_context(
    production: &Production,
    context: &FormatterContext<'_>,
    comments: Comments,
) -> (String, Comments) {
    let (production_l_h_s, comments) =
        format_production_lhs_with_context(&production.production_l_h_s, context, comments);
    let (mut alternations_str, comments) = production.alternations.txt(context.policy(), comments);
    let (mut comments_before_semicolon, comments) = format_comments_before_token(
        comments,
        &production.semicolon,
        &comment_opts_left(context.policy()),
    );

    alternations_str = alternations_str.trim_end().to_owned();
    let context = context.with_semicolon_starts_on_new_line(semicolon_starts_on_new_line(
        context.policy(),
        &comments_before_semicolon,
    ));

    let mut semi_nl_opt = "";
    if context.semicolon_starts_on_new_line() {
        if Line::ends_with_nl(&comments_before_semicolon) {
            comments_before_semicolon.clone_from(&comments_before_semicolon.trim_end().to_owned());
        }
        semi_nl_opt = "\n    ";
    }

    let prod_nl_opt = if context.empty_line_after_prod() {
        "\n"
    } else {
        ""
    };

    // Apply spacing before semicolon only when semicolon is on new line
    let semicolon_spacing = if context.semicolon_starts_on_new_line()
        && context.policy().space_before_semicolon
    {
        " "
    } else {
        ""
    };

    (
        format!(
            "{} {}{}{}{}{}{}",
            production_l_h_s,
            alternations_str,
            comments_before_semicolon,
            semi_nl_opt,
            semicolon_spacing,
            production.semicolon,
            prod_nl_opt,
        ),
        comments,
    )
}

pub(super) fn format_production_lhs_with_context(
    production_lhs: &ProductionLHS,
    context: &FormatterContext<'_>,
    comments: Comments,
) -> (String, Comments) {
    let (comments_before_non_terminal, comments) = format_comments_before_token(
        comments,
        &production_lhs.identifier.identifier,
        &comment_opts_force_single_newline(context.policy()),
    );
    let (comments_before_colon, comments) = format_comments_before_token(
        comments,
        &production_lhs.colon,
        &comment_opts_left(context.policy()),
    );

    let indent_style = &context.policy().indent_style;
    let indent_size = match indent_style {
        IndentStyle::Spaces(n) => *n as usize,
        IndentStyle::Tabs => 4,
    };

    if production_lhs.identifier.identifier.text().len() + comments_before_colon.len() <= indent_size {
        let padding = " ".repeat(indent_size - production_lhs.identifier.identifier.text().len());
        (
            format!(
                "\n{}{}{}{}{}",
                comments_before_non_terminal,
                production_lhs.identifier.identifier,
                padding,
                comments_before_colon,
                production_lhs.colon
            ),
            comments,
        )
    } else {
        let indent = Indent::make_indent(1, indent_style);
        (
            format!(
                "\n{}{}{}\n{}{}",
                comments_before_non_terminal,
                production_lhs.identifier.identifier,
                comments_before_colon,
                indent,
                production_lhs.colon
            ),
            comments,
        )
    }
}
