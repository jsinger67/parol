use miette::Result;
use parol::{obtain_grammar_config, LanguageGenerator};

pub fn sub_command() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("generate")
        .about("Generates an arbitrary sentence of the given grammar. It can be used to verify your language description.")
        .arg(
            clap::Arg::with_name("grammar_file")
                .required(true)
                .short("f")
                .long("grammar-file")
                .takes_value(true)
                .help("The grammar file to use")
        )
        .arg(
            clap::Arg::with_name("max_len")
                .short("l")
                .long("max-len")
                .takes_value(true)
                .help("The maximum length of generated sentence")
        )
}

pub fn main(args: &clap::ArgMatches) -> Result<()> {
    let file_name = args
        .value_of("grammar_file")
        .unwrap();

    let grammar_config = obtain_grammar_config(&file_name, false)?;
    let max_sentence_length = args
        .value_of("max-len")
        .map(|v| v.parse::<usize>().ok())
        .flatten();
    let mut generator = LanguageGenerator::new(&grammar_config.cfg);
    let result = generator.generate(max_sentence_length)?;
    println!("{}", result);
    Ok(())
}
