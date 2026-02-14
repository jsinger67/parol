mod declaration_fmt;
mod dispatch;
mod entry_user_fmt;
mod format_impl;
mod grammar_core_fmt;
mod helpers;
mod last_token;
mod production_fmt;
mod prolog_fmt;
mod scalar_fmt;
mod scanner_fmt;
mod scanner_state_fmt;
mod token_expr_fmt;
mod traits;

use traits::Fmt;
pub(crate) use traits::Format;
#[cfg(test)]
mod test;
