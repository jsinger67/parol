extern crate parol_runtime;

mod json_grammar;
// The output is version controlled
mod json_grammar_trait;
mod json_parser;

use crate::json_grammar::JsonGrammar;
use crate::json_parser::parse;
use log::debug;
use miette::{miette, IntoDiagnostic, Result, WrapErr};
use std::env;
use std::fs;
use std::time::Instant;

// To generate:
// parol -f ./json.par -e ./json-exp.par -p ./src/json_parser.rs -a ./src/json_grammar_trait.rs -t JsonGrammar -m json_grammar -g

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
        parse(&input, &file_name, &mut json_grammar)
            .wrap_err(format!("Failed parsing file {}", file_name))?;
        let elapsed_time = now.elapsed();
        println!("Parsing took {} milliseconds.", elapsed_time.as_millis());
        if args.len() > 2 && args[2] == "-q" {
            Ok(())
        } else {
            println!("Success!\n{}", json_grammar);
            Ok(())
        }
    } else {
        Err(miette!("Please provide a file name as first parameter!"))
    }
}
