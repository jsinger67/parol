use miette::Result;
use std::path::PathBuf;

use parol::conversions::par::render_par_string;
use parol::obtain_grammar_config;

/// Formats the given grammar with the standard format and prints the result to stdout.
#[derive(clap::Parser)]
#[clap(name = "format")]
pub struct Args {
    /// The grammar file to use
    #[clap(short = 'f', long = "grammar-file", parse(from_os_str))]
    grammar_file: PathBuf,
}

pub fn main(args: &Args) -> Result<()> {
    let file_name = &args.grammar_file;

    let grammar_config = obtain_grammar_config(&file_name, false)?;
    println!("{}", render_par_string(&grammar_config, true));
    Ok(())
}
