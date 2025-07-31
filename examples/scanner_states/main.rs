extern crate parol_runtime;

mod scanner_states_grammar;
mod scanner_states_grammar_trait;
mod scanner_states_parser;

use crate::scanner_states_grammar::ScannerStatesGrammar;
use crate::scanner_states_parser::parse;
use anyhow::{Context, Result, anyhow};
use parol::generate_tree_layout;
use parol_runtime::Report;
use parol_runtime::log::debug;
use std::env;
use std::fs;

// To generate:
// parol -f ./examples/scanner_states/scanner_states.par -e ./examples/scanner_states/scanner_states-exp.par -p ./examples/scanner_states/scanner_states_parser.rs -a ./examples/scanner_states/scanner_states_grammar_trait.rs -t ScannerStatesGrammar -m scanner_states_grammar

// To run the example
// cargo run --example scanner_states -- ./examples/scanner_states/scanner_states_test.txt

struct ScannerStatesErrorReporter;
impl Report for ScannerStatesErrorReporter {}

fn main() -> Result<()> {
    // $env:RUST_LOG="scanner_states=trace"
    env_logger::init();
    debug!("env logger started");

    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        let file_name = args[1].clone();
        let input = fs::read_to_string(file_name.clone())
            .with_context(|| format!("Can't read file {}", file_name))?;
        let mut scanner_states_grammar = ScannerStatesGrammar::new();
        match parse(&input, &file_name, &mut scanner_states_grammar) {
            Ok(syntax_tree) => {
                println!("{}", scanner_states_grammar);
                generate_tree_layout(&syntax_tree, &input, &file_name)
                    .context("Error generating tree layout")
            }
            Err(e) => {
                ScannerStatesErrorReporter::report_error(&e, file_name).unwrap_or(());
                Err(anyhow!("Parsing failed!"))
            }
        }
    } else {
        Err(anyhow!("Please provide a file name as single parameter!"))
    }
}
