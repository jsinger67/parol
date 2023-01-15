use anyhow::Result;
use std::path::PathBuf;

use parol::{conversions::par::grammar_to_par, left_factor, obtain_grammar_config};

/// Applies the left factoring algorithm on the grammar given.
#[derive(clap::Parser)]
#[clap(name = "left_factor")]
pub struct Args {
    /// The grammar file to use
    #[clap(short = 'f', long = "grammar-file")]
    pub(crate) grammar_file: PathBuf,
}

pub fn main(args: &Args) -> Result<()> {
    let file_name = &args.grammar_file;

    let mut grammar_config = obtain_grammar_config(file_name, false)?;
    let cfg = left_factor(&grammar_config.cfg);

    // Exchange original grammar with transformed one
    grammar_config.update_cfg(cfg);

    println!(
        "{}",
        grammar_to_par::render_par_string(&grammar_config, true)?
    );
    Ok(())
}
