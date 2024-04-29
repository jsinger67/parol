use std::process;

use parol::{build::Builder, ParolErrorReporter};
use parol_runtime::Report;

fn main() {
    // CLI equivalent is:
    // parol -f ./list.par -e ./list-exp.par -p ./src/list_parser.rs -a ./src/list_grammar_trait.rs -t ListGrammar -m list_grammar -g
    if let Err(err) = Builder::with_explicit_output_dir(".")
        .grammar_file("list.par")
        .expanded_grammar_output_file("list-exp.par")
        .parser_output_file("list_parser.rs")
        .actions_output_file("list_grammar_trait.rs")
        .enable_auto_generation()
        .minimize_boxed_types()
        .user_type_name("ListGrammar")
        .user_trait_module_name("list_grammar")
        .trim_parse_tree()
        .generate_parser()
    {
        ParolErrorReporter::report_error(&err, "list.par").unwrap_or_default();
        process::exit(1);
    }
}
