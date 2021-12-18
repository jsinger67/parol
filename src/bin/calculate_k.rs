use anyhow::{bail, Result};
use parol::analysis::k_decision::{calculate_k, FirstCache, FollowCache};
use parol::{obtain_grammar_config, MAX_K};
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let file_name = args[1].clone();

        let grammar_config = obtain_grammar_config(&file_name, true)?;

        let max_k = if args.len() > 2 {
            args[2]
                .parse::<usize>()
                .expect("Provide a valid integer value for second argument")
        } else {
            5usize
        };
        if max_k > MAX_K {
            bail!("Maximum lookahead is {}", MAX_K);
        }

        let first_cache = FirstCache::new();
        let follow_cache = FollowCache::new();
        let result = calculate_k(&grammar_config, max_k, &first_cache, &follow_cache);
        println!("{:#?}", result);
    } else {
        println!("Missing arguments <par-file> <k=5>!");
        println!(
            "Example:\n\
            cargo run --bin {} ./src/parser/parol-grammar-exp.par",
            module_path!()
        );
    }
    Ok(())
}
