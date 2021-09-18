//#![forbid(missing_docs)]
//!
//! Main module of this crate
//!

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate bart_derive;

#[macro_use]
extern crate error_chain;

extern crate serde;
extern crate serde_json;

///
/// error_chains error module that auto-creates basic error types.
///
pub mod errors;

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
    //calculate_k, calculate_k_tuples, calculate_lookahead_dfas, decidable,
    detect_left_recursions,
    CompiledTerminal,
    KTuple,
    KTuples,
    NtEdgeType,
    NtGrammarGraph,
    NtNodeType,
};

///
/// Module with functionalities for grammar conversion
///
pub mod conversions;

///
/// Module with functionalities for lexer and parser generation
///
pub mod generators;
pub use generators::GrammarConfig;

pub mod parser;

///
/// Module with functionalities for grammar transformation
///
pub mod transformation;
pub use transformation::left_factor;

mod utils;
pub use utils::obtain_cfg_ext;

pub(crate) use utils::str_vec::StrVec;
pub(crate) use utils::{generate_name, group_by};

///
/// Internal lookahead limit
///
pub const MAX_K: usize = 10;
