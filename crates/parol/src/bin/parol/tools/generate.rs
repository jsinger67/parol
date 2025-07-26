use anyhow::{bail, Result};
use parol::{obtain_grammar_config, LanguageGenerator};
use std::path::PathBuf;

/// Generates a random sentence of the given grammar.
/// It can be used to verify your language description.
#[derive(clap::Parser)]
#[clap(name = "generate")]
pub struct Args {
    /// The grammar file to use
    #[clap(short = 'f', long = "grammar-file")]
    pub(crate) grammar_file: PathBuf,
    /// The maximum length of generated sentence
    #[clap(short = 'l', long = "max-length")]
    max_len: Option<usize>,
    /// The output file to write the generated sentence to. If not provided, the sentence will be
    /// printed to stdout.
    #[clap(short = 'o', long = "output-file")]
    output_file: Option<PathBuf>,
}

pub fn main(args: &Args) -> Result<()> {
    let file_name = &args.grammar_file;

    let grammar_config = obtain_grammar_config(file_name, false)?;
    if !matches!(grammar_config.grammar_type, parol::parser::GrammarType::LLK) {
        bail!("Only LL grammars are supported for sentence generation");
    }

    let max_sentence_length = args.max_len;
    let mut generator = LanguageGenerator::new(&grammar_config.cfg);
    let result = generator.generate(max_sentence_length)?;
    if let Some(output_file) = &args.output_file {
        std::fs::write(output_file, result.as_bytes())?;
    } else {
        println!("{result}");
    }
    Ok(())
}
