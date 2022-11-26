mod errors;
mod json_grammar;
// The output is in the $OUT_DIR directory,
// so we have to include!() it
#[cfg(feature = "use-cargo-output")]
mod json_grammar_trait {
    include!(concat!(env!("OUT_DIR"), "/grammar_trait.rs"));
}
#[cfg(feature = "use-cargo-output")]
mod json_parser {
    include!(concat!(env!("OUT_DIR"), "/parser.rs"));
}
// The output is version controlled
#[cfg(not(feature = "use-cargo-output"))]
mod json_grammar_trait;
#[cfg(not(feature = "use-cargo-output"))]
mod json_parser;

use crate::json_grammar::JsonGrammar;
use crate::json_parser::parse;
use id_tree::Tree;
use id_tree_layout::Layouter;
use log::debug;
use miette::{miette, IntoDiagnostic, Result, WrapErr};
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
            .into_diagnostic()
            .wrap_err(format!("Can't read file {}", file_name))?;
        let mut json_grammar = JsonGrammar::new();
        let now = Instant::now();
        let syntax_tree = parse(&input, &file_name, &mut json_grammar)
            .wrap_err(format!("Failed parsing file {}", file_name))?;
        let elapsed_time = now.elapsed();
        println!("Parsing took {} milliseconds.", elapsed_time.as_millis());
        if args.len() > 2 && args[2] == "-q" {
            Ok(())
        } else {
            println!("Success!\n{}", json_grammar);
            generate_tree_layout(&syntax_tree, &file_name)
        }
    } else {
        Err(miette!("Please provide a file name as first parameter!"))
    }
}

fn generate_tree_layout(syntax_tree: &Tree<ParseTreeType>, input_file_name: &str) -> Result<()> {
    let mut svg_full_file_name = std::path::PathBuf::from(input_file_name);
    svg_full_file_name.set_extension("svg");

    Layouter::new(syntax_tree)
        .with_file_path(&svg_full_file_name)
        .write()
        .into_diagnostic()
        .wrap_err("Failed writing layout")
}
