use parol::build::Builder;

fn main() {
    // CLI equivalent is:
    // parol -f ./oberon2.par -e ./oberon2-exp.par -p ./src/oberon2_parser.rs -a ./src/oberon2_grammar_trait.rs -t Oberon2Grammar -m oberon2_grammar -g -b
    Builder::with_explicit_output_dir("src")
        .grammar_file("oberon2.par")
        .expanded_grammar_output_file("../oberon2-exp.par")
        .parser_output_file("oberon2_parser.rs")
        .actions_output_file("oberon2_grammar_trait.rs")
        .enable_auto_generation()
        .minimize_boxed_types()
        .user_type_name("Oberon2Grammar")
        .user_trait_module_name("oberon2_grammar")
        .trim_parse_tree()
        .generate_parser()
        .unwrap();
}
