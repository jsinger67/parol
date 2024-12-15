extern crate parol_runtime;

mod keywords_grammar;
mod keywords_grammar_trait;
mod keywords_parser;

use crate::keywords_grammar::KeywordsGrammar;
use crate::keywords_parser::parse;
use anyhow::{anyhow, Context, Result};
use parol::generate_tree_layout;
use parol_runtime::log::debug;
use parol_runtime::Report;
use std::env;
use std::fs;

// To generate:
// parol -f ./examples/keywords/keywords.par -e ./examples/keywords/keywords-exp.par -p ./examples/keywords/keywords_parser.rs -a ./examples/keywords/keywords_grammar_trait.rs -t KeywordsGrammar -m keywords_grammar

struct ErrorReporter;
impl Report for ErrorReporter {}

fn main() -> Result<()> {
    env_logger::init();
    debug!("env logger started");

    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        let file_name = args[1].clone();
        let input = fs::read_to_string(file_name.clone())
            .with_context(|| format!("Can't read file {}", file_name))?;
        let mut keywords_grammar = KeywordsGrammar::new();
        match parse(&input, &file_name, &mut keywords_grammar) {
            Ok(syntax_tree) => {
                if args.len() > 2 && args[2] == "-q" {
                    Ok(())
                } else {
                    generate_tree_layout(&syntax_tree, &input, &file_name)
                        .context("Error generating tree layout")?;
                    println!("Success!\n{}", keywords_grammar);
                    Ok(())
                }
            }
            Err(e) => {
                let _ = ErrorReporter::report_error(&e, file_name);
                Err(anyhow!("Parse error!"))
            }
        }
    } else {
        Err(anyhow!("Please provide a file name as single parameter!"))
    }
}
