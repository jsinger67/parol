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
use parol_runtime::{log::debug, Report};
use std::{env, fs, time::Instant};

// To generate:
// parol -f ./calc.par -e ./calc-exp.par -p ./calc_parser.rs -a ./calc_grammar_trait.rs -t CalcGrammar -m calc_grammar -b -x

struct ErrorReporter;
impl Report for ErrorReporter {}

fn main() -> Result<()> {
    env_logger::init();
    debug!("env logger started");

    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 {
        let file_name = args[1].clone();
        let input = fs::read_to_string(file_name.clone())
            .with_context(|| format!("Can't read file {}", file_name))?;
        let mut calc_grammar = CalcGrammar::new();
        let now = Instant::now();
        match parse(&input, &file_name, &mut calc_grammar) {
            Ok(_) => {
                let elapsed_time = now.elapsed();
                println!("Parsing took {} milliseconds.", elapsed_time.as_millis());
                if args.len() > 2 && args[2] == "-q" {
                    Ok(())
                } else {
                    println!("Success!\n{}", calc_grammar);
                    Ok(())
                }
            }
            Err(e) => ErrorReporter::report_error(&e, file_name),
        }
    } else {
        Err(anyhow!("Please provide a file name as first parameter!"))
    }
}

#[test]
fn test_parse_as() {
    use crate::calc_grammar_trait::{NonTerminalKind, TerminalKind};
    use crate::calc_parser::parse_as;
    use parol_runtime::parser::parse_tree_type::SynTree2;
    let input = "1 + 2 * 3;";
    let mut calc_grammar = CalcGrammar::new();
    let _syntax_tree = parse_as::<SynTree2<TerminalKind, NonTerminalKind>>(
        &input,
        "test.parol",
        &mut calc_grammar,
    )
    .unwrap();
    println!("{}", calc_grammar);
}
