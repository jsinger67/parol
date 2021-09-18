pub mod parol_grammar;
pub use parol_grammar::{
    Alternation, Alternations, Factor, ParolGrammar, ParolGrammarItem, Production,
};

pub mod parol_grammar_trait;

pub mod parol_parser;

pub mod to_grammar_config;
pub use to_grammar_config::try_to_convert;
