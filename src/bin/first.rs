use anyhow::{bail, Result};
use log::debug;
use parol::analysis::{first_k, FirstCache};
use parol::generators::generate_terminal_names;
use parol::{obtain_grammar_config, KTuples, MAX_K};
use std::collections::BTreeMap;
use std::env;

fn main() -> Result<()> {
    env_logger::init();
    // $env:RUST_LOG="parol,parol_runtime=off,productivity=debug"
    debug!("env logger started");

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Missing arguments <par-file> [k=1]!");
        println!(
            "Example:\n\
            cargo run --bin {} ./src/parser/parol-grammar-exp.par",
            module_path!()
        );
    } else {
        let file_name = args[1].clone();
        let grammar_config = obtain_grammar_config(&file_name, false)?;

        let k = if args.len() > 2 {
            args[2]
                .parse::<usize>()
                .expect("Provide a valid integer value for second argument")
        } else {
            1usize
        };

        if k > MAX_K {
            bail!("Maximum lookahead is {}", MAX_K);
        }

        let terminals = generate_terminal_names(&grammar_config);
        let first_cache = FirstCache::new();

        let (first_k_per_prod, mut first_k_per_nt) = first_k(&grammar_config, k, &first_cache);
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
    }
    Ok(())
}
