extern crate parol_runtime;

mod list_grammar;
mod list_grammar_trait;
mod list_parser;

use crate::{list_grammar::ListGrammar, list_parser::parse};
use anyhow::{Context, Result, anyhow};
use parol::generate_tree_layout;
use parol_runtime::{Report, log::debug};
use std::{env, fs, time::Instant};

// To generate:
// parol -f ./examples/list_lr/list.par -e ./examples/list_lr/list-exp.par -p ./examples/list_lr/list_parser.rs -a ./examples/list_lr/list_grammar_trait.rs -t ListGrammar -m list_grammar -b

// To run the example
// cargo run --example list_lr -- ./examples/list_lr/list_test.txt

struct ErrorReporter;
impl Report for ErrorReporter {}

fn main() -> Result<()> {
    env_logger::init();
    debug!("env logger started");

    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 {
        let file_name = args[1].clone();
        let input = fs::read_to_string(file_name.clone())
            .with_context(|| format!("Can't read file {}", file_name))?;
        let mut list_grammar = ListGrammar::new();
        let now = Instant::now();
        match parse(&input, &file_name, &mut list_grammar) {
            Ok(syntax_tree) => {
                let elapsed_time = now.elapsed();
                if args.len() > 2 && args[2] == "-q" {
                } else {
                    println!("Parsing took {} milliseconds.", elapsed_time.as_millis());
                    println!("Success!\n{}", list_grammar);
                }
                generate_tree_layout(&syntax_tree, &input, &file_name)
                    .context("Error generating tree layout")
            }
            Err(e) => ErrorReporter::report_error(&e, file_name),
        }
    } else {
        Err(anyhow!("Please provide a file name as first parameter!"))
    }
}
