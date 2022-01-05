#[macro_use]
extern crate clap;

use std::path::{Path, PathBuf};
use std::{env, fs};

use clap::{App, AppSettings};
use log::trace;
use miette::{miette, IntoDiagnostic, Result, WrapErr};

use id_tree::Tree;
use parol::{
    build::{BuildListener, IntermediateGrammar},
    render_par_string, GrammarConfig, ParolGrammar,
};
use parol_runtime::parser::ParseTreeType;

static VERSION: &str = env!("CARGO_PKG_VERSION");

mod tools;

// To rebuild the parser sources from scratch use the command build_parsers.ps1

fn main() -> Result<()> {
    env_logger::try_init().into_diagnostic()?;
    trace!("env logger started");

    let yaml = load_yaml!("arguments.yml");
    let config = App::from_yaml(yaml)
        /*
         * We want all our "tools" to be registered as subcommands.
         *
         * Doing this allows `clap` to give better help and error messages
         * then if we used AppSettings::AllowExternalSubcommands
         */
        .subcommands(tools::names().map(|name| {
            /*
             * For now, our subcommands have no names or descriptions
             *
             * They all accept infinite args (clap makes no attempt to validate things here).
             */
            tools::get_tool_sub_command(name).unwrap()()
        }))
        // Only invoke tools if they come first, to avoid ambiguity with main binary
        .setting(AppSettings::ArgsNegateSubcommands)
        .version(VERSION)
        .get_matches();

    if let (subcommand_name, Some(sub_matches)) = config.subcommand() {
        let tool_main =
            tools::get_tool_main(subcommand_name).expect("Clap should've validated tool name");
        log::debug!("Delegating to {} with {:?}", subcommand_name, sub_matches);
        return tool_main(sub_matches);
    }

    // If relative paths are specified, they should be resoled relative to the current directory
    let mut builder =
        parol::build::Builder::with_explicit_output_dir(env::current_dir().into_diagnostic()?);

    // It's okay if the output doesn't exist;
    builder.disable_output_sanity_checks();
    // Don't care about cargo.
    builder.set_cargo_integration(false);

    // NOTE: Grammar file is required option
    let grammar_file = PathBuf::from(
        config
            .value_of("grammar")
            .ok_or_else(|| miette!("Missing input grammar file (Specify with `-f`)"))?,
    );
    builder.grammar_file(&grammar_file);

    if let Some(max_k_str) = config.value_of("lookahead") {
        builder.max_lookahead(max_k_str.parse::<usize>().into_diagnostic()?)?;
    }
    if let Some(module) = config.value_of("module") {
        builder.user_trait_module_name(module);
    }
    if let Some(user_type) = config.value_of("user_type") {
        builder.user_type_name(user_type);
    }
    if let Some(actions_file) = config.value_of("actions") {
        builder.actions_output_file(actions_file);
    }
    if let Some(parser_file) = config.value_of("parser") {
        builder.parser_output_file(parser_file);
    }
    if let Some(expanded_grammar_file) = config.value_of("expanded") {
        if expanded_grammar_file == "--" {
            // We special case this in our listener (see below)
        } else {
            builder.expanded_grammar_output_file(expanded_grammar_file);
        }
    }

    let mut listener = CLIListener {
        grammar_file: &grammar_file,
        config: &config,
    };
    let mut generator = builder.begin_generation_with(Some(&mut listener))?;

    generator.parse()?;
    generator.expand()?;
    generator.post_process()?;
    generator.write_output()?;

    Ok(())
}

pub struct CLIListener<'a, 'm> {
    config: &'a clap::ArgMatches<'m>,
    grammar_file: &'a Path,
}
impl CLIListener<'_, '_> {
    fn verbose(&self) -> bool {
        self.config.is_present("verbose")
    }
}
impl BuildListener for CLIListener<'_, '_> {
    fn on_initial_grammar_parse(
        &mut self,
        syntax_tree: &Tree<ParseTreeType>,
        parol_grammar: &ParolGrammar,
    ) -> miette::Result<()> {
        if self.verbose() {
            println!("{}", parol_grammar);
        }

        if let Some(file_name) = self.config.value_of("write_internal").as_ref() {
            let serialized = format!("{}", parol_grammar);
            fs::write(file_name, serialized)
                .into_diagnostic()
                .wrap_err("Error writing left-factored grammar!")?;
        }

        if self.config.is_present("generate_tree_graph") {
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
                if let Some(file_name) = self.config.value_of("write_untransformed") {
                    let serialized = render_par_string(grammar_config, false);
                    fs::write(file_name, serialized)
                        .into_diagnostic()
                        .wrap_err("Error writing untransformed grammar!")?;
                }
            }
            // final pass
            IntermediateGrammar::LAST => {
                if let Some(file_name) = self.config.value_of("expanded").as_ref() {
                    // NOTE: We still need special handling for writing to stdout
                    let lf_source = render_par_string(grammar_config, true);
                    if *file_name == "--" {
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
