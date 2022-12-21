use miette::{IntoDiagnostic, Result};
use std::process::{Command, ExitStatus};

macro_rules! parol_path {
    () => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/debug/parol")
    };
}


///
/// Runs the examples as tests.
///
#[test]
fn run_examples_test() -> Result<()> {
    println!("Building examples...");
    build_examples()?;

    let grammar = concat!(env!("CARGO_MANIFEST_DIR"), "/src/parser/parol-grammar.par");
    println!("Running parol on its own parol-grammar {grammar}...");
    run_parol(&["-f", &grammar, "-v"])?;

    println!("Running parol on some example grammars...");
    run_parol_examples()?;
    Ok(())
}

fn build_examples() -> Result<()> {
    Command::new("cargo")
        .args(&["build", "--examples"])
        .status()
        .map(|_| ())
        .into_diagnostic()
}

fn run_parol(args: &[&str]) -> Result<ExitStatus> {
    Command::new(parol_path!())
        .args(args)
        .status()
        .into_diagnostic()
}

fn run_parol_should_fail(args: &[&str]) -> Result<bool> {
    Ok(Command::new(parol_path!())
        .args(args)
        .output()
        .unwrap()
        .status
        .code()
        .unwrap() != 0)
}

fn run_parol_examples() -> Result<()> {
    for entry in std::path::Path::new("./data/valid")
        .read_dir()
        .into_diagnostic()?
    {
        if let Ok(entry) = entry {
            if entry.path().extension().unwrap().to_str().unwrap() == "par" {
                println!("Parsing {}...", entry.path().display());
                run_parol(&["-f", entry.path().to_str().unwrap()])?;
            }
        }
    }
    for entry in std::path::Path::new("./data/invalid")
        .read_dir()
        .into_diagnostic()?
    {
        if let Ok(entry) = entry {
            if entry.path().extension().unwrap().to_str().unwrap() == "par" {
                println!("Parsing {} should fail...", entry.path().display());
                let failed = run_parol_should_fail(&["-f", entry.path().to_str().unwrap()]);
                assert!(failed.ok().unwrap());
            }
        }
    }
    Ok(())
}
