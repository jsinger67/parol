#[macro_use]
extern crate lazy_static;

extern crate parol_runtime;

mod list_grammar;
mod list_grammar_trait;
mod list_parser;

use crate::list_grammar::ListGrammar;
use crate::list_parser::parse;
use anyhow::{anyhow, Context, Result};
use log::debug;
use parol::generate_tree_layout;
use std::env;
use std::fs;

// To generate:
// cargo run --bin parol -- -f .\examples\list\list.par -l .\examples\list\list-lf.par -p .\examples\list\list_parser.rs -a .\examples\list\list_grammar_trait.rs -t ListGrammar -m list_grammar

// To run the example
// cargo run --example list -- .\examples\list\list_test.txt

fn main() -> Result<()> {
    // $env:RUST_LOG="main=off,parol_runtime=trace,list=debug"
    env_logger::init();
    debug!("env logger started");

    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        let file_name = args[1].clone();
        let input = fs::read_to_string(file_name.clone())
            .with_context(|| format!("Can't read file {}", file_name))?;
        let mut list_grammar = ListGrammar::new();
        let syntax_tree = parse(&input, file_name.to_owned(), &mut list_grammar)
            .with_context(|| format!("Failed parsing file {}", file_name))?;
        println!("{}", list_grammar);
        generate_tree_layout(&syntax_tree, &file_name)
            .with_context(|| "Error generating tree layout")
    } else {
        Err(anyhow!("Please provide a file name as single parameter!"))
    }
}
