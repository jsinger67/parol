use parol_runtime::lexer::tokenizer::{
    ERROR_TOKEN, NEW_LINE_TOKEN, UNMATCHABLE_TOKEN, WHITESPACE_TOKEN,
};

pub const TERMINALS: &[&str; {{terminal_count}}] = &[
{{{augmented_terminals}}}];

pub const TERMINAL_NAMES: &[&str; {{terminal_count}}] = &[
{{{ terminal_names }}}];
