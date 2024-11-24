use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
use std::path::PathBuf;

#[test]
fn test_help_argument() {
    let mut cmd = Command::cargo_bin("parol").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("Usage:"));
}

#[test]
fn test_version_argument() {
    let mut cmd = Command::cargo_bin("parol").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicates::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_subcommand_generate() {
    let grammar_file = PathBuf::from("tests/data/arg_tests/generate.par");
    let output_file = PathBuf::from("tests/output.rs");

    // Test with missing output file
    let mut cmd = Command::cargo_bin("parol").unwrap();
    cmd.args(["generate", "-f", grammar_file.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicates::str::starts_with("Var").and(predicates::str::ends_with("End \n")));

    // Test with output file
    let mut cmd = Command::cargo_bin("parol").unwrap();
    cmd.args([
        "generate",
        "-f",
        grammar_file.to_str().unwrap(),
        "-o",
        output_file.to_str().unwrap(),
    ])
    .assert()
    .success();

    assert!(output_file.exists());
    std::fs::remove_file(output_file).unwrap();

    // Test with LR grammar as input
    // This subcommand only supports LL grammars
    let grammar_file = PathBuf::from("tests/data/arg_tests/generate_lr.par");
    let mut cmd = Command::cargo_bin("parol").unwrap();
    cmd.args(["generate", "-f", grammar_file.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "Only LL grammars are supported for sentence generation",
        ));
}

#[test]
fn test_subcommand_left_factor() {
    let grammar_file = PathBuf::from("tests/data/arg_tests/left_factor.par");

    // Test with missing output file
    let mut cmd = Command::cargo_bin("parol").unwrap();
    cmd.args(["left-factor", "-f", grammar_file.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicates::str::contains("ASuffix"));

    // Test with output file
    let output_file = PathBuf::from("tests/output.par");
    let mut cmd = Command::cargo_bin("parol").unwrap();
    cmd.args([
        "left-factor",
        "-f",
        grammar_file.to_str().unwrap(),
        "-o",
        output_file.to_str().unwrap(),
    ])
    .assert()
    .success();
    assert!(output_file.exists());
    std::fs::remove_file(output_file).unwrap();

    // Test with LR grammar as input
    // This subcommand only supports LL grammars
    let grammar_file = PathBuf::from("tests/data/arg_tests/left_factor_lr.par");
    let mut cmd = Command::cargo_bin("parol").unwrap();
    cmd.args(["left-factor", "-f", grammar_file.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "Only LL grammars are supported for sentence generation",
        ));
}
