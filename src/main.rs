#[macro_use]
extern crate derive_builder;
#[macro_use]
extern crate function_name;
#[macro_use]
extern crate lazy_static;

extern crate parol_runtime;

mod parol_ls_grammar;
// The output is version controlled
mod parol_ls_grammar_trait;
mod parol_ls_parser;

use crate::parol_ls_grammar::ParolLsGrammar;
use crate::parol_ls_parser::parse;
use log::debug;
use miette::{miette, IntoDiagnostic, Result, WrapErr};
use std::env;
use std::fs;
use std::time::Instant;

// To generate:
// parol -f ./parol_ls.par -e ./parol_ls-exp.par -p ./src/parol_ls_parser.rs -a ./src/parol_ls_grammar_trait.rs -t ParolLsGrammar -m parol_ls_grammar -g

fn main() -> Result<()> {
    env_logger::init();
    debug!("env logger started");

    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 {
        let file_name = args[1].clone();
        let input = fs::read_to_string(file_name.clone())
            .into_diagnostic()
            .wrap_err(format!("Can't read file {}", file_name))?;
        let mut parol_ls_grammar = ParolLsGrammar::new();
        let now = Instant::now();
        parse(&input, &file_name, &mut parol_ls_grammar)
            .wrap_err(format!("Failed parsing file {}", file_name))?;
        let elapsed_time = now.elapsed();
        println!("Parsing took {} milliseconds.", elapsed_time.as_millis());
        if args.len() > 2 && args[2] == "-q" {
            Ok(())
        } else {
            println!("Success!\n{}", parol_ls_grammar);
            Ok(())
        }
    } else {
        Err(miette!("Please provide a file name as first parameter!"))
    }
}

