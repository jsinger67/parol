use anyhow::{bail, Result};
use parol::analysis::FollowCache;
use parol::analysis::follow_k;
use parol::analysis::FirstCache;
use parol::generators::generate_terminal_names;
use parol::{obtain_grammar_config, MAX_K};
use std::path::PathBuf;

/// Calculates the FOLLOW(k) sets for each non-terminal.
#[derive(clap::Parser)]
#[clap(name = "follow")]
pub struct Args {
    /// The grammar file to use
    #[clap(short = 'f', long = "grammar-file")]
    pub(crate) grammar_file: PathBuf,
    /// The maximum number of lookahead tokens to be used
    #[clap(short = 'k', long = "lookahead", default_value = "1")]
    lookahead: usize,
}

pub fn main(args: &Args) -> Result<()> {
    let file_name = &args.grammar_file;

    let grammar_config = obtain_grammar_config(file_name, true)?;
    let max_k = args.lookahead;

    if max_k > MAX_K {
        bail!("Maximum lookahead is {}", MAX_K);
    }

    let terminals = generate_terminal_names(&grammar_config);
    let non_terminals = grammar_config
        .cfg
        .get_non_terminal_set()
        .iter()
        .cloned()
        .collect::<Vec<String>>();
    let first_cache = FirstCache::new();
    let follow_cache = FollowCache::new();
    let (_, follow_k) = follow_k(&grammar_config, max_k, &first_cache, &follow_cache);
    for (nt_i, fo) in follow_k.iter().enumerate() {
        println!("  {}: {}", non_terminals[nt_i], fo.to_string(&terminals));
    }
    Ok(())
}
