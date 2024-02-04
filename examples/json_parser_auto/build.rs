use parol::build::Builder;

fn main() {
    // CLI equivalent is:
    // parol -f ./json.par -e ./json-exp.par -p ./src/json_parser.rs -a ./src/json_grammar_trait.rs -t JsonGrammar -m json_grammar -g -b
    Builder::with_explicit_output_dir("src")
        .grammar_file("json.par")
        .expanded_grammar_output_file("../json-exp.par")
        .parser_output_file("json_parser.rs")
        .actions_output_file("json_grammar_trait.rs")
        .enable_auto_generation()
        .minimize_boxed_types()
        .user_type_name("JsonGrammar")
        .user_trait_module_name("json_grammar")
        .trim_parse_tree()
        .generate_parser()
        .unwrap();
}
