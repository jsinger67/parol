//!
//! Main module of this crate
//!
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

#[macro_use]
extern crate function_name;

#[macro_use]
extern crate derive_builder;

extern crate rand;

#[cfg(feature = "build")]
pub mod build;
pub use build::InnerAttributes;

mod config;
pub use config::{CommonGeneratorConfig, ParserGeneratorConfig, UserTraitGeneratorConfig};

///
/// Basic grammar data structures and algorithms
///
#[forbid(missing_docs)]
pub mod grammar;

pub use grammar::{Cfg, Pos, Pr, Rhs, Symbol, SymbolAttribute, Terminal, TerminalKind};

///
/// Module with functionalities for grammar analysis
///
#[forbid(missing_docs)]
pub mod analysis;

pub use analysis::{
    calculate_lookahead_dfas, detect_left_recursive_non_terminals, CompiledTerminal,
    GrammarAnalysisError, KTuple, KTupleBuilder, KTuples, KTuplesBuilder, LookaheadDFA,
    RecursiveNonTerminal, RelatedHint,
};

///
/// Module with functionalities for grammar conversion
///
#[forbid(missing_docs)]
pub mod conversions;

pub use conversions::{render_dfa_dot_string, render_nt_dot_string, render_par_string};

///
/// Module with functionalities for error reporting
///
#[forbid(missing_docs)]
mod error_report;
pub use error_report::ParolErrorReporter;

///
/// Module with functionalities for lexer and parser generation
///
#[forbid(missing_docs)]
pub mod generators;
pub use generators::{
    check_and_transform_grammar, generate_lexer_source, generate_parser_source, try_format,
    GrammarConfig, GrammarTypeInfo, LanguageGenerator, ScannerConfig, UserTraitGenerator,
};

///
/// Module with parol's parser for input grammars
///
pub mod parser;
pub use parser::{parse, ParolGrammar, ParolParserError};

///
/// Module with test template(s)
///
#[forbid(missing_docs)]
pub mod test_support;

///
/// Module with functionalities for grammar transformation
///
#[forbid(missing_docs)]
pub mod transformation;
pub use transformation::left_factor;

///
/// Module with utility functionalities
///
pub mod utils;
pub(crate) use utils::str_vec::StrVec;
pub(crate) use utils::{generate_name, group_by};
pub use utils::{generate_tree_layout, obtain_grammar_config, obtain_grammar_config_from_string};

///
/// Internal lookahead limit
///
pub const MAX_K: usize = 10;
