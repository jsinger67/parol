use miette::{miette, Result};

use parol::{left_factor, obtain_grammar_config};

pub fn sub_command() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("left_factor")
        .about("Applies the left factoring algorithm on the grammar given.")
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

    let mut grammar_config = obtain_grammar_config(&file_name, false)?;
    let cfg = left_factor(&grammar_config.cfg);

    // Exchange original grammar with transformed one
    grammar_config.update_cfg(cfg);
    Ok(())
}
