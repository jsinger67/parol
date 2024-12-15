extern crate parol_runtime;

mod list_grammar;
mod list_grammar_trait;
mod list_parser;

use crate::list_grammar::ListGrammar;
use crate::list_parser::parse;
use anyhow::{anyhow, Context, Result};
use parol::generate_tree_layout;
use parol_runtime::log::debug;
use parol_runtime::Report;
use std::env;
use std::fs;

// To generate:
// parol -f ./examples/list/list.par -e ./examples/list/list-exp.par -p ./examples/list/list_parser.rs -a ./examples/list/list_grammar_trait.rs -t ListGrammar -m list_grammar

// To run the example
// cargo run --example list -- ./examples/list/list_test.txt
struct ListErrorReporter;
impl Report for ListErrorReporter {}

fn main() -> Result<()> {
    env_logger::init();
    debug!("env logger started");

    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        let file_name = args[1].clone();
        let input = fs::read_to_string(file_name.clone())
            .with_context(|| format!("Can't read file {}", file_name))?;
        let mut list_grammar = ListGrammar::new();
        match parse(&input, &file_name, &mut list_grammar) {
            Ok(syntax_tree) => {
                println!("{}", list_grammar);
                let mut s = Vec::new();
                syntree::print::print_with_source(&mut s, &syntax_tree, &input)?;
                println!("{}", String::from_utf8(s)?);
                generate_tree_layout(&syntax_tree, &input, &file_name)
                    .context("Error generating tree layout")
            }
            Err(e) => {
                ListErrorReporter::report_error(&e, file_name).unwrap_or(());
                Err(anyhow!("Parsing failed!"))
            }
        }
    } else {
        Err(anyhow!("Please provide a file name as single parameter!"))
    }
}
