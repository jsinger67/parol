#[macro_use]
extern crate derive_builder;

#[macro_use]
extern crate lazy_static;

extern crate parol_runtime;

mod basic_grammar;
// The output is version controlled
mod basic_grammar_trait;
mod basic_parser;

use crate::basic_grammar::BasicGrammar;
use crate::basic_parser::parse;
use log::debug;
use miette::{miette, IntoDiagnostic, Result, WrapErr};
use std::env;
use std::fs;
use std::time::Instant;

// To generate:
// parol -f ./basic.par -e ./basic-exp.par -p ./src/basic_parser.rs -a ./src/basic_grammar_trait.rs -t BasicGrammar -m basic_grammar -g

fn main() -> Result<()> {
    env_logger::init();
    debug!("env logger started");

    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 {
        let file_name = args[1].clone();
        let input = fs::read_to_string(file_name.clone())
            .into_diagnostic()
            .wrap_err(format!("Can't read file {}", file_name))?;
        let mut basic_grammar = BasicGrammar::new();
        let now = Instant::now();
        parse(&input, &file_name, &mut basic_grammar)
            .wrap_err(format!("Failed parsing file {}", file_name))?;
        let elapsed_time = now.elapsed();
        println!("Parsing took {} milliseconds.", elapsed_time.as_millis());
        if args.len() > 2 && args[2] == "-q" {
            Ok(())
        } else {
            println!("Success!\n{}", basic_grammar);
            Ok(())
        }
    } else {
        Err(miette!("Please provide a file name as first parameter!"))
    }
}
