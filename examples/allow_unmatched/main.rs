extern crate parol_runtime;

mod allow_unmatched_grammar;
mod allow_unmatched_grammar_trait;
mod allow_unmatched_parser;

use crate::{allow_unmatched_grammar::AllowUnmatchedGrammar, allow_unmatched_parser::parse};
use parol::generate_tree_layout;
use parol_runtime::{Report, log::debug};
use std::process::ExitCode;
use std::{env, fs, time::Instant};

// To generate:
// parol -f ./examples/allow_unmatched/allow_unmatched.par -e ./examples/allow_unmatched/allow_unmatched-exp.par -p ./examples/allow_unmatched/allow_unmatched_parser.rs -a ./examples/allow_unmatched/allow_unmatched_grammar_trait.rs -t AllowUnmatchedGrammar -m allow_unmatched_grammar -b

struct ErrorReporter;
impl Report for ErrorReporter {}

fn main() -> ExitCode {
    env_logger::init();
    debug!("env logger started");

    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 {
        let file_name = args[1].clone();
        let input = match fs::read_to_string(file_name.clone()) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Can't read file {}: {}", file_name, e);
                return ExitCode::FAILURE;
            }
        };
        let mut allow_unmatched_grammar = AllowUnmatchedGrammar::new();
        let now = Instant::now();
        match parse(&input, &file_name, &mut allow_unmatched_grammar) {
            Ok(syntax_tree) => {
                let elapsed_time = now.elapsed();
                if args.len() > 2 && args[2] == "-q" {
                } else {
                    println!("Parsing took {} milliseconds.", elapsed_time.as_millis());
                    println!("Success!\n{}", allow_unmatched_grammar);
                }
                if let Err(e) = generate_tree_layout(&syntax_tree, &input, &file_name) {
                    eprintln!("Error generating tree layout: {}", e);
                    return ExitCode::FAILURE;
                }
                ExitCode::SUCCESS
            }
            Err(e) => {
                if let Err(report_err) = ErrorReporter::report_error(&e, file_name) {
                    eprintln!("Error reporting failed: {report_err}");
                    return ExitCode::FAILURE;
                }
                ExitCode::FAILURE
            }
        }
    } else {
        eprintln!("Please provide a file name as first parameter!");
        ExitCode::FAILURE
    }
}
