#[macro_use]
extern crate clap;

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::{env, fs};

use clap::{App, AppSettings, Arg, Parser};
use log::trace;
use miette::{miette, IntoDiagnostic, Result, WrapErr};

use id_tree::Tree;
use parol::{
    build::{BuildListener, IntermediateGrammar},
    render_par_string, GrammarConfig, ParolGrammar,
};
use parol_runtime::parser::ParseTreeType;

// static VERSION: &str = env!("CARGO_PKG_VERSION");

mod tools;

#[derive(Parser)]
#[clap(
    author = "Jörg Singer <singer.joerg@gmx.de>",
    version,
    about = "A LL(k) Parser Generator written in Rust.",
    long_about = None,
    setting(AppSettings::ArgsNegateSubcommands),
)]
struct ClapApp {
    /// Input grammar file
    #[clap(short = 'f', long = "file", parse(from_os_str))]
    grammar: Option<PathBuf>,

    /// Lookahead limit for Lookahead DFA calculation
    #[clap(short = 'k', long = "lookahead", default_value = "5")]
    lookahead: usize,

    /// Output file for the generated parser source
    #[clap(short = 'p', long = "parser", parse(from_os_str))]
    parser: Option<PathBuf>,

    /// Output file for the expanded grammar. Use -e-- to output to stdout
    #[clap(short = 'e', long = "expanded", parse(from_os_str))]
    expanded: Option<PathBuf>,

    /// Writes the internal parsed grammar (ParolGrammar)
    #[clap(short = 'i', long = "write_internal", parse(from_os_str))]
    write_internal: Option<PathBuf>,

    /// Writes the untransformed parsed grammar
    #[clap(short = 'u', long = "write_untransformed", parse(from_os_str))]
    write_untransformed: Option<PathBuf>,

    /// Writes the transformed parsed grammar
    #[clap(short = 'w', long = "write_transformed", parse(from_os_str))]
    write_transformed: Option<PathBuf>,

    /// Output file for the generated trait with semantic actions
    #[clap(short = 'a', long = "actions", parse(from_os_str))]
    actions: Option<PathBuf>,

    /// User type that implements the language processing
    #[clap(short = 't', long = "user_type")]
    user_type: Option<String>,

    /// User type's module name
    #[clap(short = 'm', long = "module")]
    module: Option<String>,

    /// Activates the generation of a SVG file with the parse tree of the given grammar
    #[clap(short = 's', long = "svg")]
    generate_tree_graph: bool,

    /// Increased verbosity
    #[clap(short = 'v', long = "verbose")]
    verbose: bool,

    #[clap(subcommand)]
    subcommand: Option<tools::ToolsSubcommands>,
}

fn clap_app() -> App<'static> {
    App::new("parol")
        .author("Jörg Singer <singer.joerg@gmx.de>")
        .about("A LL(k) Parser Generator written in Rust.")
        .arg(
            Arg::new("grammar")
                .help("Input grammar file")
                .short('f')
                .long("file")
                .takes_value(true),
        )
        .arg(
            Arg::new("lookahead")
                .help("Lookahead limit for Lookahead DFA calculation")
                .short('k')
                .long("lookahead")
                .takes_value(true),
        )
        .arg(
            Arg::new("parser")
                .help("Output file for the generated parser source")
                .short('p')
                .long("parser")
                .takes_value(true),
        )
        .arg(
            Arg::new("expanded")
                .help("Output file for the expanded grammar. Use -e-- to output to stdout")
                .short('e')
                .long("expanded")
                .takes_value(true),
        )
        .arg(
            Arg::new("write_internal")
                .help("Writes the internal parsed grammar (ParolGrammar)")
                .short('i')
                .long("write_internal")
                .takes_value(true),
        )
        .arg(
            Arg::new("write_untransformed")
                .help("Writes the untransformed parsed grammar")
                .short('u')
                .long("write_untransformed")
                .takes_value(true),
        )
        .arg(
            Arg::new("write_transformed")
                .help("Writes the transformed parsed grammar")
                .short('w')
                .long("write_transformed")
                .takes_value(true),
        )
        .arg(
            Arg::new("actions")
                .help("Output file for the generated trait with semantic actions")
                .short('a')
                .long("actions")
                .takes_value(true),
        )
        .arg(
            Arg::new("user_type")
                .help("User type that implements the language processing")
                .short('t')
                .long("user_type")
                .takes_value(true),
        )
        .arg(
            Arg::new("module")
                .help("User type's module name")
                .short('m')
                .long("module")
                .takes_value(true)
        )
        .arg(
            Arg::new("generate_tree_graph")
                .help("Activates the generation of a SVG file with the parse tree of the given grammar")
                .short('s')
                .long("svg")
        )
        .arg(
            Arg::new("verbose")
                .help("Increased verbosity")
                .short('v')
                .long("verbose")
        )
}

// To rebuild the parser sources from scratch use the command build_parsers.ps1

fn main() -> Result<()> {
    env_logger::try_init().into_diagnostic()?;
    trace!("env logger started");

    // let config = clap_app()
    /*
     * We want all our "tools" to be registered as subcommands.
     *
     * Doing this allows `clap` to give better help and error messages
     * then if we used AppSettings::AllowExternalSubcommands
     */
    // .subcommands(tools::names().map(|name| {
    //     /*
    //      * For now, our subcommands have no names or descriptions
    //      *
    //      * They all accept infinite args (clap makes no attempt to validate things here).
    //      */
    //     tools::get_tool_sub_command(name).unwrap()()
    // }))

    // Only invoke tools if they come first, to avoid ambiguity with main binary

    // .setting(AppSettings::ArgsNegateSubcommands)
    // .version(VERSION)
    // .get_matches();

    // if let Some((subcommand_name, sub_matches)) = config.subcommand() {
    //     let tool_main =
    //         tools::get_tool_main(subcommand_name).expect("Clap should've validated tool name");
    //     log::debug!("Delegating to {} with {:?}", subcommand_name, sub_matches);
    //     return tool_main(sub_matches);
    // }

    // new:
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
