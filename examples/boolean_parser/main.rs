extern crate parol_runtime;

mod boolean_grammar;
mod boolean_grammar_trait;
mod boolean_parser;

use crate::boolean_grammar::BooleanGrammar;
use crate::boolean_parser::parse;
use anyhow::{anyhow, Context, Result};
use parol::generate_tree_layout;
use parol_runtime::log::debug;
use std::env;
use std::fs;

// To generate:
// parol -f ./boolean-parser.par -e ./boolean-parser-exp.par -p ./boolean_parser.rs -a ./boolean_grammar_trait.rs -t BooleanGrammar -m boolean_grammar -b

// To run the example
// cargo run --example boolean_parser -- ./examples/boolean_parser/boolean_parser_test.txt

// To activate local logging
// $env:RUST_LOG="boolean_parser::boolean_grammar=trace"

fn main() -> Result<()> {
    env_logger::init();
    debug!("env logger started");

    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        let file_name = args[1].clone();
        let input = fs::read_to_string(file_name.clone())
            .with_context(|| format!("Can't read file {}", file_name))?;
        let mut boolean_grammar = BooleanGrammar::new();
        let syntax_tree = parse(&input, &file_name, &mut boolean_grammar)
            .with_context(|| format!("Failed parsing file {}", file_name))?;
        generate_tree_layout(&syntax_tree, &input, &file_name)
            .context("Error generating tree layout")
    } else {
        Err(anyhow!("Please provide a file name as single parameter!"))
    }
}
