use miette::Result;
use std::path::PathBuf;

use parol::{left_factor, obtain_grammar_config};

/// Applies the left factoring algorithm on the grammar given.
#[derive(clap::Parser)]
#[clap(name = "left_factor")]
pub struct Args {
    /// The grammar file to use
    #[clap(short = 'f', long = "grammar-file", parse(from_os_str))]
    grammar_file: PathBuf,
}

pub fn main(args: &Args) -> Result<()> {
    let file_name = &args.grammar_file;

    let mut grammar_config = obtain_grammar_config(&file_name, false)?;
    let cfg = left_factor(&grammar_config.cfg);

    // Exchange original grammar with transformed one
    grammar_config.update_cfg(cfg);
    Ok(())
}
