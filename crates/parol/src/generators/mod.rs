/// Module with functions to generate variable names and type names
pub mod naming_helper;
pub use naming_helper::NamingHelper;

/// Module with type GrammarConfig
pub mod grammar_config;
pub use grammar_config::GrammarConfig;

/// Module with type ScannerConfig
pub mod scanner_config;
pub use scanner_config::ScannerConfig;

/// Module with grammar transformations
pub mod grammar_trans;
pub use grammar_trans::check_and_transform_grammar;

/// Module that generates type information (AST etc.) for the generated sources
pub mod grammar_type_generator;
pub use grammar_type_generator::GrammarTypeInfo;

/// Module with the language generator
pub mod language_generator;
pub use language_generator::LanguageGenerator;

/// Module with the lexer generator
pub mod lexer_generator;
pub use lexer_generator::{generate_lexer_source, generate_terminal_names};

/// Internal lexer generation IR used by language backends
pub(crate) mod lexer_ir;

/// Internal language backends for lexer generation
pub(crate) mod lexer_backend;

/// Module with the parser generator
pub mod parser_generator;
pub use parser_generator::{
    generate_lalr1_parser_export_model, generate_lalr1_parser_source, generate_parser_export_model,
    generate_parser_export_model_from_grammar, generate_parser_source,
};

/// Internal parser backend context and algorithm dispatch types
pub(crate) mod parser_ir;

/// Internal language-specific parser render helpers
pub(crate) mod parser_render_ir;

/// Internal language-agnostic parser model for portable IR evolution
pub(crate) mod parser_model;
pub use parser_model::{PARSER_EXPORT_MODEL_VERSION, ParserAlgorithmKindModel, ParserExportModel};

/// Internal language backends for parser generation
pub(crate) mod parser_backend;

/// Module with the user-trait generator
pub mod user_trait_generator;
pub use user_trait_generator::UserTraitGenerator;

/// Internal user-trait generation IR used by language backends
pub(crate) mod user_trait_ir;

/// Internal language backends for user-trait generation
pub(crate) mod user_trait_backend;

/// Module with the code formatting function
pub mod rust_code_formatter;
pub use rust_code_formatter::try_format;

mod template_data;

mod symbol_table;
mod symbol_table_facade;

/// Module with the terminal name generator
pub mod terminal_name_generator;
pub use terminal_name_generator::generate_terminal_name;

/// Module with the node kind enum generator
pub mod node_kind_enum_generator;

/// Module with the node types exporter
pub mod export_node_types;

/// C# Lexer generator
pub mod cs_lexer_generator;
/// C# Parser generator
pub mod cs_parser_generator;
/// C# User trait generator
pub mod cs_user_trait_generator;
