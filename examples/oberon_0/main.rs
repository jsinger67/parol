extern crate parol_runtime;

mod oberon_0_grammar;
mod oberon_0_grammar_trait;
mod oberon_0_parser;

use crate::oberon_0_grammar::Oberon0Grammar;
use crate::oberon_0_parser::parse;
use parol::generate_tree_layout;
use parol_runtime::Report;
use parol_runtime::log::debug;
use std::env;
use std::fs;
use std::process::ExitCode;
use std::time::Instant;

// To rebuild the parser sources from scratch use the command build_parsers.ps1

// To run the example
// cargo run --example oberon_0 -- .\examples\oberon_0\Sample.mod

struct Oberon0ErrorReporter;
impl Report for Oberon0ErrorReporter {}

fn main() -> ExitCode {
    // $env:RUST_LOG="parol_runtime=debug,oberon_0=debug"
    env_logger::init();
    debug!("env logger started");

    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        let file_name = args[1].clone();
        let input = match fs::read_to_string(file_name.clone()) {
            Ok(input) => input,
            Err(_) => {
                println!("Can't read file {}", file_name);
                return ExitCode::FAILURE;
            }
        };

        let mut oberon_0_grammar = Oberon0Grammar::new();

        let now = Instant::now();
        match parse(&input, &file_name, &mut oberon_0_grammar) {
            Ok(syntax_tree) => {
                let elapsed_time = now.elapsed();
                if args.len() > 2 && args[2] == "-q" {
                    println!("Parsing took {} milliseconds.", elapsed_time.as_millis());
                    ExitCode::SUCCESS
                } else {
                    println!("Success!\n{}", oberon_0_grammar);
                    println!("Parsing took {} milliseconds.", elapsed_time.as_millis());
                    let _ = generate_tree_layout(&syntax_tree, &input, &file_name);
                    ExitCode::SUCCESS
                }
            }
            Err(e) => {
                let _ = Oberon0ErrorReporter::report_error(&e, file_name);
                ExitCode::FAILURE
            }
        }
    } else {
        println!("Please provide a file name as first parameter!");
        ExitCode::FAILURE
    }
}
