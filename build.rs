use parol::build::Builder;

fn main() {
    // CLI equivalent is:
    // parol -f ./basic.par -e ./basic-exp.par -p ./src/basic_parser.rs -a ./src/basic_grammar_trait.rs -t BasicGrammar -m basic_grammar -g
    Builder::with_explicit_output_dir("src")
        .grammar_file("basic.par")
        .expanded_grammar_output_file("../basic-exp.par")
        .parser_output_file("basic_parser.rs")
        .actions_output_file("basic_grammar_trait.rs")
        .enable_auto_generation()
        .user_type_name("BasicGrammar")
        .user_trait_module_name("basic_grammar")
        .generate_parser()
        .unwrap();
}
