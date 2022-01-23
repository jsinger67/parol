use crate::PathBuf;

use clap::{AppSettings, Parser};

#[derive(Parser)]
#[clap(
    author = "Jörg Singer <singer.joerg@gmx.de>",
    version,
    about = "A LL(k) Parser Generator written in Rust.",
    long_about = None,
    setting(AppSettings::ArgsNegateSubcommands),
)]
pub(crate) struct ClapApp {
    /// Input grammar file
    #[clap(short = 'f', long = "file", parse(from_os_str))]
    pub grammar: Option<PathBuf>,

    /// Lookahead limit for Lookahead DFA calculation
    #[clap(short = 'k', long = "lookahead", default_value = "5")]
    pub lookahead: usize,

    /// Output file for the generated parser source
    #[clap(short = 'p', long = "parser", parse(from_os_str))]
    pub parser: Option<PathBuf>,

    /// Output file for the expanded grammar. Use -e-- to output to stdout
    #[clap(short = 'e', long = "expanded", parse(from_os_str))]
    pub expanded: Option<PathBuf>,

    /// Writes the internal parsed grammar (ParolGrammar)
    #[clap(short = 'i', long = "write_internal", parse(from_os_str))]
    pub write_internal: Option<PathBuf>,

    /// Writes the untransformed parsed grammar
    #[clap(short = 'u', long = "write_untransformed", parse(from_os_str))]
    pub write_untransformed: Option<PathBuf>,

    /// Writes the transformed parsed grammar
    #[clap(short = 'w', long = "write_transformed", parse(from_os_str))]
    pub write_transformed: Option<PathBuf>,

    /// Output file for the generated trait with semantic actions
    #[clap(short = 'a', long = "actions", parse(from_os_str))]
    pub actions: Option<PathBuf>,

    /// User type that implements the language processing
    #[clap(short = 't', long = "user_type")]
    pub user_type: Option<String>,

    /// User type's module name
    #[clap(short = 'm', long = "module")]
    pub module: Option<String>,

    /// Activates the generation of a SVG file with the parse tree of the given grammar
    #[clap(short = 's', long = "svg")]
    pub generate_tree_graph: bool,

    /// Increased verbosity
    #[clap(short = 'v', long = "verbose")]
    pub verbose: bool,

    #[clap(subcommand)]
    pub subcommand: Option<super::tools::ToolsSubcommands>,
}
