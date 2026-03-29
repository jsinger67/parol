use anyhow::Result;
use parol::generators::ParserAlgorithmKindModel;
use parol::{
    calculate_lalr1_parse_table, calculate_lookahead_dfas, generate_lalr1_parser_export_model,
    generate_parser_export_model, generate_parser_export_model_from_grammar, obtain_grammar_config,
};
use std::path::PathBuf;

fn arg_test_grammar(file_name: &str) -> PathBuf {
    PathBuf::from("tests/data/arg_tests").join(file_name)
}

#[test]
fn export_model_from_grammar_matches_llk_precomputed_path() -> Result<()> {
    let grammar_config = obtain_grammar_config(arg_test_grammar("generate.par"), false)?;

    let lookahead_dfas = calculate_lookahead_dfas(&grammar_config, 5)?;
    let expected = generate_parser_export_model(&grammar_config, &lookahead_dfas)?;
    let actual = generate_parser_export_model_from_grammar(&grammar_config, 5)?;

    assert_eq!(actual.algorithm, ParserAlgorithmKindModel::Llk);
    assert_eq!(actual, expected);
    assert!(actual.lalr_parse_table.is_none());
    assert!(!actual.lookahead_automata.is_empty());

    Ok(())
}

#[test]
fn export_model_from_grammar_matches_lalr_precomputed_path() -> Result<()> {
    let grammar_config = obtain_grammar_config(arg_test_grammar("generate_lr.par"), false)?;

    let parse_table = calculate_lalr1_parse_table(&grammar_config)?.0;
    let expected = generate_lalr1_parser_export_model(&grammar_config, &parse_table)?;
    let actual = generate_parser_export_model_from_grammar(&grammar_config, 5)?;

    assert_eq!(actual.algorithm, ParserAlgorithmKindModel::Lalr1);
    assert_eq!(actual, expected);
    assert!(actual.lookahead_automata.is_empty());
    assert!(actual.lalr_parse_table.is_some());

    Ok(())
}

#[test]
fn export_model_from_grammar_ignores_max_lookahead_for_lalr() -> Result<()> {
    let grammar_config = obtain_grammar_config(arg_test_grammar("generate_lr.par"), false)?;

    let model_k1 = generate_parser_export_model_from_grammar(&grammar_config, 1)?;
    let model_k9 = generate_parser_export_model_from_grammar(&grammar_config, 9)?;

    assert_eq!(model_k1.algorithm, ParserAlgorithmKindModel::Lalr1);
    assert_eq!(model_k1, model_k9);

    Ok(())
}
