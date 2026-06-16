use anyhow::Result;
use parol::build::Builder;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_builder_add_derives_appends_derives_to_generated_types() -> Result<()> {
    let parser_output = PathBuf::from("tests/output_builder_parser.rs");
    let actions_output = PathBuf::from("tests/output_builder_grammar_trait.rs");

    let _ = fs::remove_file(&parser_output);
    let _ = fs::remove_file(&actions_output);

    Builder::with_explicit_output_dir(".")
        .grammar_file("tests/data/arg_tests/generate.par")
        .parser_output_file(&parser_output)
        .actions_output_file(&actions_output)
        .add_derives(vec![
            "serde::Serialize".to_string(),
            "serde::Deserialize".to_string(),
        ])
        .generate_parser()?;

    let generated = fs::read_to_string(&actions_output)?;
    assert!(generated.contains("#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]"));

    fs::remove_file(parser_output)?;
    fs::remove_file(actions_output)?;

    Ok(())
}
