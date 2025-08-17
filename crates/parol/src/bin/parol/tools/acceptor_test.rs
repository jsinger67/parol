//! Subcommand: acceptor_test
//! Usage: acceptor_test <grammar-file-path> <test-count> [--max-length <max-length>]

use clap::Parser;
use parol::test_support::acceptor_test;
use std::path::PathBuf;

/// Run acceptor tests for a grammar file.
#[derive(Parser)]
#[clap(name = "acceptor_test")]
pub struct Args {
    /// Path to the grammar file
    #[arg(short = 'f', long, value_name = "GRAMMAR_FILE", required = true)]
    pub grammar_file: PathBuf,

    /// Number of tests to run
    #[arg(short = 'c', long, value_name = "TEST_COUNT", required = true)]
    test_count: usize,

    /// Optional maximum length for generated sources
    #[arg(short = 'l', long, value_name = "MAX_LENGTH")]
    max_length: Option<usize>,
}

pub fn main(args: &Args) -> anyhow::Result<()> {
    // Mirror test_acceptor_test logic: call acceptor_test with error handling
    match std::panic::catch_unwind(|| {
        acceptor_test::acceptor_test(&args.grammar_file, args.test_count, args.max_length)
    }) {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Error running acceptor_test: {:?}", e);
            std::process::exit(1);
        }
    }
}
