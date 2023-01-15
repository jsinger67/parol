use anyhow::{anyhow, Result};
use std::process::{Command, ExitStatus};

macro_rules! binary_path {
    ($binary:literal) => {
        format!(
            "{}{}",
            concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/debug/"),
            $binary
        )
    };
}

macro_rules! example_path {
    ($example:literal) => {
        format!(
            "{}{}",
            concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/debug/examples/"),
            $example
        )
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

    println!("Running Calc example...");
    run(
        &example_path!("calc"),
        &[concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/calc/calc_test.txt"
        )],
    )?;

    println!("Running CalcAuto example...");
    run(
        &example_path!("calc_auto"),
        &[concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/calc_auto/calc_test.txt"
        )],
    )?;

    println!("Running List example...");
    run(
        &example_path!("list"),
        &[concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/list/list_test.txt"
        )],
    )?;

    println!("Running ListAuto example...");
    run(
        &example_path!("list_auto"),
        &[concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/list_auto/list_test.txt"
        )],
    )?;

    println!("Running Oberon-0 example...");
    run(
        &example_path!("oberon_0"),
        &[concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/oberon_0/Sample.mod"
        )],
    )?;

    println!("Running Scanner States example...");
    run(
        &example_path!("scanner_states"),
        &[concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/scanner_states/scanner_states_test.txt"
        )],
    )?;

    println!("Running Boolean Parser example...");
    run(
        &example_path!("boolean_parser"),
        &[concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/boolean_parser/boolean_parser_test.txt"
        )],
    )?;

    println!("Running Keywords examples...");
    run_keywords_examples()?;

    println!("Running Keywords2 examples...");
    run_keywords2_examples()?;

    println!("Running Basic Interpreter examples...");
    run_basic_interpreter_examples()?;

    println!("Running JSON Parser examples...");
    run_json_examples()?;

    println!("Running JSON Parser Auto examples...");
    run_json_auto_examples()?;

    Ok(())
}

fn build_examples() -> Result<()> {
    Command::new("cargo")
        .args(&["build", "--examples"])
        .status()
        .map(|_| ())
        .map_err(|e| anyhow!(e))
}

fn run_parol(args: &[&str]) -> Result<ExitStatus> {
    Command::new(binary_path!("parol"))
        .args(args)
        .status()
        .map_err(|e| anyhow!(e))
}

fn run(command: &str, args: &[&str]) -> Result<ExitStatus> {
    println!("Running command {}, {:?}", command, args);
    Command::new(command)
        .args(args)
        .status()
        .map_err(|e| anyhow!(e))
}

fn run_parol_examples() -> Result<()> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/data/valid");
    println!("from folder {path}:");
    for entry in std::path::Path::new(path).read_dir()? {
        if let Ok(entry) = entry {
            if entry.path().extension().unwrap().to_str().unwrap() == "par" {
                println!("Parsing {}...", entry.path().display());
                let exit_status = run_parol(&["-f", entry.path().to_str().unwrap()])?;
                assert!(exit_status.success());
            }
        }
    }
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/data/invalid");
    println!("from folder {path}:");
    for entry in std::path::Path::new(path).read_dir()? {
        if let Ok(entry) = entry {
            if entry.path().extension().unwrap().to_str().unwrap() == "par" {
                println!("Parsing {} should fail...", entry.path().display());
                let exit_status = run_parol(&["-f", entry.path().to_str().unwrap()])?;
                assert!(!exit_status.success());
            }
        }
    }
    Ok(())
}

fn run_keywords_examples() -> Result<()> {
    let parser = example_path!("keywords");
    for entry in std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/keywords/testfiles/valid"
    ))
    .read_dir()?
    {
        if let Ok(entry) = entry {
            if entry.path().extension().unwrap().to_str().unwrap() == "txt" {
                println!("Parsing {}...", entry.path().display());
                let exit_status = run(&parser, &[entry.path().to_str().unwrap()])?;
                assert!(exit_status.success());
            }
        }
    }
    for entry in std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/keywords/testfiles/invalid"
    ))
    .read_dir()?
    {
        if let Ok(entry) = entry {
            if entry.path().extension().unwrap().to_str().unwrap() == "txt" {
                println!("Parsing {} should fail...", entry.path().display());
                let exit_status = run(&parser, &[entry.path().to_str().unwrap()])?;
                assert!(!exit_status.success());
            }
        }
    }
    Ok(())
}

fn run_keywords2_examples() -> Result<()> {
    let parser = example_path!("keywords2");
    for entry in std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/keywords2/testfiles/valid"
    ))
    .read_dir()?
    {
        if let Ok(entry) = entry {
            if entry.path().extension().unwrap().to_str().unwrap() == "txt" {
                println!("Parsing {}...", entry.path().display());
                let exit_status = run(&parser, &[entry.path().to_str().unwrap()])?;
                assert!(exit_status.success());
            }
        }
    }
    Ok(())
}

fn run_basic_interpreter_examples() -> Result<()> {
    let parser = binary_path!("basic");
    for entry in std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/basic_interpreter/tests/data/valid"
    ))
    .read_dir()?
    {
        if let Ok(entry) = entry {
            if entry.path().extension().unwrap().to_str().unwrap() == "bas" {
                println!("Parsing {}...", entry.path().display());
                let exit_status = run(&parser, &[entry.path().to_str().unwrap()])?;
                assert!(exit_status.success());
            }
        }
    }
    for entry in std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/basic_interpreter/tests/data/invalid"
    ))
    .read_dir()?
    {
        if let Ok(entry) = entry {
            if entry.path().extension().unwrap().to_str().unwrap() == "bas" {
                println!("Parsing {} should fail...", entry.path().display());
                let exit_status = run(&parser, &[entry.path().to_str().unwrap()])?;
                assert!(!exit_status.success());
            }
        }
    }
    Ok(())
}

fn run_json_examples() -> Result<()> {
    let parser = binary_path!("json_parser");
    for entry in std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/json_parser/json"
    ))
    .read_dir()?
    {
        if let Ok(entry) = entry {
            if entry.path().extension().unwrap().to_str().unwrap() == "json" {
                println!("Parsing {}...", entry.path().display());
                let exit_status = run(&parser, &[entry.path().to_str().unwrap()])?;
                assert!(exit_status.success());
            }
        }
    }
    Ok(())
}

fn run_json_auto_examples() -> Result<()> {
    let parser = binary_path!("json_parser_auto");
    for entry in std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/json_parser/json"
    ))
    .read_dir()?
    {
        if let Ok(entry) = entry {
            if entry.path().extension().unwrap().to_str().unwrap() == "json" {
                println!("Parsing {}...", entry.path().display());
                let exit_status = run(&parser, &[entry.path().to_str().unwrap()])?;
                assert!(exit_status.success());
            }
        }
    }
    Ok(())
}
