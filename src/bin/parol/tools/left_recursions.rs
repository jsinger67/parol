use miette::Result;
use std::path::PathBuf;

use parol::{analysis::left_recursion::detect_left_recursive_non_terminals, obtain_grammar_config};

/// Checks the given grammar for direct and indirect left recursions.
#[derive(clap::Parser)]
#[clap(name = "left_recursion")]
pub struct Args {
    /// The grammar file to use
    #[clap(short = 'f', long = "grammar-file")]
    grammar_file: PathBuf,
}

pub fn main(args: &Args) -> Result<()> {
    let file_name = &args.grammar_file;

    let grammar_config = obtain_grammar_config(&file_name, false)?;
    let recursions = detect_left_recursive_non_terminals(&grammar_config.cfg);
    if recursions.is_empty() {
        println!("No left recursions found!\n");
    } else {
        println!("Found {} left recursive non-terminals:\n", recursions.len());
        recursions.iter().for_each(|n| {
            println!("{}", n);
        });
    }
    Ok(())
}
