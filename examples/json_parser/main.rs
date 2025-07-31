extern crate parol_runtime;

mod json_grammar;
// The output is version controlled
mod json_grammar_trait;
mod json_parser;

use crate::json_grammar::JsonGrammar;
use crate::json_parser::parse;
use parol_runtime::Report;
use parol_runtime::log::debug;
use std::env;
use std::fs;
use std::process::ExitCode;
use std::time::Instant;

// To generate:
// parol -f ./json.par -e ./json-exp.par -p ./src/json_parser.rs -a ./src/json_grammar_trait.rs -t JsonGrammar -m json_grammar

struct JSONErrorReporter;
impl Report for JSONErrorReporter {}

fn main() -> ExitCode {
    env_logger::init();
    debug!("env logger started");

    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 {
        let file_name = args[1].clone();
        let input = match fs::read_to_string(file_name.clone()) {
            Ok(input) => input,
            Err(_) => {
                println!("Can't read file {}", file_name);
                return ExitCode::FAILURE;
            }
        };
        let mut json_grammar = JsonGrammar::new();
        let now = Instant::now();
        match parse(&input, &file_name, &mut json_grammar) {
            Ok(_) => {
                let elapsed_time = now.elapsed();
                if args.len() > 2 && args[2] == "-q" {
                    ExitCode::SUCCESS
                } else {
                    println!("Parsing took {} milliseconds.", elapsed_time.as_millis());
                    println!("Success!\n{}", json_grammar);
                    ExitCode::SUCCESS
                }
            }
            Err(e) => {
                let _ = JSONErrorReporter::report_error(&e, file_name);
                ExitCode::FAILURE
            }
        }
    } else {
        println!("Please provide a file name as first parameter!");
        ExitCode::FAILURE
    }
}
