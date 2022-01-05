use miette::{miette, Result};

use parol::{detect_left_recursions, obtain_grammar_config};

pub fn sub_command() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("left_recursion")
        .about("Checks the given grammar for direct and indirect left recursions.")
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
    let recursions = detect_left_recursions(&grammar_config.cfg);
    if recursions.is_empty() {
        println!("No left recursions found!\n");
    } else {
        println!("Found {} left recursions:\n", recursions.len());
        recursions.iter().for_each(|n| {
            let p = n
                .iter()
                .map(|s| format!("{}", s))
                .collect::<Vec<String>>()
                .join(" => ");
            println!("{}", p);
        });
    }
    Ok(())
}
