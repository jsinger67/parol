#[macro_use]
extern crate clap;

use std::path::{Path, PathBuf};
use std::{env, fs};

use clap::{App, AppSettings, Arg, SubCommand};
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
            SubCommand::with_name(name).arg(Arg::with_name("args").index(1).multiple(true))
        }))
        // Only invoke tools if they come first, to avoid ambiguity with main binary
        .setting(AppSettings::ArgsNegateSubcommands)
        .version(VERSION)
        .get_matches();

    if let (subcommand_name, Some(sub_matches)) = config.subcommand() {
        let mut ext_args: Vec<&str> = sub_matches
            .values_of("args")
            .map_or_else(Vec::default, |args| args.collect());
        /*
         * All of the tools were originally written using `env::args()` meaning they expect tool name to be
         * first.
         *
         * Therefore they expect first argument at index 1 instead of zero.
         * Fake a command name to avoid changing all the indices
         */
        ext_args.insert(0, subcommand_name);
        let tool_main =
            tools::get_tool_main(subcommand_name).expect("Clap should've validated tool name");
        log::debug!("Delegating to {} with {:?}", subcommand_name, ext_args);
        return tool_main(&ext_args);
    }

    // If relative paths are spsecified, they should be resoled relative to the current directory
    let mut builder = parol::build::Builder::with_output_dir(env::current_dir().into_diagnostic()?);

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

    // NOTE: only-lookahead appears to have been broken (even before this commit).
    // See issue #2
    if !config.is_present("parser") && !config.is_present("only_lookahead") {
        return Ok(());
    }

    generator.post_process()?;
    generator.write_output()?;

    Ok(())
}

pub struct CLIListener<'a, 'm> {
    config: &'a clap::ArgMatches<'m>,
    grammar_file: &'a Path,
}
impl CLIListener<'_, '_> {
    fn vebrose(&self) -> bool {
        self.config.is_present("verbose")
    }
}
impl BuildListener for CLIListener<'_, '_> {
    fn on_initial_grammar_parse(
        &mut self,
        syntax_tree: &Tree<ParseTreeType>,
        parol_grammar: &ParolGrammar,
    ) -> miette::Result<()> {
        if self.vebrose() {
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
