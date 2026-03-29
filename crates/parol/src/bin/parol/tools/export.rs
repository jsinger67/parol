use anyhow::Result;
use parol::{generate_parser_export_model_from_grammar, obtain_grammar_config};
use std::path::PathBuf;

/// Exports a language-agnostic parser model as JSON.
#[derive(clap::Parser)]
#[clap(name = "export")]
pub struct Args {
    /// The grammar file to analyze
    #[clap(short = 'f', long = "grammar-file")]
    pub(crate) grammar_file: PathBuf,
    /// Lookahead limit used for LL(k) model generation
    #[clap(short = 'k', long = "lookahead", default_value = "5")]
    lookahead: usize,
    /// The output file to write the exported model to. If omitted, JSON is printed to stdout.
    #[clap(short = 'o', long = "output-file")]
    output_file: Option<PathBuf>,
    /// Pretty-print the JSON output.
    #[clap(long = "pretty")]
    pretty: bool,
}

pub fn main(args: &Args) -> Result<()> {
    let grammar_config = obtain_grammar_config(&args.grammar_file, false)?;
    let export_model = generate_parser_export_model_from_grammar(&grammar_config, args.lookahead)?;

    let json = if args.pretty {
        serde_json::to_string_pretty(&export_model)?
    } else {
        serde_json::to_string(&export_model)?
    };

    if let Some(output_file) = &args.output_file {
        std::fs::write(output_file, json.as_bytes())?;
    } else {
        println!("{json}");
    }

    Ok(())
}
