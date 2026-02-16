use crate::parol_ls_grammar_trait::{
    Declaration, IdentifierList, IdentifierListList, StartDeclaration, UserTypeDeclaration,
    UserTypeName,
};

use super::super::comments::Comments;
use super::super::context::FormatterContext;
use super::super::fmt_options::FmtOptions;
use super::declaration_fmt::format_declaration_with_context;
use super::helpers::{
    comment_opts_force_single_newline, comment_opts_right, format_comments_before_token,
};
use super::traits::Fmt;

impl Fmt for Declaration {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        let context = FormatterContext::new(options);
        format_declaration_with_context(self, &context, comments)
    }
}

impl Fmt for StartDeclaration {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        let (comments_before_start, comments) = format_comments_before_token(
            comments,
            &self.percent_start,
            &comment_opts_force_single_newline(options),
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
        (format!("{identifier}{state_list_list}",), comments)
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
        let (comments_before_string, comments) =
            format_comments_before_token(comments, &self.string, &comment_opts_right(options));
        (
            format!("{}{}", comments_before_string, self.string.text()),
            comments,
        )
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
        (format!("{identifier}{user_type_name_list}",), comments)
    }
}
