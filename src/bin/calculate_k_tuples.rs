use miette::{bail, Result};
use parol::analysis::k_decision::{calculate_k_tuples, FirstCache, FollowCache};
use parol::generators::generate_terminal_names;
use parol::obtain_grammar_config;
use parol::MAX_K;
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
    } else {
        println!("Missing arguments <par-file> <k>!");
        println!(
            "Example:\n\
            cargo run --bin {} ./src/parser/parol-grammar-exp.par",
            module_path!()
        );
    }
    Ok(())
}
