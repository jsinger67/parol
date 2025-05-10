use derive_builder::Builder;

#[derive(Builder, Debug, Default)]
pub(crate) struct BuildRsData<'a> {
    crate_name: &'a str,
    grammar_name: String,
    tree_gen: bool,
}

impl std::fmt::Display for BuildRsData<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let BuildRsData {
            crate_name,
            grammar_name,
            tree_gen,
        } = self;

        let trim_parse_tree = if *tree_gen {
            ""
        } else {
            "\n        .trim_parse_tree()"
        };

        write!(
            f,
            r#"use std::process;

use parol::{{build::Builder, parol_runtime::Report, ParolErrorReporter}};

fn main() {{
    // CLI equivalent is:
    // parol -f ./{crate_name}.par -e ./{crate_name}-exp.par -p ./src/{crate_name}_parser.rs -a ./src/{crate_name}_grammar_trait.rs -t {grammar_name}Grammar -m {crate_name}_grammar
    if let Err(err) = Builder::with_explicit_output_dir("src")
        .grammar_file("{crate_name}.par")
        .expanded_grammar_output_file("../{crate_name}-exp.par")
        .parser_output_file("{crate_name}_parser.rs")
        .actions_output_file("{crate_name}_grammar_trait.rs")
        .user_type_name("{grammar_name}Grammar")
        .user_trait_module_name("{crate_name}_grammar"){trim_parse_tree}
        .generate_parser()
    {{
        ParolErrorReporter::report_error(&err, "{crate_name}.par").unwrap_or_default();
        process::exit(1);
    }}
}}
"#
        )
    }
}
