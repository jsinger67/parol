use miette::Result;
use parol::obtain_grammar_config;

pub fn sub_command() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("serialize")
        .about("Serializes a grammar to json format. Seldom to apply.")
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
    let serialized = serde_json::to_string(&grammar_config).unwrap();
    println!("{}", serialized);
    let cfg_ext1 = serde_json::from_str(&serialized).unwrap();
    assert_eq!(grammar_config, cfg_ext1);
    Ok(())
}
