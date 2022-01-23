use miette::{bail, Result};
use parol::analysis::k_decision::{calculate_k, FirstCache, FollowCache};
use parol::{obtain_grammar_config, MAX_K};
use std::path::PathBuf;

/// Calculates the maximum lookahead needed for your grammar, similar to `decidable`.
#[derive(clap::Parser)]
#[clap(name = "calculate_k")]
pub struct Args {
    /// The grammar file to use
    #[clap(short = 'f', long = "grammar-file", parse(from_os_str))]
    grammar_file: PathBuf,
    /// The maximum number of lookahead tokens to be used
    #[clap(short = 'k', long = "lookahead", default_value = "5")]
    lookahead: usize,
}

pub fn sub_command() -> clap::App<'static> {
    clap::App::new("calculate_k")
        .about("Calculates the maximum lookahead needed for your grammar, similar to `decidable`.")
        .arg(
            clap::Arg::new("grammar_file")
                .required(true)
                .short('f')
                .long("grammar-file")
                .takes_value(true)
                .help("The grammar file to use"),
        )
        .arg(
            clap::Arg::new("lookahead")
                .short('k')
                .long("lookahead")
                .takes_value(true)
                .default_value("5")
                .help("The maximum number of lookahead tokens to be used"),
        )
}

pub fn main(args: &Args) -> Result<()> {
    let file_name = &args.grammar_file;

    let grammar_config = obtain_grammar_config(&file_name, true)?;
    let max_k = args.lookahead;

    if max_k > MAX_K {
        bail!("Maximum lookahead is {}", MAX_K);
    }

    let first_cache = FirstCache::new();
    let follow_cache = FollowCache::new();
    let result = calculate_k(&grammar_config, max_k, &first_cache, &follow_cache);
    println!("{:#?}", result);
    Ok(())
}
