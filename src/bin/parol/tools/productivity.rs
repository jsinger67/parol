use miette::Result;
use std::path::PathBuf;

use parol::analysis::non_productive_non_terminals;
use parol::obtain_grammar_config;

#[derive(clap::Parser)]
pub struct Args {
    /// The grammar file to use
    #[clap(short = 'f', long = "grammar-file", parse(from_os_str))]
    grammar_file: PathBuf,
}

pub fn main(args: &Args) -> Result<()> {
    let file_name = &args.grammar_file;

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
    Ok(())
}
