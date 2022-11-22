use parol::build::Builder;

fn main() {
    // CLI equivalent
    // parol -f ./json.par -e ./json-exp.par -p ./src/json_parser.rs -a ./src/json_grammar_trait.rs -t JsonGrammar -m json_grammar

    // Unlike Builder::with_cargo_output, explicit names for the parser and actions files are
    // required!
    Builder::with_explicit_output_dir("src")
        .grammar_file("json.par")
        .expanded_grammar_output_file("../json-exp.par")
        .parser_output_file("json_parser.rs")
        .actions_output_file("json_grammar_trait.rs")
        .user_type_name("JsonGrammar")
        .user_trait_module_name("json_grammar")
        .generate_parser()
        .unwrap();
}
