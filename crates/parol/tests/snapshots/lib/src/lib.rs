extern crate parol_runtime;

mod snapshot_lib_grammar;
pub use snapshot_lib_grammar::SnapshotLibGrammar;

mod snapshot_lib_grammar_trait;
pub use snapshot_lib_grammar_trait::ASTType;

mod snapshot_lib_parser;
pub use snapshot_lib_parser::parse;
