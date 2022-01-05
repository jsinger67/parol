use miette::{miette, Result};

use parol::{obtain_grammar_config, LanguageGenerator};

pub fn sub_command() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("generate")
        .about("Generates an arbitrary sentence of the given grammar. It can be used to verify your language description.")
        .arg(clap::Arg::with_name("grammar_file")
        .short("f")
        .help("The grammar file to use")
        .index(1))
        .arg(clap::Arg::with_name("max_len")
        .short("l")
        .help("The maximum length of generated sentence")
        .index(2))
}

pub fn main(args: &clap::ArgMatches) -> Result<()> {
    let file_name = args
        .value_of("grammar_file")
        .ok_or_else(|| miette!("Missing argument <grammar_file>!"))?;

    let grammar_config = obtain_grammar_config(&file_name, false)?;
    let max_sentence_length = args
        .value_of("lookahead")
        .map(|v| v.parse::<usize>().ok())
        .flatten();
    let mut generator = LanguageGenerator::new(&grammar_config.cfg);
    let result = generator.generate(max_sentence_length)?;
    println!("{}", result);
    Ok(())
}
