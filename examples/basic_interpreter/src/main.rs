#[macro_use]
extern crate derive_builder;
#[macro_use]
extern crate function_name;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate miette;
#[macro_use]
extern crate thiserror;

extern crate parol_runtime;

pub mod basic_grammar;
// The output is version controlled
pub mod basic_grammar_trait;
pub mod basic_parser;
pub mod errors;
pub mod operators;

use crate::basic_grammar::BasicGrammar;
use crate::basic_parser::parse;
use id_tree::Tree;
use id_tree_layout::Layouter;
use log::debug;
use miette::{miette, IntoDiagnostic, Result, WrapErr};
use parol_runtime::parser::ParseTreeType;
use std::env;
use std::fs;
use std::time::Instant;

// To generate:
// parol -f ./basic.par -e ./basic-exp.par -p ./src/basic_parser.rs -a ./src/basic_grammar_trait.rs -t BasicGrammar -m basic_grammar -g

fn main() -> Result<()> {
    env_logger::init();
    debug!("env logger started");

    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 {
        let file_name = args[1].clone();
        let input = fs::read_to_string(file_name.clone())
            .into_diagnostic()
            .wrap_err(format!("Can't read file {}", file_name))?;
        let mut basic_grammar = BasicGrammar::new();
        let now = Instant::now();
        let syntax_tree = parse(&input, &file_name, &mut basic_grammar)
            .wrap_err(format!("Failed parsing file {}", file_name))?;
        let elapsed_time = now.elapsed();
        if args.len() > 2 && args[2] == "-q" {
            println!("\n{}", basic_grammar);
            Ok(())
        } else {
            println!("Parsing took {} milliseconds.", elapsed_time.as_millis());
            println!("Success!\nVariables:\n{}", basic_grammar);
            generate_tree_layout(&syntax_tree, &file_name)
        }
    } else {
        Err(miette!("Please provide a file name as first parameter!"))
    }
}

fn generate_tree_layout(syntax_tree: &Tree<ParseTreeType>, input_file_name: &str) -> Result<()> {
    let mut svg_full_file_name = std::path::PathBuf::from(input_file_name);
    svg_full_file_name.set_extension("svg");

    Layouter::new(syntax_tree)
        .with_file_path(&svg_full_file_name)
        .write()
        .into_diagnostic()
        .wrap_err("Failed writing layout")
}
