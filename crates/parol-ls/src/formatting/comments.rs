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
        let comments_before_scanner_str = comments_before_token.handle_comments(options);
        (comments_before_scanner_str, comments)
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
}
