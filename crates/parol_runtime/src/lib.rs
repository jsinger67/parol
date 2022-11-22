#[macro_use]
pub extern crate lazy_static;

#[macro_use]
extern crate thiserror;

///
/// Module that provides types for lexical analysis.
///
pub mod lexer;

///
/// Module that provides types for syntactical analysis.
///
pub mod parser;

///
/// error_chain's error module that auto-creates basic error types.
///
pub mod errors;

// re-export
#[cfg(feature = "auto_generation")]
pub use derive_builder;
pub use function_name;
pub use id_tree;
pub use id_tree_layout;
pub use log;
pub use miette;
#[cfg(feature = "auto_generation")]
pub use parol_macros;
