pub mod basic_grammar;
// The output is version controlled
pub mod basic_grammar_trait;
pub mod basic_parser;
pub mod errors;
pub mod operators;

use crate::basic_grammar::BasicGrammar;
use crate::basic_parser::parse;
use crate::errors::basic_error_reporter;
use anyhow::{Context, Result};
use error_report::{ErrorReporter, Report};
use id_tree::Tree;
use id_tree_layout::Layouter;
use parol_runtime::log::debug;
use parol_runtime::parser::ParseTreeType;
use std::env;
use std::fs;
use std::time::Instant;

// To generate:
// parol -f ./basic.par -e ./basic-exp.par -p ./src/basic_parser.rs -a ./src/basic_grammar_trait.rs -t BasicGrammar -m basic_grammar -g

fn main() -> anyhow::Result<std::process::ExitCode> {
    env_logger::init();
    debug!("env logger started");

    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 {
        let file_name = args[1].clone();
        let input = fs::read_to_string(file_name.clone())
            .context(format!("Can't read file {}", file_name))?;
        let mut basic_grammar = BasicGrammar::new();
        let now = Instant::now();
        match parse(&input, &file_name, &mut basic_grammar) {
            Ok(syntax_tree) => {
                let elapsed_time = now.elapsed();
                if args.len() > 2 && args[2] == "-q" {
                    println!("\n{}", basic_grammar);
                    Ok(std::process::ExitCode::SUCCESS)
                } else {
                    println!("Parsing took {} milliseconds.", elapsed_time.as_millis());
                    println!("Success!\nVariables:\n{}", basic_grammar);
                    match generate_tree_layout(&syntax_tree, &file_name) {
                        Ok(_) => (),
                        Err(e) => {
                            println!("Error generating tree layout: {e}");
                            return Ok(std::process::ExitCode::FAILURE);
                        }
                    };
                    Ok(std::process::ExitCode::SUCCESS)
                }
            }
            Err(e) => {
                ErrorReporter::report_error(&e, file_name, Some(&basic_error_reporter))
                    .unwrap_or(());
                Ok(std::process::ExitCode::FAILURE)
            }
        }
    } else {
        println!("Please provide a file name as first parameter!");
        Ok(std::process::ExitCode::FAILURE)
    }
}

fn generate_tree_layout(syntax_tree: &Tree<ParseTreeType>, input_file_name: &str) -> Result<()> {
    let mut svg_full_file_name = std::path::PathBuf::from(input_file_name);
    svg_full_file_name.set_extension("svg");

    Layouter::new(syntax_tree)
        .with_file_path(&svg_full_file_name)
        .write()
        .context("Failed writing layout")
}
