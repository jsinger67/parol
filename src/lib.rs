//#![forbid(missing_docs)]
//!
//! Main module of this crate
//!

use regex::Regex;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate bart_derive;

#[macro_use]
extern crate thiserror;

extern crate rand;
extern crate rand_regex;
extern crate serde;
extern crate serde_json;

#[cfg(feature = "build")]
pub mod build;

///
/// Basic grammar data structures and algorithms
///
pub mod grammar;

pub use grammar::{Cfg, Pos, Pr, Rhs, Symbol, SymbolString, SymbolStrings, Terminal};

///
/// Module with functionalities for grammar analysis
///
pub mod analysis;

pub use analysis::{
    calculate_lookahead_dfas, detect_left_recursions, CompiledTerminal, KTuple, KTuples,
    NtEdgeType, NtGrammarGraph, NtNodeType,
};

///
/// Module with functionalities for grammar conversion
///
pub mod conversions;

pub use conversions::{render_dfa_dot_string, render_nt_dot_string, render_par_string};

///
/// Module with functionalities for lexer and parser generation
///
pub mod generators;
pub use generators::{
    check_and_transform_grammar, generate_lexer_source, generate_parser_source,
    generate_user_trait_source, try_format, GrammarConfig, LanguageGenerator, ScannerConfig,
};

pub mod parser;
pub use parser::{parse, ParolGrammar};

///
/// Module with functionalities for grammar transformation
///
pub mod transformation;
pub use transformation::left_factor;

mod utils;
pub use utils::{generate_tree_layout, obtain_grammar_config};

pub(crate) use utils::str_vec::StrVec;
pub(crate) use utils::{generate_name, group_by};

///
/// Internal lookahead limit
///
pub const MAX_K: usize = 10;

lazy_static! {
    ///
    /// Regex used for portable newline handling
    pub static ref RX_NEWLINE: Regex =
        Regex::new(r"\r\n|\r\n").expect("error parsing regex");
}
