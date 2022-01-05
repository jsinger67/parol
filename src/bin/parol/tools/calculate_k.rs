use miette::{bail, miette, IntoDiagnostic, Result, WrapErr};
use parol::analysis::k_decision::{calculate_k, FirstCache, FollowCache};
use parol::{obtain_grammar_config, MAX_K};

pub fn sub_command() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("calculate_k")
        .about("Calculates the maximum lookahead needed for your grammar, similar to `decidable`.")
        .arg(
            clap::Arg::with_name("grammar_file")
                .short("f")
                .help("The grammar file to use")
                .index(1),
        )
        .arg(
            clap::Arg::with_name("lookahead")
                .short("k")
                .default_value("5")
                .help("The maximum number of lookahead tokens to be used")
                .index(2),
        )
}

pub fn main(args: &clap::ArgMatches) -> Result<()> {
    let file_name = args
        .value_of("grammar_file")
        .ok_or_else(|| miette!("Missing argument <grammar_file>!"))?;

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

    let first_cache = FirstCache::new();
    let follow_cache = FollowCache::new();
    let result = calculate_k(&grammar_config, max_k, &first_cache, &follow_cache);
    println!("{:#?}", result);
    Ok(())
}
