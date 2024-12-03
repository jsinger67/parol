extern crate parol_runtime;

mod oberon2_grammar;
// The output is version controlled
mod oberon2_grammar_trait;
mod oberon2_parser;

use crate::oberon2_grammar::Oberon2Grammar;
use crate::oberon2_parser::parse;
use anyhow::{anyhow, Context, Result};
use parol_runtime::log::debug;
use std::env;
use std::fs;
use std::time::Instant;

// To generate:
// parol -f ./oberon2.par -e ./oberon2-exp.par -p ./oberon2_parser.rs -a ./oberon2_grammar_trait.rs -t Oberon2Grammar -m oberon2_grammar

fn main() -> Result<()> {
    env_logger::init();
    debug!("env logger started");

    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 {
        let file_name = args[1].clone();
        let input = fs::read_to_string(file_name.clone())
            .with_context(|| format!("Can't read file {}", file_name))?;
        let mut oberon2_grammar = Oberon2Grammar::new();
        let now = Instant::now();
        parse(&input, &file_name, &mut oberon2_grammar)
            .with_context(|| format!("Failed parsing file {}", file_name))?;
        let elapsed_time = now.elapsed();
        println!("Parsing took {} milliseconds.", elapsed_time.as_millis());
        if args.len() > 2 && args[2] == "-q" {
            Ok(())
        } else {
            println!("Success!\n{}", oberon2_grammar);
            Ok(())
        }
    } else {
        Err(anyhow!("Please provide a file name as first parameter!"))
    }
}
