extern crate parol_runtime;

mod snapshot_bin_grammar;
// The output is version controlled
mod snapshot_bin_grammar_trait;
mod snapshot_bin_parser;

use crate::snapshot_bin_grammar::SnapshotBinGrammar;
use crate::snapshot_bin_parser::parse;
use anyhow::{anyhow, Context, Result};
use parol_runtime::log::debug;
use std::env;
use std::fs;
use std::time::Instant;

// To generate:
// parol -f ./snapshot_bin.par -e ./snapshot_bin-exp.par -p ./src/snapshot_bin_parser.rs -a ./src/snapshot_bin_grammar_trait.rs -t SnapshotBinGrammar -m snapshot_bin_grammar -g

fn main() -> Result<()> {
    env_logger::init();
    debug!("env logger started");

    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 {
        let file_name = args[1].clone();
        let input = fs::read_to_string(file_name.clone())
            .with_context(|| format!("Can't read file {}", file_name))?;
        let mut snapshot_bin_grammar = SnapshotBinGrammar::new();
        let now = Instant::now();
        parse(&input, &file_name, &mut snapshot_bin_grammar)
            .with_context(|| format!("Failed parsing file {}", file_name))?;
        let elapsed_time = now.elapsed();
        println!("Parsing took {} milliseconds.", elapsed_time.as_millis());
        if args.len() > 2 && args[2] == "-q" {
            Ok(())
        } else {
            println!("Success!\n{}", snapshot_bin_grammar);
            Ok(())
        }
    } else {
        Err(anyhow!("Please provide a file name as first parameter!"))
    }
}
