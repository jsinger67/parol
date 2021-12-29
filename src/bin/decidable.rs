use log::trace;
use miette::{bail, Result};
use parol::analysis::{decidable, explain_conflicts, FirstCache, FollowCache};
use parol::generators::generate_terminal_names;
use parol::obtain_grammar_config;
use parol::MAX_K;
use std::env;

fn main() -> Result<()> {
    env_logger::init();
    trace!("env logger started");

    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let file_name = args[1].clone();

        let grammar_config = obtain_grammar_config(&file_name, false)?;

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
        let mut non_terminals_with_conflicts = vec![];
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
                    non_terminals_with_conflicts.push(n);
                    0
                }
            })
            .max();
        if errors > 0 {
            let terminals = generate_terminal_names(&grammar_config);
            for nt in &non_terminals_with_conflicts {
                println!("Conflicts for non-terminal '{}':", nt);
                let conflicts =
                    explain_conflicts(&grammar_config, nt, max_k, &first_cache, &follow_cache)?;
                for (p1, t1, p2, t2) in conflicts {
                    println!("  Conflict in productions {} and {}:", p1, p2);
                    println!("    {}: {}", p1, t1.to_string(&terminals));
                    println!("    {}: {}", p2, t2.to_string(&terminals));
                    let intersection = t1.intersection(&t2);
                    println!("    ∩: {}\n", intersection.to_string(&terminals));
                }
            }
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
