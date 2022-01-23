extern crate clap;

mod arguments;
mod tools;

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::{env, fs};

use arguments::ClapApp;
use clap::Parser;
use log::trace;
use miette::{miette, IntoDiagnostic, Result, WrapErr};

use id_tree::Tree;
use parol::{
    build::{BuildListener, IntermediateGrammar},
    render_par_string, GrammarConfig, ParolGrammar,
};
use parol_runtime::parser::ParseTreeType;

// To rebuild the parser sources from scratch use the command build_parsers.ps1

fn main() -> Result<()> {
    env_logger::try_init().into_diagnostic()?;
    trace!("env logger started");

    let config = ClapApp::parse();

    if let Some(subcommand) = config.subcommand {
        return subcommand.invoke_main();
    }

    // If relative paths are specified, they should be resoled relative to the current directory
    let mut builder =
        parol::build::Builder::with_explicit_output_dir(env::current_dir().into_diagnostic()?);

    // It's okay if the output doesn't exist;
    builder.disable_output_sanity_checks();
    // Don't care about cargo.
    builder.set_cargo_integration(false);

    // NOTE: Grammar file is required option
    let grammar_file = config
        .grammar
        .as_ref()
        .ok_or_else(|| miette!("Missing input grammar file (Specify with `-f`)"))?;
    builder.grammar_file(&grammar_file);

    builder.max_lookahead(config.lookahead)?;
    if let Some(module) = &config.module {
        builder.user_trait_module_name(module);
    }
    if let Some(user_type) = &config.user_type {
        builder.user_type_name(user_type);
    }
    if let Some(actions_file) = &config.actions {
        builder.actions_output_file(actions_file);
    }
    if let Some(parser_file) = &config.parser {
        builder.parser_output_file(parser_file);
    }
    if let Some(expanded_grammar_file) = &config.expanded {
        if expanded_grammar_file == OsStr::new("--") {
            // We special case this in our listener (see below)
        } else {
            builder.expanded_grammar_output_file(expanded_grammar_file);
        }
    }

    let mut listener = CLIListener {
        grammar_file,
        config: &config,
    };
    let mut generator = builder.begin_generation_with(Some(&mut listener))?;

    generator.parse()?;
    generator.expand()?;
    generator.post_process()?;
    generator.write_output()?;

    Ok(())
}

pub struct CLIListener<'a> {
    config: &'a ClapApp,
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
    ) -> miette::Result<()> {
        if self.verbose() {
            println!("{}", parol_grammar);
        }

        if let Some(file_name) = self.config.write_internal.as_ref() {
            let serialized = format!("{}", parol_grammar);
            fs::write(file_name, serialized)
                .into_diagnostic()
                .wrap_err("Error writing left-factored grammar!")?;
        }

        if self.config.generate_tree_graph {
            parol::generate_tree_layout(syntax_tree, &self.grammar_file)
                .wrap_err("Error generating tree layout")?;
        }

        Ok(())
    }

    fn on_intermediate_grammar(
        &mut self,
        stage: IntermediateGrammar,
        grammar_config: &GrammarConfig,
    ) -> miette::Result<()> {
        match stage {
            // no passes yet
            IntermediateGrammar::Untransformed => {
                if let Some(file_name) = self.config.write_untransformed.as_ref() {
                    let serialized = render_par_string(grammar_config, false);
                    fs::write(file_name, serialized)
                        .into_diagnostic()
                        .wrap_err("Error writing untransformed grammar!")?;
                }
            }
            // final pass
            IntermediateGrammar::LAST => {
                if let Some(file_name) = self.config.expanded.as_ref() {
                    // NOTE: We still need special handling for writing to stdout
                    let lf_source = render_par_string(grammar_config, true);
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
