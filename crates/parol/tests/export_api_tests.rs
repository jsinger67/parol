use anyhow::Result;
use parol::generators::{PARSER_EXPORT_MODEL_VERSION, ParserAlgorithmKindModel, ParserExportModel};
use parol::{
    calculate_lalr1_parse_table, calculate_lookahead_dfas, generate_lalr1_parser_export_model,
    generate_parser_export_model, generate_parser_export_model_from_grammar, obtain_grammar_config,
};
use std::path::PathBuf;

fn arg_test_grammar(file_name: &str) -> PathBuf {
    PathBuf::from("tests/data/arg_tests").join(file_name)
}

fn assert_common_contract(model: &ParserExportModel) {
    assert_eq!(model.version, PARSER_EXPORT_MODEL_VERSION);
    assert!(!model.non_terminal_names.is_empty());
    assert!(model.start_symbol_index < model.non_terminal_names.len());
    assert!(!model.productions.is_empty());
    assert!(!model.scanner.scanner_states.is_empty());
    assert_eq!(model.production_datatypes.len(), model.productions.len());
}

fn validate_export_model_for_consumer(model: &ParserExportModel) -> Result<(), String> {
    if model.version != PARSER_EXPORT_MODEL_VERSION {
        return Err(format!(
            "Unsupported export version {} (expected {})",
            model.version, PARSER_EXPORT_MODEL_VERSION
        ));
    }
    if model.non_terminal_names.is_empty() {
        return Err("non_terminal_names must not be empty".to_string());
    }
    if model.start_symbol_index >= model.non_terminal_names.len() {
        return Err("start_symbol_index out of range".to_string());
    }
    if model.productions.is_empty() {
        return Err("productions must not be empty".to_string());
    }
    if model.scanner.scanner_states.is_empty() {
        return Err("scanner.scanner_states must not be empty".to_string());
    }
    if model.production_datatypes.len() != model.productions.len() {
        return Err("production_datatypes length must match productions".to_string());
    }

    match model.algorithm {
        ParserAlgorithmKindModel::Llk => {
            if model.lookahead_automata.is_empty() {
                return Err("LL(k) export must contain lookahead_automata".to_string());
            }
            if model.lalr_parse_table.is_some() {
                return Err("LL(k) export must not contain lalr_parse_table".to_string());
            }
        }
        ParserAlgorithmKindModel::Lalr1 => {
            if !model.lookahead_automata.is_empty() {
                return Err("LALR(1) export must not contain lookahead_automata".to_string());
            }
            if model.lalr_parse_table.is_none() {
                return Err("LALR(1) export must contain lalr_parse_table".to_string());
            }
        }
    }

    Ok(())
}

#[test]
fn export_model_from_grammar_matches_llk_precomputed_path() -> Result<()> {
    let grammar_config = obtain_grammar_config(arg_test_grammar("generate.par"), false)?;

    let lookahead_dfas = calculate_lookahead_dfas(&grammar_config, 5)?;
    let expected = generate_parser_export_model(&grammar_config, &lookahead_dfas)?;
    let actual = generate_parser_export_model_from_grammar(&grammar_config, 5)?;

    assert_eq!(actual.algorithm, ParserAlgorithmKindModel::Llk);
    assert_eq!(actual, expected);
    assert_common_contract(&actual);
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
    assert_common_contract(&actual);
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

#[test]
fn export_model_llk_contract_sections_are_present() -> Result<()> {
    let grammar_config = obtain_grammar_config(arg_test_grammar("generate.par"), false)?;
    let model = generate_parser_export_model_from_grammar(&grammar_config, 5)?;

    assert_eq!(model.algorithm, ParserAlgorithmKindModel::Llk);
    assert_common_contract(&model);
    assert!(!model.lookahead_automata.is_empty());
    assert!(model.lalr_parse_table.is_none());

    Ok(())
}

#[test]
fn export_model_lalr_contract_sections_are_present() -> Result<()> {
    let grammar_config = obtain_grammar_config(arg_test_grammar("generate_lr.par"), false)?;
    let model = generate_parser_export_model_from_grammar(&grammar_config, 5)?;

    assert_eq!(model.algorithm, ParserAlgorithmKindModel::Lalr1);
    assert_common_contract(&model);
    assert!(model.lookahead_automata.is_empty());
    assert!(model.lalr_parse_table.is_some());

    Ok(())
}

#[test]
fn consumer_validation_rejects_unsupported_export_version() -> Result<()> {
    let grammar_config = obtain_grammar_config(arg_test_grammar("generate.par"), false)?;
    let mut model = generate_parser_export_model_from_grammar(&grammar_config, 5)?;
    model.version = PARSER_EXPORT_MODEL_VERSION + 1;

    let error = validate_export_model_for_consumer(&model).unwrap_err();
    assert!(error.contains("Unsupported export version"));

    Ok(())
}

#[test]
fn consumer_validation_rejects_llk_with_lalr_parse_table() -> Result<()> {
    let grammar_config = obtain_grammar_config(arg_test_grammar("generate.par"), false)?;
    let lr_grammar_config = obtain_grammar_config(arg_test_grammar("generate_lr.par"), false)?;
    let lr_model = generate_parser_export_model_from_grammar(&lr_grammar_config, 5)?;

    let mut llk_model = generate_parser_export_model_from_grammar(&grammar_config, 5)?;
    llk_model.lalr_parse_table = lr_model.lalr_parse_table.clone();

    let error = validate_export_model_for_consumer(&llk_model).unwrap_err();
    assert!(error.contains("LL(k) export must not contain lalr_parse_table"));

    Ok(())
}

#[test]
fn consumer_validation_rejects_lalr_without_parse_table() -> Result<()> {
    let grammar_config = obtain_grammar_config(arg_test_grammar("generate_lr.par"), false)?;
    let mut model = generate_parser_export_model_from_grammar(&grammar_config, 5)?;
    model.lalr_parse_table = None;

    let error = validate_export_model_for_consumer(&model).unwrap_err();
    assert!(error.contains("LALR(1) export must contain lalr_parse_table"));

    Ok(())
}

#[test]
fn consumer_validation_rejects_lalr_with_lookahead_automata() -> Result<()> {
    let grammar_config = obtain_grammar_config(arg_test_grammar("generate_lr.par"), false)?;
    let llk_grammar_config = obtain_grammar_config(arg_test_grammar("generate.par"), false)?;
    let llk_model = generate_parser_export_model_from_grammar(&llk_grammar_config, 5)?;

    let mut lalr_model = generate_parser_export_model_from_grammar(&grammar_config, 5)?;
    lalr_model.lookahead_automata = llk_model.lookahead_automata.clone();

    let error = validate_export_model_for_consumer(&lalr_model).unwrap_err();
    assert!(error.contains("LALR(1) export must not contain lookahead_automata"));

    Ok(())
}
