use parol::generators::PARSER_EXPORT_MODEL_VERSION;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_json(path: PathBuf) -> Value {
    let content = fs::read_to_string(path).expect("Failed reading JSON file");
    serde_json::from_str(&content).expect("Failed parsing JSON file")
}

fn schema_path() -> PathBuf {
    manifest_dir().join("schemas/parser-export-model.v1.schema.json")
}

fn snapshot_path(file_name: &str) -> PathBuf {
    manifest_dir().join("tests/data/arg_tests").join(file_name)
}

fn assert_required_top_level_fields(schema: &Value) {
    let required = schema
        .get("required")
        .and_then(Value::as_array)
        .expect("Schema must define top-level required fields");

    let required_names = required
        .iter()
        .map(|v| v.as_str().expect("Required entries must be strings"))
        .collect::<Vec<_>>();

    for field in [
        "version",
        "algorithm",
        "non_terminal_names",
        "start_symbol_index",
        "productions",
        "lookahead_automata",
        "lalr_parse_table",
        "scanner",
        "production_datatypes",
    ] {
        assert!(
            required_names.contains(&field),
            "Schema is missing required field '{field}'"
        );
    }
}

fn assert_snapshot_satisfies_common_contract(snapshot: &Value) {
    let obj = snapshot
        .as_object()
        .expect("Export snapshot must be a JSON object");

    for key in [
        "version",
        "algorithm",
        "non_terminal_names",
        "start_symbol_index",
        "productions",
        "lookahead_automata",
        "lalr_parse_table",
        "scanner",
        "production_datatypes",
    ] {
        assert!(obj.contains_key(key), "Snapshot missing key '{key}'");
    }

    assert_eq!(
        obj.get("version")
            .and_then(Value::as_u64)
            .expect("version must be an unsigned integer"),
        PARSER_EXPORT_MODEL_VERSION as u64
    );

    let non_terminals = obj
        .get("non_terminal_names")
        .and_then(Value::as_array)
        .expect("non_terminal_names must be an array");
    assert!(
        !non_terminals.is_empty(),
        "non_terminal_names must not be empty"
    );

    let start_symbol_index =
        obj.get("start_symbol_index")
            .and_then(Value::as_u64)
            .expect("start_symbol_index must be an unsigned integer") as usize;
    assert!(
        start_symbol_index < non_terminals.len(),
        "start_symbol_index must be within non_terminal_names"
    );

    let productions = obj
        .get("productions")
        .and_then(Value::as_array)
        .expect("productions must be an array");
    assert!(!productions.is_empty(), "productions must not be empty");

    let production_datatypes = obj
        .get("production_datatypes")
        .and_then(Value::as_array)
        .expect("production_datatypes must be an array");
    assert_eq!(
        production_datatypes.len(),
        productions.len(),
        "production_datatypes length must match productions"
    );

    let scanner_states = obj
        .get("scanner")
        .and_then(Value::as_object)
        .and_then(|scanner| scanner.get("scanner_states"))
        .and_then(Value::as_array)
        .expect("scanner.scanner_states must be an array");
    assert!(
        !scanner_states.is_empty(),
        "scanner.scanner_states must not be empty"
    );
}

#[test]
fn export_schema_has_expected_version_and_required_fields() {
    let schema = read_json(schema_path());

    let version_const = schema
        .get("properties")
        .and_then(Value::as_object)
        .and_then(|properties| properties.get("version"))
        .and_then(Value::as_object)
        .and_then(|version| version.get("const"))
        .and_then(Value::as_u64)
        .expect("Schema must define properties.version.const");

    assert_eq!(version_const, PARSER_EXPORT_MODEL_VERSION as u64);
    assert_required_top_level_fields(&schema);
}

#[test]
fn llk_snapshot_matches_schema_contract_shape() {
    let snapshot = read_json(snapshot_path("export_llk.expected.json"));
    assert_snapshot_satisfies_common_contract(&snapshot);

    let algorithm = snapshot
        .get("algorithm")
        .and_then(Value::as_str)
        .expect("algorithm must be a string");
    assert_eq!(algorithm, "Llk");

    let lookahead = snapshot
        .get("lookahead_automata")
        .and_then(Value::as_array)
        .expect("lookahead_automata must be an array");
    assert!(!lookahead.is_empty(), "Llk export needs lookahead_automata");

    assert!(
        snapshot
            .get("lalr_parse_table")
            .expect("lalr_parse_table must exist")
            .is_null(),
        "Llk export must not contain lalr_parse_table"
    );
}

#[test]
fn lalr_snapshot_matches_schema_contract_shape() {
    let snapshot = read_json(snapshot_path("export_lalr1.expected.json"));
    assert_snapshot_satisfies_common_contract(&snapshot);

    let algorithm = snapshot
        .get("algorithm")
        .and_then(Value::as_str)
        .expect("algorithm must be a string");
    assert_eq!(algorithm, "Lalr1");

    let lookahead = snapshot
        .get("lookahead_automata")
        .and_then(Value::as_array)
        .expect("lookahead_automata must be an array");
    assert!(
        lookahead.is_empty(),
        "Lalr1 export must not contain lookahead_automata"
    );

    let parse_table = snapshot
        .get("lalr_parse_table")
        .expect("lalr_parse_table must exist");
    assert!(
        parse_table.is_object(),
        "Lalr1 export must contain a lalr_parse_table object"
    );
}
