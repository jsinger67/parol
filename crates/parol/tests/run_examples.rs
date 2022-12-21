use miette::{IntoDiagnostic, Result};
use std::process::{Command, ExitStatus};

macro_rules! parol_path {
    () => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/debug/parol")
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
    run_example(
        &example_path!("calc"),
        &["../../examples/calc/calc_test.txt"],
    )?;

    println!("Running CalcAuto example...");
    run_example(
        &example_path!("calc_auto"),
        &["../../examples/calc_auto/calc_test.txt"],
    )?;

    println!("Running List example...");
    run_example(
        &example_path!("list"),
        &["../../examples/list/list_test.txt"],
    )?;

    println!("Running ListAuto example...");
    run_example(
        &example_path!("list_auto"),
        &["../../examples/list_auto/list_test.txt"],
    )?;

    println!("Running Oberon-0 example...");
    run_example(
        &example_path!("oberon_0"),
        &["../../examples/oberon_0/Sample.mod"],
    )?;

    println!("Running Scanner States example...");
    run_example(
        &example_path!("scanner_states"),
        &["../../examples/scanner_states/scanner_states_test.txt"],
    )?;

    println!("Running Boolean Parser example...");
    run_example(
        &example_path!("boolean_parser"),
        &["../../examples/boolean_parser/boolean_parser_test.txt"],
    )?;

    println!("Running Keywords example...");
    run_keywords_examples()?;

    println!("Running Keywords2 example...");
    run_keywords2_examples()?;

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
        .unwrap()
        != 0)
}

fn run_example(example: &str, args: &[&str]) -> Result<ExitStatus> {
    println!("Running example {}, {:?}", example, args);
    Command::new(example).args(args).status().into_diagnostic()
}

fn run_example_should_fail(example: &str, args: &[&str]) -> Result<bool> {
    println!("Running example that should fail {}, {:?}", example, args);
    Ok(Command::new(example)
        .args(args)
        .output()
        .unwrap()
        .status
        .code()
        .unwrap()
        != 0)
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

fn run_keywords_examples() -> Result<()> {
    let parser = example_path!("keywords");
    for entry in std::path::Path::new("../../examples/keywords/testfiles/valid")
        .read_dir()
        .into_diagnostic()?
    {
        if let Ok(entry) = entry {
            if entry.path().extension().unwrap().to_str().unwrap() == "txt" {
                println!("Parsing {}...", entry.path().display());
                run_example(&parser, &["-f", entry.path().to_str().unwrap()])?;
            }
        }
    }
    for entry in std::path::Path::new("../../examples/keywords/testfiles/invalid")
        .read_dir()
        .into_diagnostic()?
    {
        if let Ok(entry) = entry {
            if entry.path().extension().unwrap().to_str().unwrap() == "txt" {
                println!("Parsing {} should fail...", entry.path().display());
                let failed =
                    run_example_should_fail(&parser, &["-f", entry.path().to_str().unwrap()]);
                assert!(failed.ok().unwrap());
            }
        }
    }
    Ok(())
}

fn run_keywords2_examples() -> Result<()> {
    let parser = example_path!("keywords2");
    for entry in std::path::Path::new("../../examples/keywords2/testfiles/valid")
        .read_dir()
        .into_diagnostic()?
    {
        if let Ok(entry) = entry {
            if entry.path().extension().unwrap().to_str().unwrap() == "txt" {
                println!("Parsing {}...", entry.path().display());
                run_example(&parser, &["-f", entry.path().to_str().unwrap()])?;
            }
        }
    }
    Ok(())
}
