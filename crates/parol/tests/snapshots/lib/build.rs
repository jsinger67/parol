use std::process;

use parol::{build::Builder, ParolErrorReporter};
use parol_runtime::Report;

fn main() {
    // CLI equivalent is:
    // parol -f ./snapshot_lib.par -e ./snapshot_lib-exp.par -p ./src/snapshot_lib_parser.rs -a ./src/snapshot_lib_grammar_trait.rs -t SnapshotLibGrammar -m snapshot_lib_grammar -g
    if let Err(err) = Builder::with_explicit_output_dir("src")
        .grammar_file("snapshot_lib.par")
        .expanded_grammar_output_file("../snapshot_lib-exp.par")
        .parser_output_file("snapshot_lib_parser.rs")
        .actions_output_file("snapshot_lib_grammar_trait.rs")
        .enable_auto_generation()
        .user_type_name("SnapshotLibGrammar")
        .user_trait_module_name("snapshot_lib_grammar")
        .trim_parse_tree()
        .generate_parser()
    {
        ParolErrorReporter::report_error(&err, "snapshot_lib.par").unwrap_or_default();
        process::exit(1);
    }
}
