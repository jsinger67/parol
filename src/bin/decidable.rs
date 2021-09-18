#[macro_use]
extern crate error_chain;

use log::trace;
use parol::analysis::k_decision::{decidable, FirstCache, FollowCache};
use parol::errors::*;
use parol::obtain_cfg_ext;
use parol::MAX_K;
use std::env;

quick_main!(run);

fn run() -> Result<()> {
    env_logger::init();
    trace!("env logger started");

    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let file_name = args[1].clone();

        let grammar_config = obtain_cfg_ext(&file_name, false)?;

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

        let mut errors = 0;
        let mut non_terminals = vec![];
        let actual_k = grammar_config
            .cfg
            .get_non_terminal_set()
            .iter()
            .map(|n| {
                (
                    n.clone(),
                    decidable(&grammar_config, n, max_k, &first_cache, &follow_cache),
                )
            })
            .map(|(n, r)| {
                if let Ok(r) = r {
                    r
                } else {
                    errors += 1;
                    non_terminals.push(n);
                    0
                }
            })
            .max();
        if errors > 0 {
            println!("{}", non_terminals.join(", "));
            println!("{} undecidable non-terminal(s)", errors);
        } else {
            println!("Grammar is LL{}", actual_k.unwrap());
        }
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
