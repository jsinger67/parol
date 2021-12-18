#[macro_use]
extern crate lazy_static;

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
