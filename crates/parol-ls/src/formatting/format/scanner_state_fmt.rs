use crate::parol_ls_grammar_trait::{ScannerDirectives, ScannerState, ScannerStateList};

use super::super::comments::Comments;
use super::super::context::FormatterContext;
use super::super::fmt_options::FmtOptions;
use super::dispatch::handle_scanner_directives;
use super::helpers::{comment_opts_leading_own_line, format_comments_before_token};
use super::scanner_fmt::format_scanner_directives_with_context;
use super::traits::Fmt;

impl Fmt for ScannerDirectives {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        let context = FormatterContext::new(options);
        format_scanner_directives_with_context(self, &context, comments)
    }
}

impl Fmt for ScannerState {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        let nl_before_closing_brace = if self.scanner_state_list.is_empty() {
            ""
        } else {
            "\n"
        };

        let (comments_before_scanner, comments) = format_comments_before_token(
            comments,
            &self.percent_scanner,
            &comment_opts_leading_own_line(options),
        );

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

        let brace_sep = if options.scanner_brace_newline {
            "\n"
        } else {
            " "
        };

        let mut result = format!(
            "{}{} {}{}",
            comments_before_scanner, self.percent_scanner, identifier, brace_sep
        );
        result = format!("{}{}", result, self.l_brace);
        result = format!("{}{}", result, scanner_state_list);
        result = format!("{}{}", result, nl_before_closing_brace);

        result = format!("{}{}", result, self.r_brace);

        (result, comments)
    }
}

impl Fmt for ScannerStateList {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        handle_scanner_directives(&self.scanner_directives, options, comments)
    }
}
