use miette::{miette, Result};
use parol::generators::grammar_type_generator::GrammarTypeInfo;
use parol::{left_factor, obtain_grammar_config, obtain_grammar_config_from_string};
use std::path::PathBuf;

/// Calculates the type structure of the generated expanded grammar.
#[derive(clap::Parser)]
#[clap(name = "deduce_types")]
pub struct Args {
    /// The grammar file to use
    #[clap(short = 'f', long = "grammar-file", parse(from_os_str))]
    grammar_file: Option<PathBuf>,
    /// Grammar input as text
    #[clap(short = 's', long = "grammar-text")]
    grammar: Option<String>,
    /// Increase verbosity
    #[clap(short = 'v', long = "verbose")]
    verbose: bool,
}

pub fn main(args: &Args) -> Result<()> {
    let mut grammar_config = if let Some(file_name) = &args.grammar_file {
        obtain_grammar_config(&file_name, args.verbose)?
    } else if let Some(input) = &args.grammar {
        obtain_grammar_config_from_string(input, args.verbose)?
    } else {
        return Err(miette!("Please provide a valid grammar input!"));
    };

    let grammar_name = if let Some(file_name) = &args.grammar_file {
        file_name.file_stem().unwrap().to_str().unwrap_or("TestGrammar")
    } else {
        "TestGrammar"
    }.replace("-exp", "").replace('.', "_").replace('-', "_");

    let cfg = left_factor(&grammar_config.cfg);
    // Exchange original grammar with transformed one
    grammar_config.update_cfg(cfg);

    let width = (grammar_config.cfg.pr.len() as f32).log10() as usize + 1;
    let mut type_info = GrammarTypeInfo::try_new(&grammar_name)?;
    type_info.build(&grammar_config)?;
    let scanner_state_resolver = grammar_config.get_scanner_state_resolver();
    for (i, pr) in grammar_config.cfg.pr.iter().enumerate() {
        println!(
            "/* {:w$} */ {}",
            i,
            pr.format(&scanner_state_resolver)?,
            w = width
        );
    }
    println!();
    println!("Type information:");
    println!("{}", type_info);
    Ok(())
}
