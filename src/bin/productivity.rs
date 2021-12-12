#[macro_use]
extern crate error_chain;

use log::debug;
use parol::analysis::non_productive_non_terminals;
use parol::errors::*;
use parol::obtain_grammar_config;
use std::env;

quick_main!(run);

fn run() -> Result<()> {
    env_logger::init();
    // $env:RUST_LOG="parol,parol_runtime=off,productivity=debug"
    debug!("env logger started");

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Missing arguments <par-file>!");
        println!(
            "Example:\n\
            cargo run --bin {} ./src/parser/parol-grammar-exp.par",
            module_path!()
        );
    } else {
        let file_name = args[1].clone();
        let grammar_config = obtain_grammar_config(&file_name, false)?;

        let non_productive_non_terminals = non_productive_non_terminals(&grammar_config.cfg);
        if non_productive_non_terminals.is_empty() {
            println!("No non-productive non-terminals found!");
        } else {
            println!("Non-productive non-terminals:");
            for nt in non_productive_non_terminals {
                println!("  {}", nt);
            }
        }
    }
    Ok(())
}
