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
use anyhow::{anyhow, Context, Result};
use parol::generate_tree_layout;
use parol_runtime::log::debug;
use std::env;
use std::fs;

// To generate:
// parol -f ./examples/calc/calc.par -e ./examples/calc/calc-exp.par -p ./examples/calc/calc_parser.rs -a ./examples/calc/calc_grammar_trait.rs -t CalcGrammar -m calc_grammar

// To run the example
// cargo run --example calc -- .\examples\calc\calc_test.txt

fn main() -> Result<()> {
    // $env:RUST_LOG="calc=trace"
    env_logger::init();
    debug!("env logger started");

    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        let file_name = args[1].clone();
        let input = fs::read_to_string(file_name.clone())
            .with_context(|| format!("Can't read file {}", file_name))?;
        let mut calc_grammar = CalcGrammar::new();
        let syntax_tree = parse(&input, &file_name, &mut calc_grammar)
            .with_context(|| format!("Failed parsing file {}", file_name))?;
        println!("{}", calc_grammar);
        generate_tree_layout(&syntax_tree, &file_name).context("Error generating tree layout")
    } else {
        Err(anyhow!("Please provide a file name as single parameter!"))
    }
}
