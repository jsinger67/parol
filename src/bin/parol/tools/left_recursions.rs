use miette::Result;

use parol::{detect_left_recursions, obtain_grammar_config};

pub fn sub_command() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("left_recursion")
        .about("Checks the given grammar for direct and indirect left recursions.")
        .arg(
            clap::Arg::with_name("grammar_file")
                .required(true)
                .short("f")
                .long("grammar-file")
                .takes_value(true)
                .help("The grammar file to use")
        )
}

pub fn main(args: &clap::ArgMatches) -> Result<()> {
    let file_name = args
        .value_of("grammar_file")
        .unwrap();

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
