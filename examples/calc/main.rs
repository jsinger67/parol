extern crate parol_runtime;

mod assign_operator;
mod binary_operator;
mod calc_grammar;
mod calc_grammar_trait;
mod calc_parser;
mod errors;

use crate::calc_grammar::CalcGrammar;
use crate::calc_parser::parse;
use anyhow::{anyhow, Context, Result};
use parol_runtime::log::debug;
use std::env;
use std::fs;

fn main() -> Result<()> {
    env_logger::init();
    debug!("env logger started");

    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        let file_name = args[1].clone();
        let input = fs::read_to_string(file_name.clone())
            .with_context(|| format!("Can't read file {}", file_name))?;
        let mut calc_grammar = CalcGrammar::new();
        let _syntax_tree = parse(&input, file_name.clone(), &mut calc_grammar)
            .with_context(|| format!("Failed parsing file {}", file_name))?;
        println!("{}", calc_grammar);
        Ok(())
    } else {
        Err(anyhow!("Please provide a file name as single parameter!"))
    }
}
