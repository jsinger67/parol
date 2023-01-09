///
/// Module with types that support the representation of a parser PAR grammar
///  
#[forbid(missing_docs)]
pub mod parol_grammar;
pub use parol_grammar::{
    Alternation, Alternations, Factor, ParolGrammar, ParolGrammarItem, Production,
};

pub mod parol_grammar_trait;

pub mod parol_parser;
pub use parol_parser::parse;

///
/// Conversion [parol_grammar::ParolGrammar] to [crate::generators::GrammarConfig]
///
#[forbid(missing_docs)]
pub mod to_grammar_config;
pub(crate) use to_grammar_config::try_to_convert;
