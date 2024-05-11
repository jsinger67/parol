use std::process;

use parol::{build::Builder, ParolErrorReporter};
use parol_runtime::Report;

fn main() {
    // CLI equivalent is:
    // parol -f ./calc.par -e ./calc-exp.par -p ./calc_parser.rs -a ./calc_grammar_trait.rs -t CalcGrammar -m calc_grammar -g
    if let Err(err) = Builder::with_explicit_output_dir(".")
        .grammar_file("calc.par")
        .expanded_grammar_output_file("calc-exp.par")
        .parser_output_file("calc_parser.rs")
        .actions_output_file("calc_grammar_trait.rs")
        .enable_auto_generation()
        .user_type_name("CalcGrammar")
        .user_trait_module_name("calc_grammar")
        .trim_parse_tree()
        .generate_parser()
    {
        ParolErrorReporter::report_error(&err, "calc.par").unwrap_or_default();
        process::exit(1);
    }
}
