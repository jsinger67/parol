#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate lazy_static;

extern crate parol_runtime;

mod boolean_grammar;
mod boolean_grammar_trait;
mod boolean_parser;
mod errors;

use crate::boolean_grammar::BooleanGrammar;
use crate::boolean_parser::parse;
use crate::errors::*;
use log::debug;
use parol::generate_tree_layout;
use std::env;
use std::fs;

// To generate:
// cargo run --bin parol -- -f ./examples/boolean_parser/boolean-parser.par -e ./examples/boolean_parser/boolean-parser-exp.par -p ./examples/boolean_parser/boolean_parser.rs -a ./examples/boolean_parser/boolean_grammar_trait.rs -t BooleanGrammar -m boolean_grammar

// To run the example
// cargo run --example boolean_parser -- ./examples/boolean_parser/boolean_parser_test.txt

// To activate local logging
// $env:RUST_LOG="boolean_parser::boolean_grammar=trace"

quick_main!(run);

fn run() -> Result<()> {
    env_logger::init();
    debug!("env logger started");

    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        let file_name = args[1].clone();
        let input = fs::read_to_string(file_name.clone())
            .chain_err(|| format!("Can't read file {}", file_name))?;
        let mut boolean_grammar = BooleanGrammar::new();
        let syntax_tree = parse(&input, file_name.to_owned(), &mut boolean_grammar)
            .chain_err(|| format!("Failed parsing file {}", file_name))?;
        println!("{}", boolean_grammar);
        generate_tree_layout(&syntax_tree, &file_name).chain_err(|| "Error generating tree layout")
    } else {
        Err("Please provide a file name as single parameter!".into())
    }
}
