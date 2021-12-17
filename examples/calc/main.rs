#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate lazy_static;

extern crate parol_runtime;

mod assign_operator;
mod binary_operator;
mod calc_grammar;
mod calc_grammar_trait;
mod calc_parser;
mod errors;
mod unary_operator;

use crate::calc_grammar::CalcGrammar;
use crate::calc_parser::parse;
use crate::errors::*;
use log::debug;
use parol::generate_tree_layout;
use std::env;
use std::fs;

// To generate:
// cargo run --bin parol -- -f .\examples\calc\calc.par -l .\examples\calc\calc-lf.par -p .\examples\calc\calc_parser.rs -a .\examples\calc\calc_grammar_trait.rs -t CalcGrammar -m calc_grammar

// To run the example
// cargo run --example calc -- .\examples\calc\calc_test.txt

quick_main!(run);

fn run() -> Result<()> {
    // $env:RUST_LOG="calc=trace"
    env_logger::init();
    debug!("env logger started");

    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        let file_name = args[1].clone();
        let input = fs::read_to_string(file_name.clone())
            .chain_err(|| format!("Can't read file {}", file_name))?;
        let mut calc_grammar = CalcGrammar::new();
        let syntax_tree = parse(&input, file_name.to_owned(), &mut calc_grammar)
            .chain_err(|| format!("Failed parsing file {}", file_name))?;
        println!("{}", calc_grammar);
        generate_tree_layout(&syntax_tree, &file_name).chain_err(|| "Error generating tree layout")
    } else {
        Err("Please provide a file name as single parameter!".into())
    }
}
