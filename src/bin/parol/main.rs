#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate clap;

use clap::App;
use parol::parser::parol_grammar::ParolGrammar;
use parol::parser::parol_parser::parse;
use parol::MAX_K;
use std::convert::TryFrom;

use id_tree::Tree;
use id_tree_layout::Layouter;
use log::trace;
use parol::analysis::k_decision::calculate_lookahead_dfas;
use parol::conversions::par::render_par_string;
use parol::generators::GrammarConfig;
use parol::generators::{
    check_and_transform_grammar, generate_lexer_source, generate_parser_source,
    generate_user_trait_source, try_format,
};
use parol_runtime::parser::ParseTreeType;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

// To rebuild the parser sources from scratch use the command build_parsers.ps1

error_chain! {
    links {
        Parol(parol::errors::Error, parol::errors::ErrorKind);
    }
    foreign_links {
        Parse(::std::num::ParseIntError);
        Log(log::SetLoggerError);
    }
}

quick_main!(run);

fn run() -> Result<()> {
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
        .chain_err(|| "Error writing left-factored grammar!")?;

    let cfg = check_and_transform_grammar(&grammar_config.cfg)
        .chain_err(|| "Basic grammar checks and transformations failed!")?;

    // Exchange original grammar with transformed one
    grammar_config.update_cfg(cfg);

    write_expanded_grammar(&grammar_config, config.value_of("expanded"))
        .chain_err(|| "Error writing left-factored grammar!")?;

    if !config.is_present("parser") && !config.is_present("only_lookahead") {
        return Ok(());
    }

    let lookahead_dfa_s = calculate_lookahead_dfas(&grammar_config, max_k)
        .chain_err(|| "Lookahead calculation for the given grammar failed!")?;

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

    let lexer_source =
        generate_lexer_source(&grammar_config).chain_err(|| "Failed to generate lexer source!")?;

    let user_trait_module_name = config.value_of("module").unwrap();

    let user_type = config.value_of("user_type").unwrap();

    let parser_source = generate_parser_source(&grammar_config, &lexer_source, &lookahead_dfa_s)
        .chain_err(|| "Failed to generate parser source!")?;

    if let Some(parser_file_out) = config.value_of("parser") {
        fs::write(parser_file_out, parser_source)
            .chain_err(|| "Error writing generated lexer source!")?;
        try_format(parser_file_out);
    } else if verbose {
        println!("\nParser source:\n{}", parser_source);
    }

    let user_trait_source =
        generate_user_trait_source(user_type, user_trait_module_name, &grammar_config)
            .chain_err(|| "Failed to generate user trait source!")?;
    if let Some(user_trait_file_out) = config.value_of("actions") {
        fs::write(user_trait_file_out, user_trait_source)
            .chain_err(|| "Error writing generated user trait source!")?;
        try_format(user_trait_file_out);
    } else if verbose {
        println!("\nSource for semantic actions:\n{}", user_trait_source);
    }

    Ok(())
}

fn generate_tree_layout(
    syntax_tree: &Tree<ParseTreeType>,
    input_file_name: &str,
    verbose: bool,
) -> Result<()> {
    let mut svg_full_file_name = PathBuf::from_str(input_file_name).unwrap();
    svg_full_file_name.set_extension("");
    let file_name = svg_full_file_name
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
    svg_full_file_name.set_file_name(file_name);
    svg_full_file_name.set_extension("svg");

    if verbose {
        println!(
            "Writing tree layout to {}",
            svg_full_file_name.to_str().unwrap()
        );
    }

    Layouter::new(syntax_tree)
        .with_file_path(std::path::Path::new(&svg_full_file_name))
        .write()
        .chain_err(|| "Failed writing layout")
}

fn obtain_grammar_config(
    grammar: Option<&str>,
    verbose: bool,
    generate_tree_graph: bool,
) -> Result<GrammarConfig> {
    if let Some(file_name) = grammar {
        let input =
            fs::read_to_string(file_name).chain_err(|| format!("Can't read file {}", file_name))?;
        let mut parol_grammar = ParolGrammar::new();
        let syntax_tree = parse(&input, file_name.to_owned(), &mut parol_grammar)
            .chain_err(|| format!("Failed parsing file {}", file_name))?;

        if verbose {
            println!("{}", parol_grammar);
        }

        if generate_tree_graph {
            generate_tree_layout(&syntax_tree, file_name, verbose)?;
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
            fs::write(file_name, lf_source).chain_err(|| "Error writing left-factored grammar!")?;
        }
    }
    Ok(())
}
