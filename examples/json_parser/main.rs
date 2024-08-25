mod errors;
mod json_grammar;

// The output is version controlled
mod json_grammar_trait;
mod json_parser;

use crate::json_grammar::JsonGrammar;
use crate::json_parser::parse;
use anyhow::{Context, Result};
use parol_runtime::{log::debug, ParseTree, Report};
use std::{env, fs, process::ExitCode, time::Instant};
use syntree_layout::Layouter;

// To generate:
// parol -f ./json.par -e ./json-exp.par -p ./json_parser.rs -a ./json_grammar_trait.rs -t JsonGrammar -m json_grammar

fn main() -> ExitCode {
    env_logger::init();
    debug!("env logger started");

    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 {
        let file_name = args[1].clone();
        let input = match fs::read_to_string(file_name.clone()) {
            Ok(input) => input,
            Err(_) => {
                println!("Can't read file {}", file_name);
                return ExitCode::FAILURE;
            }
        };
        let mut json_grammar = JsonGrammar::new();
        let now = Instant::now();
        match parse(&input, &file_name, &mut json_grammar) {
            Ok(syntax_tree) => {
                let elapsed_time = now.elapsed();
                println!("Parsing took {} milliseconds.", elapsed_time.as_millis());
                if args.len() > 2 && args[2] == "-q" {
                    ExitCode::SUCCESS
                } else {
                    println!("Parsing took {} milliseconds.", elapsed_time.as_millis());
                    println!("Success!\n{}", json_grammar);
                    match generate_tree_layout(&syntax_tree, &file_name) {
                        Ok(_) => ExitCode::SUCCESS,
                        Err(e) => {
                            eprintln!("Error generating tree layout: {}", e);
                            ExitCode::FAILURE
                        }
                    }
                }
            }
            Err(e) => {
                let _ = errors::JSONErrorReporter::report_error(&e, file_name);
                ExitCode::FAILURE
            }
        }
    } else {
        println!("Please provide a file name as first parameter!");
        ExitCode::FAILURE
    }
}

fn generate_tree_layout(syntax_tree: &ParseTree<'_>, input_file_name: &str) -> Result<()> {
    let mut svg_full_file_name = std::path::PathBuf::from(input_file_name);
    svg_full_file_name.set_extension("svg");

    Layouter::new(syntax_tree)
        .with_file_path(&svg_full_file_name)
        .embed_with(
            |n, f| match n {
                parol_runtime::ParseTreeType::T(t) => write!(f, "{}", t.text()),
                parol_runtime::ParseTreeType::N(n) => write!(f, "{}", n),
            },
            |n| matches!(n, parol_runtime::ParseTreeType::T(_)),
        )?
        .write()
        .context("Failed writing layout")
}
