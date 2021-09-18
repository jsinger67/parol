#[macro_use]
extern crate error_chain;

use parol::conversions::par::render_par_string;
use parol::errors::*;
use parol::obtain_cfg_ext;
use std::env;

quick_main!(run);

fn run() -> Result<()> {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let file_name = args[1].clone();
        let grammar_config = obtain_cfg_ext(&file_name, false)?;
        println!("{}", render_par_string(&grammar_config, true));
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
