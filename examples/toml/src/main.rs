extern crate parol_runtime;

mod parol_toml_grammar;
// The output is version controlled
mod parol_toml_grammar_trait;
mod parol_toml_parser;

use crate::parol_toml_grammar::ParolTomlGrammar;
use crate::parol_toml_parser::parse;
use anyhow::{anyhow, Context, Result};
use id_tree::Tree;
use id_tree_layout::Layouter;
use parol_runtime::log::debug;
use parol_runtime::ParseTreeType;
use std::env;
use std::fs;
use std::time::Instant;

// To generate:
// parol -f ./parol_toml.par -e ./parol_toml-exp.par -p ./src/parol_toml_parser.rs -a ./src/parol_toml_grammar_trait.rs -t ParolTomlGrammar -m parol_toml_grammar -g

fn main() -> Result<()> {
    env_logger::init();
    debug!("env logger started");

    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 {
        let file_name = args[1].clone();
        let input = fs::read_to_string(file_name.clone())
            .with_context(|| format!("Can't read file {}", file_name))?;
        let mut parol_toml_grammar = ParolTomlGrammar::new();
        let now = Instant::now();
        let syntax_tree = parse(&input, &file_name, &mut parol_toml_grammar)
            .with_context(|| format!("Failed parsing file {}", file_name))?;
        let elapsed_time = now.elapsed();
        println!("Parsing took {} milliseconds.", elapsed_time.as_millis());
        if args.len() > 2 && args[2] == "-q" {
            Ok(())
        } else {
            generate_tree_layout(&syntax_tree, &file_name)?;
            println!("Success!\n{}", parol_toml_grammar);
            Ok(())
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
        .context("Failed writing layout")
}
