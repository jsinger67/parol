extern crate clap;

mod arguments;
mod tools;

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::time::Instant;
use std::{env, fs};

use anyhow::Context;
use arguments::CliArgs;
use clap::Parser;
use owo_colors::OwoColorize;
use parol_runtime::{log::trace, parser::ParseTreeType, Report, Result};

use id_tree::Tree;
use parol::{
    build::{BuildListener, IntermediateGrammar},
    render_par_string, GrammarConfig, ParolErrorReporter, ParolGrammar,
};
use parol_macros::parol;

// To rebuild the parser sources from scratch use the command build_parsers.ps1

fn run(args: &CliArgs) -> Result<u128> {
    let now = Instant::now();
    if let Some(subcommand) = &args.subcommand {
        return subcommand
            .invoke_main()
            .map_err(|e| parol!(e))
            .map(|_| now.elapsed().as_millis());
    }

    // If relative paths are specified, they should be resoled relative to the current directory
    let mut builder =
        parol::build::Builder::with_explicit_output_dir(env::current_dir().map_err(|e| parol!(e))?);

    // It's okay if the output doesn't exist;
    builder.disable_output_sanity_checks();
    // Don't care about cargo.
    builder.set_cargo_integration(false);

    // NOTE: Grammar file is required option
    let grammar_file = args
        .grammar
        .as_ref()
        .ok_or_else(|| parol!("Missing input grammar file (Specify with `-f`)"))?;
    builder.grammar_file(grammar_file);

    builder
        .max_lookahead(args.lookahead)
        .map_err(|e| parol!(e))?;
    if let Some(module) = &args.module {
        builder.user_trait_module_name(module);
    }
    if let Some(user_type) = &args.user_type {
        builder.user_type_name(user_type);
    }
    if let Some(actions_file) = &args.actions {
        builder.actions_output_file(actions_file);
    }
    if let Some(parser_file) = &args.parser {
        builder.parser_output_file(parser_file);
    }
    if args.auto_generate {
        builder.enable_auto_generation();
    }
    if args.range {
        builder.range();
    }
    builder.inner_attributes(args.inner_attributes.clone());
    if let Some(expanded_grammar_file) = &args.expanded {
        if expanded_grammar_file == OsStr::new("--") {
            // We special case this in our listener (see below)
        } else {
            builder.expanded_grammar_output_file(expanded_grammar_file);
        }
    }

    let mut listener = CLIListener {
        grammar_file,
        config: args,
    };
    let mut generator = builder
        .begin_generation_with(Some(&mut listener))
        .map_err(|e| parol!(e))?;

    generator.parse()?;
    generator.expand()?;
    generator.post_process()?;
    generator.write_output()?;

    Ok(now.elapsed().as_millis())
}

pub struct CLIListener<'a> {
    config: &'a CliArgs,
    grammar_file: &'a Path,
}
impl CLIListener<'_> {
    fn verbose(&self) -> bool {
        self.config.verbose
    }
}
impl BuildListener for CLIListener<'_> {
    fn on_initial_grammar_parse(
        &mut self,
        syntax_tree: &Tree<ParseTreeType>,
        parol_grammar: &ParolGrammar,
    ) -> Result<()> {
        if self.verbose() {
            println!("{}", parol_grammar);
        }

        if let Some(file_name) = self.config.write_internal.as_ref() {
            let serialized = format!("{}", parol_grammar);
            fs::write(file_name, serialized).context("Error writing left-factored grammar!")?;
        }

        if self.config.generate_tree_graph {
            parol::generate_tree_layout(syntax_tree, self.grammar_file)
                .context("Error generating tree layout")
                .map_err(|e| parol!(e))?;
        }

        Ok(())
    }

    fn on_intermediate_grammar(
        &mut self,
        stage: IntermediateGrammar,
        grammar_config: &GrammarConfig,
    ) -> Result<()> {
        match stage {
            // no passes yet
            IntermediateGrammar::Untransformed => {
                if let Some(file_name) = self.config.write_untransformed.as_ref() {
                    let serialized = render_par_string(grammar_config, false)?;
                    fs::write(file_name, serialized)
                        .context("Error writing untransformed grammar!")
                        .map_err(|e| parol!(e))?;
                }
            }
            // final pass
            IntermediateGrammar::LAST => {
                if let Some(file_name) = self.config.expanded.as_ref() {
                    // NOTE: We still need special handling for writing to stdout
                    let lf_source =
                        render_par_string(grammar_config, true).map_err(|e| parol!(e))?;
                    if *file_name == OsStr::new("--") {
                        print!("{}", lf_source);
                    } else {
                        // Should be handled by the builder
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}

fn main() -> Result<std::process::ExitCode> {
    env_logger::try_init().map_err(|e| parol!(e))?;
    trace!("env logger started");

    let args = CliArgs::parse();
    let file = extract_file_name(&args);
    match run(&args) {
        Ok(millis) => {
            println!(
                "{} {} ({} milliseconds)",
                "Parol".bright_blue(),
                "succeeded".bright_green(),
                millis
            );
            return Ok(std::process::ExitCode::SUCCESS);
        }
        Err(err) => ParolErrorReporter::report_error(&err, file).unwrap_or(()),
    }
    println!("{} {}", "Parol".bright_blue(), "failed".bright_red());
    Ok(std::process::ExitCode::FAILURE)
}

// We need the file name to support error reporting
fn extract_file_name(args: &CliArgs) -> PathBuf {
    if args.subcommand.is_some() {
        match args.subcommand.as_ref().unwrap() {
            tools::ToolsSubcommands::calculate_k(args) => args.grammar_file.to_path_buf(),
            tools::ToolsSubcommands::calculate_k_tuples(args) => args.grammar_file.to_path_buf(),
            tools::ToolsSubcommands::decidable(args) => args.grammar_file.to_path_buf(),
            tools::ToolsSubcommands::deduce_types(args) => args
                .grammar_file
                .as_ref()
                .map_or(PathBuf::default(), |f| f.to_path_buf()),
            tools::ToolsSubcommands::first(args) => args.grammar_file.to_path_buf(),
            tools::ToolsSubcommands::follow(args) => args.grammar_file.to_path_buf(),
            tools::ToolsSubcommands::format(args) => args.grammar_file.to_path_buf(),
            tools::ToolsSubcommands::generate(args) => args.grammar_file.to_path_buf(),
            tools::ToolsSubcommands::left_factor(args) => args.grammar_file.to_path_buf(),
            tools::ToolsSubcommands::left_recursions(args) => args.grammar_file.to_path_buf(),
            tools::ToolsSubcommands::new(_) => PathBuf::default(),
            tools::ToolsSubcommands::productivity(args) => args.grammar_file.to_path_buf(),
        }
    } else {
        args.grammar.as_ref().unwrap().to_path_buf()
    }
}
