use miette::{bail, IntoDiagnostic, Result, WrapErr};
use parol::analysis::{first_k, FirstCache};
use parol::generators::generate_terminal_names;
use parol::{obtain_grammar_config, KTuples, MAX_K};
use std::collections::BTreeMap;

pub fn sub_command() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("first")
        .about("Calculates the FIRST(k) sets for each production and for each non-terminal.")
        .arg(
            clap::Arg::with_name("grammar_file")
                .required(true)
                .short("f")
                .long("grammar-file")
                .takes_value(true)
                .help("The grammar file to use")
        )
        .arg(
            clap::Arg::with_name("lookahead")
                .short("k")
                .long("lookahead")
                .takes_value(true)
                .default_value("1")
                .help("The maximum number of lookahead tokens to be used")
        )
}

pub fn main(args: &clap::ArgMatches) -> Result<()> {
    let file_name = args
        .value_of("grammar_file")
        .unwrap();

    let grammar_config = obtain_grammar_config(&file_name, true)?;
    let max_k = args
        .value_of("lookahead")
        .unwrap()
        .parse::<usize>()
        .into_diagnostic()
        .wrap_err("Provide a valid integer value for second argument")?;

    if max_k > MAX_K {
        bail!("Maximum lookahead is {}", MAX_K);
    }

    let terminals = generate_terminal_names(&grammar_config);
    let first_cache = FirstCache::new();

    let (first_k_per_prod, mut first_k_per_nt) = first_k(&grammar_config, max_k, &first_cache);
    println!("Per production:");
    for (i, f) in first_k_per_prod.iter().enumerate() {
        println!(
            "  {}({}): {}",
            i,
            grammar_config.cfg.pr[i].get_n_str(),
            f.to_string(&terminals)
        );
    }
    println!("Per non-terminal:");
    let first_k_per_nt: BTreeMap<String, KTuples> = first_k_per_nt.drain().collect();
    for (nt, fi) in first_k_per_nt.iter() {
        println!("  {}: {}", nt, fi.to_string(&terminals));
    }
    Ok(())
}
