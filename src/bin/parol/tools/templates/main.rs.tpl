#[macro_use]
extern crate derive_builder;

#[macro_use]
extern crate lazy_static;

extern crate parol_runtime;

mod {{crate_name}}_grammar;
// The output is version controlled
mod {{crate_name}}_grammar_trait;
mod {{crate_name}}_parser;

use crate::{{crate_name}}_grammar::{{grammar_name}}Grammar;
use crate::{{crate_name}}_parser::parse;
use log::debug;
use miette::{miette, IntoDiagnostic, Result, WrapErr};
use std::env;
use std::fs;
use std::time::Instant;

// To generate:
// parol -f ./{{crate_name}}.par -e ./{{crate_name}}-exp.par -p ./src/{{crate_name}}_parser.rs -a ./src/{{crate_name}}_grammar_trait.rs -t {{grammar_name}}Grammar -m {{crate_name}}_grammar -g

fn main() -> Result<()> {
    env_logger::init();
    debug!("env logger started");

    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 {
        let file_name = args[1].clone();
        let input = fs::read_to_string(file_name.clone())
            .into_diagnostic()
            .wrap_err(format!("Can't read file {}", file_name))?;
        let mut {{crate_name}}_grammar = {{grammar_name}}Grammar::new();
        let now = Instant::now();
        parse(&input, &file_name, &mut {{crate_name}}_grammar)
            .wrap_err(format!("Failed parsing file {}", file_name))?;
        let elapsed_time = now.elapsed();
        println!("Parsing took {} milliseconds.", elapsed_time.as_millis());
        if args.len() > 2 && args[2] == "-q" {
            Ok(())
        } else {
            println!("Success!\n{}", {{crate_name}}_grammar);
            Ok(())
        }
    } else {
        Err(miette!("Please provide a file name as first parameter!"))
    }
}
