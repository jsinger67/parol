use miette::{bail, Result};
use parol::analysis::k_decision::{calculate_k_tuples, FirstCache, FollowCache};
use parol::generators::generate_terminal_names;
use parol::obtain_grammar_config;
use parol::MAX_K;
use std::path::PathBuf;

/// Calculates the lookahead tokens with size k for each non-terminal.
#[derive(clap::Parser)]
#[clap(name = "calculate_k_tuples")]
pub struct Args {
    /// The grammar file to use
    #[clap(short = 'f', long = "grammar-file", parse(from_os_str))]
    grammar_file: PathBuf,
    /// The maximum number of lookahead tokens to be used
    #[clap(short = 'k', long = "lookahead", default_value = "5")]
    lookahead: usize,
}

pub fn main(args: &Args) -> Result<()> {
    let file_name = &args.grammar_file;

    let grammar_config = obtain_grammar_config(&file_name, true)?;
    let max_k = args.lookahead;

    if max_k > MAX_K {
        bail!("Maximum lookahead is {}", MAX_K);
    }

    let terminals = generate_terminal_names(&grammar_config);
    let first_cache = FirstCache::new();
    let follow_cache = FollowCache::new();
    let result = calculate_k_tuples(&grammar_config, max_k, &first_cache, &follow_cache);
    match result {
        Err(err) => println!("Error: {}", err),
        Ok(tuples) => tuples.iter().for_each(|(prod_num, k_tuples)| {
            println!("/* {} */ {}", prod_num, grammar_config.cfg.pr[*prod_num]);
            println!("    {}", k_tuples.to_string(&terminals));
        }),
    }
    Ok(())
}
