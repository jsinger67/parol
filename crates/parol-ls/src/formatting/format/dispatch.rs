use crate::parol_ls_grammar_trait::{ScannerDirectives, Symbol};

use super::super::comments::Comments;
use super::super::context::FormatterContext;
use super::super::fmt_options::FmtOptions;
use super::scanner_fmt::format_scanner_directives_with_context;
use super::traits::Fmt;

pub(super) fn handle_scanner_directives(
    scanner_directives: &ScannerDirectives,
    options: &FmtOptions,
    comments: Comments,
) -> (String, Comments) {
    let context = FormatterContext::new(options);
    format_scanner_directives_with_context(scanner_directives, &context, comments)
}

pub(super) fn handle_symbol(
    symbol: &Symbol,
    options: &FmtOptions,
    comments: Comments,
) -> (String, Comments) {
    match symbol {
        Symbol::NonTerminal(n) => n.non_terminal.txt(options, comments),
        Symbol::SimpleToken(t) => t.simple_token.txt(options, comments),
        Symbol::TokenWithStates(t) => t.token_with_states.txt(options, comments),
    }
}
