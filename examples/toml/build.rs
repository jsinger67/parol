use parol::build::Builder;

fn main() {
    // CLI equivalent is:
    // parol -f ./parol_toml.par -e ./parol_toml-exp.par -p ./src/parol_toml_parser.rs -a ./src/parol_toml_grammar_trait.rs -t ParolTomlGrammar -m parol_toml_grammar -g
    Builder::with_explicit_output_dir("src")
        .grammar_file("parol_toml.par")
        .expanded_grammar_output_file("../parol_toml-exp.par")
        .parser_output_file("parol_toml_parser.rs")
        .actions_output_file("parol_toml_grammar_trait.rs")
        .enable_auto_generation()
        .user_type_name("ParolTomlGrammar")
        .user_trait_module_name("parol_toml_grammar")
        .generate_parser()
        .unwrap();
}
