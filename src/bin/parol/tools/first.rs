use miette::{bail, Result};
use parol::analysis::{first_k, FirstCache};
use parol::generators::generate_terminal_names;
use parol::{obtain_grammar_config, KTuples, MAX_K};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Calculates the FIRST(k) sets for each production and for each non-terminal.
#[derive(clap::Parser)]
#[clap(name = "first")]
pub struct Args {
    /// The grammar file to use
    #[clap(short = 'f', long = "grammar-file", parse(from_os_str))]
    grammar_file: PathBuf,
    /// The maximum number of lookahead tokens to be used
    #[clap(short = 'k', long = "lookahead", default_value = "1")]
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
