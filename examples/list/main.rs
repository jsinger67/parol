extern crate parol_runtime;

mod list_grammar;
mod list_grammar_trait;
mod list_parser;

use crate::list_grammar::ListGrammar;
use crate::list_parser::parse;
use anyhow::{anyhow, Context, Result};
use parol_runtime::log::debug;
use std::env;
use std::fs;

// To generate:
// parol -f ./examples/list_auto/list.par -e ./examples/list_auto/list-exp.par -p ./examples/list_auto/list_parser.rs -a ./examples/list_auto/list_grammar_trait.rs -t ListGrammar -m list_grammar

// To run the example
// cargo run --example list_auto -- ./examples/list_auto/list_test.txt

fn main() -> Result<()> {
    env_logger::init();
    debug!("env logger started");

    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        let file_name = args[1].clone();
        let input = fs::read_to_string(file_name.clone())
            .with_context(|| format!("Can't read file {}", file_name))?;
        let mut list_grammar = ListGrammar::new();
        let _syntax_tree = parse(&input, &file_name, &mut list_grammar)
            .with_context(|| format!("Failed parsing file {}", file_name))?;
        println!("{}", list_grammar);
        Ok(())
    } else {
        Err(anyhow!("Please provide a file name as single parameter!"))
    }
}
