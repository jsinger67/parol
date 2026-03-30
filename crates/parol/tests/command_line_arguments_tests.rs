use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::PredicateBooleanExt;
use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn test_help_argument() {
    cargo_bin_cmd!("parol")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("Usage:"));
}

#[test]
fn test_version_argument() {
    cargo_bin_cmd!("parol")
        .arg("--version")
        .assert()
        .success()
        .stdout(predicates::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_subcommand_generate() {
    let grammar_file = PathBuf::from("tests/data/arg_tests/generate.par");
    let output_file = PathBuf::from("tests/output.rs");

    // Test with missing output file
    cargo_bin_cmd!("parol")
        .args(["generate", "-f", grammar_file.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicates::str::starts_with("Var").and(predicates::str::ends_with("End \n")));

    // Test with output file
    cargo_bin_cmd!("parol")
        .args([
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
    cargo_bin_cmd!("parol")
        .args(["generate", "-f", grammar_file.to_str().unwrap()])
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
    cargo_bin_cmd!("parol")
        .args(["left-factor", "-f", grammar_file.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicates::str::contains("ASuffix"));

    // Test with output file
    let output_file = PathBuf::from("tests/output.par");
    cargo_bin_cmd!("parol")
        .args([
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
    cargo_bin_cmd!("parol")
        .args(["left-factor", "-f", grammar_file.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "Only LL grammars are supported for sentence generation",
        ));
}

fn normalize_newlines(text: &str) -> String {
    text.replace("\r\n", "\n")
}

fn assert_export_snapshot(grammar_file: &Path, expected_file: &Path, output_file: &Path) {
    let _ = fs::remove_file(output_file);

    cargo_bin_cmd!("parol")
        .args([
            "export",
            "-f",
            grammar_file.to_str().unwrap(),
            "--pretty",
            "-o",
            output_file.to_str().unwrap(),
        ])
        .assert()
        .success();

    let expected = fs::read_to_string(expected_file).unwrap();
    let actual = fs::read_to_string(output_file).unwrap();
    assert_eq!(normalize_newlines(&expected), normalize_newlines(&actual));

    fs::remove_file(output_file).unwrap();
}

#[test]
fn test_subcommand_export_llk_snapshot() {
    let grammar_file = PathBuf::from("tests/data/arg_tests/generate.par");
    let expected_file = PathBuf::from("tests/data/arg_tests/export_llk.expected.json");
    let output_file = PathBuf::from("tests/output_export_llk.json");

    assert_export_snapshot(&grammar_file, &expected_file, &output_file);
}

#[test]
fn test_subcommand_export_lalr1_snapshot() {
    let grammar_file = PathBuf::from("tests/data/arg_tests/generate_lr.par");
    let expected_file = PathBuf::from("tests/data/arg_tests/export_lalr1.expected.json");
    let output_file = PathBuf::from("tests/output_export_lalr1.json");

    assert_export_snapshot(&grammar_file, &expected_file, &output_file);
}
