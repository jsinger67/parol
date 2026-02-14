use crate::parol_ls_grammar::OwnedToken;

use super::{Comments, FmtOptions, LineEnd, Padding};

pub(super) fn format_comments_before_token(
    comments: Comments,
    token: &OwnedToken,
    options: &FmtOptions,
) -> (String, Comments) {
    Comments::format_comments_before(comments, token, options)
}

pub(super) fn format_trailing_comment(
    comments: Comments,
    token: &OwnedToken,
    options: &FmtOptions,
) -> (String, Comments) {
    Comments::formatted_immediately_following_comment(comments, token, options)
}

pub(super) fn comment_opts_left(options: &FmtOptions) -> FmtOptions {
    options.clone().with_padding(Padding::Left)
}

pub(super) fn comment_opts_right(options: &FmtOptions) -> FmtOptions {
    options.clone().with_padding(Padding::Right)
}

pub(super) fn comment_opts_force_single_newline(options: &FmtOptions) -> FmtOptions {
    options.clone().with_line_end(LineEnd::ForceSingleNewline)
}

pub(super) fn comment_opts_left_force_remove(options: &FmtOptions) -> FmtOptions {
    comment_opts_left(options).with_line_end(LineEnd::ForceRemove)
}
