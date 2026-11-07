use std::collections::VecDeque;

use crate::{parol_ls_grammar::OwnedToken, rng::Rng};

use super::{CommentLinePolicy, FmtOptions, indent::Indent};

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
        // When leading_line_comments_on_own_line is enabled, extract same-line trailing comments
        // to ensure they stay with their token instead of being extracted by the next token
        let (comments_before_token, comments) = self.split_comments(at.into());

        // When leading_line_comments_on_own_line is true, separate trailing comments
        // (on the same line as the token) from leading comments (on a different line)
        let (trailing_comments, leading_comments) = if options.leading_line_comments_on_own_line {
            // Separate comments by whether they're on the same line as the token's START
            comments_before_token.separate_by_line(at.location.start_line)
        } else {
            (comments_before_token, Comments::default())
        };

        // Format trailing comments (keep on same line as the token)
        let trailing_str = if options.leading_line_comments_on_own_line {
            // For trailing comments, no padding - keep them directly inline
            let opts_for_trailing = options.clone().with_padding(super::fmt_options::Padding::None);
            trailing_comments.handle_comments_no_line_policy(&opts_for_trailing)
        } else {
            trailing_comments.handle_comments(options)
        };

        // Format leading comments (should be on own line before the token)
        let leading_str = if !leading_comments.comments.is_empty() && options.leading_line_comments_on_own_line {
            let is_line_comment = leading_comments.comments.front()
                .map(|c| c.is_line_comment())
                .unwrap_or(false);
            
            let opts_for_leading = options.clone().with_padding(super::fmt_options::Padding::None);
            let formatted = leading_comments.handle_comments(&opts_for_leading);
            
            if formatted.is_empty() {
                formatted
            } else {
                let indent = Indent::make_indent(options.nesting_depth, &options.indent_style);
                if is_line_comment {
                    format!("\n{}{}", indent, formatted.trim_start())
                } else {
                    if options.nesting_depth > 0 {
                        format!("\n{}{}", indent, formatted.trim_start())
                    } else {
                        format!("{}{}", indent, formatted.trim_start())
                    }
                }
            }
        } else if !leading_comments.comments.is_empty() {
            leading_comments.handle_comments(options)
        } else {
            String::new()
        };

        // Combine: leading first, then trailing
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
        
        // When leading_line_comments_on_own_line is enabled, also extract comments on the same line
        // even if token_number doesn't match, to ensure trailing comments stay inline
        let search_pos = if options.leading_line_comments_on_own_line {
            self.comments.iter().position(|t| {
                (t.token_number == comment_token_number && t.location.start_line == at_line) ||
                (t.location.start_line == at_line)
            })
        } else {
            self.comments.iter().position(|t| {
                t.token_number == comment_token_number && t.location.start_line == at_line
            })
        };
        
        if let Some(pos) = search_pos {
            let comment_token = self.comments.remove(pos).unwrap();
            let following_comment_token: Comments = comment_token.into();
            
            // When leading_line_comments_on_own_line is enabled, don't apply line policies to trailing comments
            let following_comment_str = if options.leading_line_comments_on_own_line {
                following_comment_token.handle_comments_no_line_policy(options)
            } else {
                following_comment_token.handle_comments(options)
            };
            
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

    pub(crate) fn handle_comments_no_line_policy(self, options: &FmtOptions) -> String {
        let comments_str = self.comments.iter().fold(String::new(), |mut acc, c| {
            let text = c.text();
            acc.push_str(text);
            acc
        });
        if comments_str.is_empty() {
            comments_str
        } else {
            // Don't apply the line_comment policy - just use the original line_end
            options.apply_line_end_policy(comments_str, options.line_end)
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
                // Comment comes completely before the token
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

    /// Extract comments on the same line AFTER the token ends.
    /// Used when leading_line_comments_on_own_line is enabled to capture trailing comments.
    fn extract_same_line_after(mut self, at: Rng) -> (Comments, Comments) {
        let mut same_line_after = VecDeque::new();
        let mut rest = VecDeque::new();

        while let Some(comment) = self.comments.pop_front() {
            let rng = Into::<Rng>::into(&comment);
            
            // Check if comment is on same line as token but starts after token ends
            if rng.0.start.line == at.0.start.line && rng.0.start.character > at.0.end.character {
                same_line_after.push_back(comment);
            } else {
                rest.push_back(comment);
            }
        }

        (Comments { comments: same_line_after }, Comments { comments: rest })
    }

    /// Separate comments into those on the same line as the given line number,
    /// and those on different lines.
    fn separate_by_line(mut self, line: u32) -> (Comments, Comments) {
        let mut same_line = VecDeque::new();
        let mut different_line = VecDeque::new();

        while let Some(comment) = self.comments.pop_front() {
            if comment.location.start_line == line {
                same_line.push_back(comment);
            } else {
                different_line.push_back(comment);
            }
        }

        (Comments { comments: same_line }, Comments { comments: different_line })
    }
}
