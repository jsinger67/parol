use miette::{miette, Result};

use parol::conversions::par::render_par_string;
use parol::obtain_grammar_config;

pub fn sub_command() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("format")
        .about(
            r"Formats the given grammar with the standard format and prints the result to stdout.",
        )
        .arg(
            clap::Arg::with_name("grammar_file")
                .short("f")
                .help("The grammar file to use")
                .index(1),
        )
}

pub fn main(args: &clap::ArgMatches) -> Result<()> {
    let file_name = args
        .value_of("grammar_file")
        .ok_or_else(|| miette!("Missing argument <grammar_file>!"))?;

    let grammar_config = obtain_grammar_config(&file_name, false)?;
    println!("{}", render_par_string(&grammar_config, true));
    Ok(())
}
