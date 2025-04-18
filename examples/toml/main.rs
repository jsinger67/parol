extern crate parol_runtime;

mod parol_toml_grammar;
// The output is version controlled
mod parol_toml_grammar_trait;
mod parol_toml_parser;

use crate::parol_toml_grammar::ParolTomlGrammar;
use crate::parol_toml_parser::parse;
use anyhow::{anyhow, Context, Result};
use parol_runtime::log::debug;
use parol_runtime::ParseTree;
use parol_runtime::Report;
use std::env;
use std::fs;
use std::process::ExitCode;
use std::time::Instant;
use syntree_layout::Layouter;

// To generate:
// parol -f ./parol_toml.par -e ./parol_toml-exp.par -p ./src/parol_toml_parser.rs -a ./src/parol_toml_grammar_trait.rs -t ParolTomlGrammar -m parol_toml_grammar
pub struct TomlErrorReporter {}
impl Report for TomlErrorReporter {}

fn main() -> Result<ExitCode> {
    env_logger::init();
    debug!("env logger started");

    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 {
        let file_name = args[1].clone();
        let input = fs::read_to_string(file_name.clone())
            .with_context(|| format!("Can't read file {}", file_name))?;
        let mut parol_toml_grammar = ParolTomlGrammar::new();
        let now = Instant::now();
        match parse(&input, &file_name, &mut parol_toml_grammar) {
            Ok(syntax_tree) => {
                let elapsed_time = now.elapsed();
                if args.len() > 2 && args[2] == "-q" {
                    Ok(ExitCode::SUCCESS)
                } else {
                    println!("Parsing took {} milliseconds.", elapsed_time.as_millis());
                    println!("Success!\n{}", parol_toml_grammar);
                    match generate_tree_layout(&syntax_tree, &input, &file_name) {
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
                TomlErrorReporter::report_error(&e, file_name).unwrap_or(());
                Ok(std::process::ExitCode::FAILURE)
            }
        }
    } else {
        Err(anyhow!("Please provide a file name as first parameter!"))
    }
}

fn generate_tree_layout(syntax_tree: &ParseTree, input: &str, input_file_name: &str) -> Result<()> {
    let mut svg_full_file_name = std::path::PathBuf::from(input_file_name);
    svg_full_file_name.set_extension("svg");

    Layouter::new(syntax_tree)
        .with_file_path(&svg_full_file_name)
        .embed_with_source_and_display(input)?
        .write()
        .context("Failed writing layout")
}
