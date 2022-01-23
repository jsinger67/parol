use miette::Result;
use parol::obtain_grammar_config;
use std::path::PathBuf;

/// Serializes a grammar to json format. Seldom to apply.
#[derive(clap::Parser)]
#[clap(name = "serialize")]
pub struct Args {
    /// The grammar file to use
    #[clap(short = 'f', long = "grammar-file", parse(from_os_str))]
    grammar_file: PathBuf,
}

pub fn main(args: &Args) -> Result<()> {
    let file_name = &args.grammar_file;
    let grammar_config = obtain_grammar_config(&file_name, false)?;
    let serialized = serde_json::to_string(&grammar_config).unwrap();
    println!("{}", serialized);
    let cfg_ext1 = serde_json::from_str(&serialized).unwrap();
    assert_eq!(grammar_config, cfg_ext1);
    Ok(())
}
