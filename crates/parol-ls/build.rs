use parol::build::Builder;

fn main() {
    // CLI equivalent is:
    // parol -f ./parol_ls.par -e ./parol_ls-exp.par -p ./src/parol_ls_parser.rs -a ./src/parol_ls_grammar_trait.rs -t ParolLsGrammar -m parol_ls_grammar -g
    Builder::with_explicit_output_dir("src")
        .grammar_file("parol_ls.par")
        .expanded_grammar_output_file("../parol_ls-exp.par")
        .parser_output_file("parol_ls_parser.rs")
        .actions_output_file("parol_ls_grammar_trait.rs")
        .enable_auto_generation()
        .user_type_name("ParolLsGrammar")
        .user_trait_module_name("parol_ls_grammar")
        .generate_parser()
        .unwrap();
}
