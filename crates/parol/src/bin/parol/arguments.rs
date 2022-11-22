use crate::PathBuf;

use clap::Parser;

// LL(k) Parser Generator written in Rust
#[derive(Parser)]
#[clap(author, version, about)]
pub(crate) struct ClapApp {
    /// Input grammar file
    #[clap(short = 'f', long = "file")]
    pub grammar: Option<PathBuf>,

    /// Lookahead limit for Lookahead DFA calculation
    #[clap(short = 'k', long = "lookahead", default_value = "5")]
    pub lookahead: usize,

    /// Output file for the generated parser source
    #[clap(short = 'p', long = "parser")]
    pub parser: Option<PathBuf>,

    /// Output file for the expanded grammar. Use -e-- to output to stdout
    #[clap(short = 'e', long = "expanded")]
    pub expanded: Option<PathBuf>,

    /// Writes the internal parsed grammar (ParolGrammar)
    #[clap(short = 'i', long = "write_internal")]
    pub write_internal: Option<PathBuf>,

    /// Writes the untransformed parsed grammar
    #[clap(short = 'u', long = "write_untransformed")]
    pub write_untransformed: Option<PathBuf>,

    /// Writes the transformed parsed grammar
    #[clap(short = 'w', long = "write_transformed")]
    pub write_transformed: Option<PathBuf>,

    /// Output file for the generated trait with semantic actions
    #[clap(short = 'a', long = "actions")]
    pub actions: Option<PathBuf>,

    /// User type that implements the language processing
    #[clap(short = 't', long = "user_type")]
    pub user_type: Option<String>,

    /// User type's module name
    #[clap(short = 'm', long = "module")]
    pub module: Option<String>,

    /// Activates the auto-generation of semantic actions in expanded grammar - recommended
    #[clap(short = 'g', long = "auto_generate")]
    pub auto_generate: bool,

    /// Activates the generation of a SVG file with the parse tree of the given grammar
    #[clap(short = 's', long = "svg")]
    pub generate_tree_graph: bool,

    /// Increased verbosity
    #[clap(short = 'v', long = "verbose")]
    pub verbose: bool,

    #[clap(subcommand)]
    pub subcommand: Option<super::tools::ToolsSubcommands>,
}
