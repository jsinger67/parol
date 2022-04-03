#[macro_use]
extern crate derive_builder;

#[macro_use]
extern crate lazy_static;

extern crate parol_runtime;

mod {{crate_name}}_grammar;
pub use {{crate_name}}_grammar::{{grammar_name}}Grammar;

mod {{crate_name}}_grammar_trait;
pub use {{crate_name}}_grammar_trait::ASTType;

mod {{crate_name}}_parser;
pub use {{crate_name}}_parser::parse;
