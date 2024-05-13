extern crate parol_runtime;

mod list_grammar;
mod list_grammar_trait;
mod list_parser;

use crate::list_grammar::ListGrammar;
use crate::list_parser::parse;
use anyhow::{anyhow, Context, Result};
use parol::generate_tree_layout;
use parol_runtime::log::debug;
use std::env;
use std::fs;
use std::time::Instant;

// To generate:
// parol -f ./list.par -e ./list-exp.par -p ./list_parser.rs -a ./list_grammar_trait.rs -t ListGrammar -m list_grammar

// To run the example
// cargo run --example list -- ./examples/list/list_test.txt

fn main() -> Result<()> {
    env_logger::init();
    debug!("env logger started");

    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        let file_name = args[1].clone();
        let input = fs::read_to_string(file_name.clone())
            .with_context(|| format!("Can't read file {}", file_name))?;
        let mut list_grammar = ListGrammar::new();
        let now = Instant::now();
        let syntax_tree = parse(&input, &file_name, &mut list_grammar)
            .with_context(|| format!("Failed parsing file {}", file_name))?;
        let elapsed_time = now.elapsed();
        println!("Parsing took {} milliseconds.", elapsed_time.as_millis());
        println!("{}", list_grammar);
        generate_tree_layout(&syntax_tree, &file_name).context("Error generating tree layout")
    } else {
        Err(anyhow!("Please provide a file name as single parameter!"))
    }
}
