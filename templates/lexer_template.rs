use parol_runtime::lexer::tokenizer::{
    {{{used_token_constants}}}
};

pub const TERMINALS: &[&str; {{terminal_count}}] = &[
{{{augmented_terminals}}}];

pub const TERMINAL_NAMES: &[&str; {{terminal_count}}] = &[
{{{ terminal_names }}}];
