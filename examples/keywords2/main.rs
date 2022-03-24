#[macro_use]
extern crate lazy_static;

extern crate parol_runtime;

mod keywords_grammar;
mod keywords_grammar_trait;
mod keywords_parser;

use crate::keywords_grammar::KeywordsGrammar;
use crate::keywords_parser::parse;
use log::debug;
use miette::{miette, IntoDiagnostic, Result, WrapErr};
use parol::generate_tree_layout;
use std::env;
use std::fs;

// To generate:
// parol -f ./examples/keywords2/keywords.par -e ./examples/keywords2/keywords-exp.par -p ./examples/keywords2/keywords_parser.rs -a ./examples/keywords2/keywords_grammar_trait.rs -t KeywordsGrammar -m keywords_grammar

fn main() -> Result<()> {
    env_logger::init();
    debug!("env logger started");

    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        let file_name = args[1].clone();
        let input = fs::read_to_string(file_name.clone())
            .into_diagnostic()
            .wrap_err(format!("Can't read file {}", file_name))?;
        let mut keywords_grammar = KeywordsGrammar::new();
        let syntax_tree = parse(&input, &file_name, &mut keywords_grammar)
            .wrap_err(format!("Failed parsing file {}", file_name))?;
        println!("{}", keywords_grammar);
        generate_tree_layout(&syntax_tree, &file_name).wrap_err("Error generating tree layout")
    } else {
        Err(miette!("Please provide a file name as single parameter!"))
    }
}
