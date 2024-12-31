use anyhow::{bail, Result};
use parol::analysis::{first_k, FirstCache, FirstSet};
use parol::generators::generate_terminal_names;
use parol::{obtain_grammar_config, KTuples, MAX_K};
use std::path::PathBuf;

/// Calculates the FIRST(k) sets for each production and for each non-terminal.
#[derive(clap::Parser)]
#[clap(name = "first")]
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
    let first_cache = FirstCache::new();

    let FirstSet {
        productions: first_k_per_prod,
        non_terminals: mut first_k_per_nt,
    } = first_k(&grammar_config, max_k, &first_cache);

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
    let non_terminals = grammar_config
        .cfg
        .get_non_terminal_set()
        .iter()
        .cloned()
        .collect::<Vec<String>>();
    let first_k_per_nt: Vec<KTuples> = std::mem::take(&mut first_k_per_nt);
    for (nt_i, fi) in first_k_per_nt.iter().enumerate() {
        println!("  {}: {}", non_terminals[nt_i], fi.to_string(&terminals));
    }
    Ok(())
}
