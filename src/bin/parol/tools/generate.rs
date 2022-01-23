use miette::Result;
use parol::{obtain_grammar_config, LanguageGenerator};
use std::path::PathBuf;

/// Generates a random sentence of the given grammar.
/// It can be used to verify your language description.
#[derive(clap::Parser)]
#[clap(name = "generate")]
pub struct Args {
    /// The grammar file to use
    #[clap(short = 'f', long = "grammar-file", parse(from_os_str))]
    grammar_file: PathBuf,
    /// The maximum length of generated sentence
    #[clap(short = 'l', long = "max-length")]
    max_len: Option<usize>,
}

pub fn sub_command() -> clap::App<'static> {
    clap::App::new("generate")
        .about("Generates a random sentence of the given grammar. It can be used to verify your language description.")
        .arg(
            clap::Arg::new("grammar_file")
                .required(true)
                .short('f')
                .long("grammar-file")
                .takes_value(true)
                .help("The grammar file to use")
        )
        .arg(
            clap::Arg::new("max_len")
                .short('l')
                .long("max-len")
                .takes_value(true)
                .help("The maximum length of generated sentence")
        )
}

pub fn main(args: &Args) -> Result<()> {
    let file_name = &args.grammar_file;

    let grammar_config = obtain_grammar_config(&file_name, false)?;
    let max_sentence_length = args.max_len;
    let mut generator = LanguageGenerator::new(&grammar_config.cfg);
    let result = generator.generate(max_sentence_length)?;
    println!("{}", result);
    Ok(())
}
