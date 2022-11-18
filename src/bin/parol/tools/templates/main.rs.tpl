extern crate parol_runtime;

// auto generation needs derive_builder
mod derive_builder {
    pub use parol_runtime::derive_builder::*;
}

mod {{crate_name}}_grammar;
// The output is version controlled
mod {{crate_name}}_grammar_trait;
mod {{crate_name}}_parser;

use crate::{{crate_name}}_grammar::{{grammar_name}}Grammar;
use crate::{{crate_name}}_parser::parse;{{#tree_gen?}}
use parol_runtime::id_tree::Tree;
use parol_runtime::id_tree_layout::Layouter;{{/tree_gen}}
use parol_runtime::log::debug;
use parol_runtime::miette::{miette, IntoDiagnostic, Result, WrapErr};{{#tree_gen?}}
use parol_runtime::parser::ParseTreeType;{{/tree_gen}}
use std::env;
use std::fs;
use std::time::Instant;

// To generate:
// parol -f ./{{crate_name}}.par -e ./{{crate_name}}-exp.par -p ./src/{{crate_name}}_parser.rs -a ./src/{{crate_name}}_grammar_trait.rs -t {{grammar_name}}Grammar -m {{crate_name}}_grammar -g

fn main() -> Result<()> {
    env_logger::init();
    debug!("env logger started");

    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 {
        let file_name = args[1].clone();
        let input = fs::read_to_string(file_name.clone())
            .into_diagnostic()
            .wrap_err(format!("Can't read file {}", file_name))?;
        let mut {{crate_name}}_grammar = {{grammar_name}}Grammar::new();
        let now = Instant::now();
        {{#tree_gen?}}let syntax_tree = {{/tree_gen}}parse(&input, &file_name, &mut {{crate_name}}_grammar)
            .wrap_err(format!("Failed parsing file {}", file_name))?;
        let elapsed_time = now.elapsed();
        println!("Parsing took {} milliseconds.", elapsed_time.as_millis());
        if args.len() > 2 && args[2] == "-q" {
            Ok(())
        } else {
            {{#tree_gen?}}generate_tree_layout(&syntax_tree, &file_name)?;
            {{/tree_gen}}println!("Success!\n{}", {{crate_name}}_grammar);
            Ok(())
        }
    } else {
        Err(miette!("Please provide a file name as first parameter!"))
    }
}

{{#tree_gen?}}fn generate_tree_layout(syntax_tree: &Tree<ParseTreeType>, input_file_name: &str) -> Result<()> {
    let mut svg_full_file_name = std::path::PathBuf::from(input_file_name);
    svg_full_file_name.set_extension("svg");

    Layouter::new(syntax_tree)
        .with_file_path(&svg_full_file_name)
        .write()
        .into_diagnostic()
        .wrap_err("Failed writing layout")
}
{{/tree_gen}}
