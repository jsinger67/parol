#[macro_use]
extern crate clap;

use anyhow::{bail, Context, Result};
use clap::App;
use parol::parser::parol_grammar::ParolGrammar;
use parol::parser::parol_parser::parse;
use parol::MAX_K;
use std::convert::TryFrom;

use log::trace;
use parol::analysis::k_decision::calculate_lookahead_dfas;
use parol::conversions::par::render_par_string;
use parol::generate_tree_layout;
use parol::generators::GrammarConfig;
use parol::generators::{
    check_and_transform_grammar, generate_lexer_source, generate_parser_source,
    generate_user_trait_source, try_format,
};
use std::fs;

// To rebuild the parser sources from scratch use the command build_parsers.ps1

fn main() -> Result<()> {
    // $env:RUST_LOG="parol_runtime=trace,parol=trace"
    // std::env::set_var("RUST_LOG", "parol::analysis::first,parol::analysis::follow,parol::analysis::k_decision,parol::main=trace");
    env_logger::try_init()?;
    trace!("env logger started");

    let yaml = load_yaml!("arguments.yml");
    let config = App::from_yaml(yaml).get_matches();

    let max_k = config.value_of("lookahead").unwrap().parse::<usize>()?;
    if max_k > MAX_K {
        bail!("Maximum lookahead is {}", MAX_K);
    }

    let verbose = config.is_present("verbose");

    let mut grammar_config = obtain_grammar_config(
        config.value_of("grammar"),
        verbose,
        config.is_present("generate_tree_graph"),
    )?;

    write_expanded_grammar(&grammar_config, config.value_of("expanded"))
        .with_context(|| "Error writing left-factored grammar!")?;

    let cfg = check_and_transform_grammar(&grammar_config.cfg)
        .with_context(|| "Basic grammar checks and transformations failed!")?;

    // Exchange original grammar with transformed one
    grammar_config.update_cfg(cfg);

    write_expanded_grammar(&grammar_config, config.value_of("expanded"))
        .with_context(|| "Error writing left-factored grammar!")?;

    if !config.is_present("parser") && !config.is_present("only_lookahead") {
        return Ok(());
    }

    let lookahead_dfa_s = calculate_lookahead_dfas(&grammar_config, max_k)
        .with_context(|| "Lookahead calculation for the given grammar failed!")?;

    if config.is_present("verbose") {
        print!("Lookahead DFAs:\n{:?}", lookahead_dfa_s);
    }

    // Update maximum lookahead size for scanner generation
    grammar_config.update_lookahead_size(
        lookahead_dfa_s
            .iter()
            .max_by_key(|(_, dfa)| dfa.k)
            .unwrap()
            .1
            .k,
    );

    if config.is_present("verbose") {
        print!("\nGrammar config:\n{:?}", grammar_config);
    }

    let lexer_source = generate_lexer_source(&grammar_config)
        .with_context(|| "Failed to generate lexer source!")?;

    let user_trait_module_name = config.value_of("module").unwrap();

    let user_type = config.value_of("user_type").unwrap();

    let parser_source = generate_parser_source(&grammar_config, &lexer_source, &lookahead_dfa_s)
        .with_context(|| "Failed to generate parser source!")?;

    if let Some(parser_file_out) = config.value_of("parser") {
        fs::write(parser_file_out, parser_source)
            .with_context(|| "Error writing generated lexer source!")?;
        try_format(parser_file_out);
    } else if verbose {
        println!("\nParser source:\n{}", parser_source);
    }

    let user_trait_source =
        generate_user_trait_source(user_type, user_trait_module_name, &grammar_config)
            .with_context(|| "Failed to generate user trait source!")?;
    if let Some(user_trait_file_out) = config.value_of("actions") {
        fs::write(user_trait_file_out, user_trait_source)
            .with_context(|| "Error writing generated user trait source!")?;
        try_format(user_trait_file_out);
    } else if verbose {
        println!("\nSource for semantic actions:\n{}", user_trait_source);
    }

    Ok(())
}

fn obtain_grammar_config(
    grammar: Option<&str>,
    verbose: bool,
    generate_tree_graph: bool,
) -> Result<GrammarConfig> {
    if let Some(file_name) = grammar {
        let input = fs::read_to_string(file_name)
            .with_context(|| format!("Can't read file {}", file_name))?;
        let mut parol_grammar = ParolGrammar::new();
        let syntax_tree = parse(&input, file_name.to_owned(), &mut parol_grammar)
            .with_context(|| format!("Failed parsing file {}", file_name))?;

        if verbose {
            println!("{}", parol_grammar);
        }

        if generate_tree_graph {
            generate_tree_layout(&syntax_tree, file_name)
                .with_context(|| "Error generating tree layout")?;
        }

        Ok(GrammarConfig::try_from(parol_grammar)?)
    } else {
        bail!("Need grammar file!");
    }
}

fn write_expanded_grammar(grammar_config: &GrammarConfig, expanded: Option<&str>) -> Result<()> {
    if let Some(file_name) = expanded.as_ref() {
        let lf_source = render_par_string(grammar_config, true);
        if *file_name == "--" {
            print!("{}", lf_source);
        } else {
            fs::write(file_name, lf_source)
                .with_context(|| "Error writing left-factored grammar!")?;
        }
    }
    Ok(())
}
