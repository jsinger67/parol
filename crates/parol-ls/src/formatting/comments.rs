use std::collections::VecDeque;

use crate::{parol_ls_grammar::OwnedToken, rng::Rng};

use super::{CommentLinePolicy, FmtOptions};

#[derive(Debug, Clone, Default)]
pub(crate) struct Comments {
    pub(crate) comments: VecDeque<OwnedToken>,
}

impl From<OwnedToken> for Comments {
    fn from(value: OwnedToken) -> Self {
        Comments {
            comments: VecDeque::from([value]),
        }
    }
}

impl Comments {
    pub(crate) fn format_comments_before(
        self,
        at: &OwnedToken,
        options: &FmtOptions,
    ) -> (String, Comments) {
        let (comments_before_token, comments) = self.split_comments(at.into());

        // When leading_line_comments_on_own_line is true, separate trailing comments
        // (on the same line as the token) from leading comments (on a different line)
        let (trailing_comments, leading_comments) = if options.leading_line_comments_on_own_line {
            comments_before_token.separate_trailing_from_leading(at.location.end_line)
        } else {
            (comments_before_token, Comments::default())
        };

        // Format trailing comments (keep on same line as the token)
        // Use original padding (typically Padding::Left which adds a space before)
        let trailing_str = trailing_comments.handle_comments(options);

        // Format leading comments (should be on own line before the token)
        // For comments on a different line, prepend a newline (but not if they're at file start)
        let leading_str =
            if !leading_comments.comments.is_empty() && options.leading_line_comments_on_own_line {
                // Check if comments are at file start before consuming them
                let at_file_start = leading_comments
                    .comments
                    .iter()
                    .all(|c| c.location.start_line == 1);
                let formatted = leading_comments.handle_comments(options);
                if formatted.is_empty() {
                    formatted
                } else if !at_file_start {
                    // Prepend a newline to separate from previous content
                    format!("\n{}", formatted)
                } else {
                    // At file start, just use the comment as-is
                    formatted
                }
            } else if !leading_comments.comments.is_empty() {
                leading_comments.handle_comments(options)
            } else {
                String::new()
            };

        // Combine: leading comments first, then trailing comments
        let combined = if leading_str.is_empty() {
            trailing_str
        } else if trailing_str.is_empty() {
            leading_str
        } else {
            format!("{}{}", leading_str, trailing_str)
        };

        (combined, comments)
    }

    // Finds an immediately following comment on the same line
    pub(crate) fn formatted_immediately_following_comment(
        mut self,
        at: &OwnedToken,
        options: &FmtOptions,
    ) -> (String, Comments) {
        let comment_token_number = at.token_number + 1;
        let at_line = at.location.end_line;
        if let Some(pos) = self.comments.iter().position(|t| {
            t.token_number == comment_token_number && t.location.start_line == at_line
        }) {
            let comment_token = self.comments.remove(pos).unwrap();
            let following_comment_token: Comments = comment_token.into();
            let following_comment_str = following_comment_token.handle_comments(options);
            (following_comment_str, self)
        } else {
            (String::default(), self)
        }
    }

    pub(crate) fn handle_comments(self, options: &FmtOptions) -> String {
        let comments_str = self.comments.iter().fold(String::new(), |mut acc, c| {
            acc.push_str(c.text());
            acc
        });
        if comments_str.is_empty() {
            comments_str
        } else {
            let policy = CommentLinePolicy::from_options(options);
            let is_line_comment = self
                .comments
                .iter()
                .last()
                .map(|c| c.is_line_comment())
                .unwrap_or(false);
            let line_end = policy.resolve_line_end(options.line_end, is_line_comment);
            options.apply_line_end_policy(comments_str, line_end)
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.comments.is_empty()
    }

    pub(crate) fn push_back(&mut self, token: OwnedToken) {
        self.comments.push_back(token)
    }

    fn split_comments(mut self, at: Rng) -> (Comments, Comments) {
        let mut left = VecDeque::new();
        while let Some(comment) = self.comments.front() {
            let rng = Into::<Rng>::into(comment);
            if rng.comes_before(&at) {
                left.push_back(self.comments.pop_front().unwrap());
            } else {
                break;
            }
        }
        (Comments { comments: left }, self)
    }

    // Separate comments into trailing (on same line as token) and leading (on different line)
    fn separate_trailing_from_leading(mut self, token_line: u32) -> (Comments, Comments) {
        let mut trailing = VecDeque::new();
        let mut leading = VecDeque::new();

        while let Some(comment) = self.comments.pop_front() {
            if comment.location.start_line == token_line {
                // Comment is on the same line as the token - it's a trailing comment
                trailing.push_back(comment);
            } else {
                // Comment is on a different line - it's a leading comment
                leading.push_back(comment);
            }
        }

        (
            Comments { comments: trailing },
            Comments { comments: leading },
        )
    }
}
