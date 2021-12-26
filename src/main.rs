#[macro_use]
extern crate lazy_static;

extern crate parol_runtime;

mod json_grammar;
mod json_grammar_trait;
mod json_parser;

use crate::json_grammar::JsonGrammar;
use crate::json_parser::parse;
use anyhow::{anyhow, Context, Result};
use id_tree::Tree;
use id_tree_layout::Layouter;
use log::debug;
use parol_runtime::parser::ParseTreeType;
use std::env;
use std::fs;
use std::time::Instant;

// To generate:
// parol -f ./json.par -e ./json-exp.par -p ./src/json_parser.rs -a ./src/json_grammar_trait.rs -t JsonGrammar -m json_grammar

fn main() -> Result<()> {
    env_logger::init();
    debug!("env logger started");

    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 {
        let file_name = args[1].clone();
        let input = fs::read_to_string(file_name.clone())
            .with_context(|| format!("Can't read file {}", file_name))?;
        let mut json_grammar = JsonGrammar::new();
        let now = Instant::now();
        let syntax_tree = parse(&input, file_name.to_owned(), &mut json_grammar)
            .with_context(|| format!("Failed parsing file {}", file_name))?;
        let elapsed_time = now.elapsed();
        println!("Parsing took {} milliseconds.", elapsed_time.as_millis());
        if args.len() > 2 && args[2] == "-q" {
            Ok(())
        } else {
            println!("Success!\n{}", json_grammar);
            generate_tree_layout(&syntax_tree, &file_name)
        }
    } else {
        Err(anyhow!("Please provide a file name as first parameter!"))
    }
}

fn generate_tree_layout(syntax_tree: &Tree<ParseTreeType>, input_file_name: &str) -> Result<()> {
    let mut svg_full_file_name = std::path::PathBuf::from(input_file_name);
    svg_full_file_name.set_extension("svg");

    Layouter::new(syntax_tree)
        .with_file_path(&svg_full_file_name)
        .write()
        .with_context(|| "Failed writing layout")
}
