use anyhow::{bail, Result};
use std::path::PathBuf;

use parol::{conversions::par::grammar_to_par, left_factor, obtain_grammar_config};

/// Applies the left factoring algorithm on the grammar given.
#[derive(clap::Parser)]
#[clap(name = "left_factor")]
pub struct Args {
    /// The grammar file to use
    #[clap(short = 'f', long = "grammar-file")]
    pub(crate) grammar_file: PathBuf,
    /// The output file to write the generated sentence to. If not provided, the sentence will be
    /// printed to stdout.
    #[clap(short = 'o', long = "output-file")]
    output_file: Option<PathBuf>,
}

pub fn main(args: &Args) -> Result<()> {
    let file_name = &args.grammar_file;

    let mut grammar_config = obtain_grammar_config(file_name, false)?;
    if !matches!(grammar_config.grammar_type, parol::parser::GrammarType::LLK) {
        bail!("Only LL grammars are supported for sentence generation");
    }

    let cfg = left_factor(&grammar_config.cfg);

    // Exchange original grammar with transformed one
    grammar_config.update_cfg(cfg);

    let par_string = grammar_to_par::render_par_string(&grammar_config, true)?;

    if let Some(output_file) = &args.output_file {
        std::fs::write(output_file, par_string)?;
    } else {
        println!("{}", par_string);
    }
    Ok(())
}
