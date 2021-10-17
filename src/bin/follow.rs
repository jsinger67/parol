#[macro_use]
extern crate error_chain;

use log::debug;
use parol::analysis::follow_k;
use parol::analysis::FirstCache;
use parol::errors::*;
use parol::generators::generate_terminal_names;
use parol::{obtain_cfg_ext, MAX_K};
use std::env;

quick_main!(run);

fn run() -> Result<()> {
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
        let grammar_config = obtain_cfg_ext(&file_name, false)?;

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
        let follow_k = follow_k(&grammar_config, k, &first_cache);
        for (nt, fo) in follow_k.iter() {
            println!("  {}: {}", nt, fo.to_string(&terminals));
        }
    }
    Ok(())
}
