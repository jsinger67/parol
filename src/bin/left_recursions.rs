#[macro_use]
extern crate error_chain;

use parol::errors::*;
use parol::{detect_left_recursions, obtain_cfg_ext};
use std::env;

quick_main!(run);

fn run() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let file_name = args[1].clone();
        let grammar_config = obtain_cfg_ext(&file_name, false)?;
        let recursions = detect_left_recursions(&grammar_config.cfg);
        if recursions.is_empty() {
            println!("No left recursions found!\n");
        } else {
            println!("Found {} left recursions:\n", recursions.len());
            recursions.iter().for_each(|n| {
                let p = n
                    .iter()
                    .map(|s| format!("{}", s))
                    .collect::<Vec<String>>()
                    .join(" => ");
                println!("{}", p);
            });
        }
    } else {
        println!("Missing arguments <par-file>!");
        println!(
            "Example:\n\
            cargo run --bin {} ./src/parser/parol-grammar-exp.par",
            module_path!()
        );
    }
    Ok(())
}
