use parol::build::Builder;
use parol_runtime::Result;

fn main() -> Result<()> {
    // CLI equivalent is:
    // parol -f ./parol_ls.par -e ./parol_ls-exp.par -p ./src/parol_ls_parser.rs -a ./src/parol_ls_grammar_trait.rs -t ParolLsGrammar -m parol_ls_grammar -g -b -x
    Builder::with_explicit_output_dir("src")
        .grammar_file("parol_ls.par")
        .expanded_grammar_output_file("../parol_ls-exp.par")
        .parser_output_file("parol_ls_parser.rs")
        .actions_output_file("parol_ls_grammar_trait.rs")
        .minimize_boxed_types()
        .user_type_name("ParolLsGrammar")
        .user_trait_module_name("parol_ls_grammar")
        .trim_parse_tree()
        .generate_parser()?;
    Ok(())
}
