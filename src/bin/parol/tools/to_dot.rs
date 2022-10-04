use miette::Result;
use petgraph::dot::Dot;
use std::path::PathBuf;

/// Generates a graph representation of the given grammar
#[derive(clap::Parser)]
#[clap(name = "to_dot")]
pub struct Args {
    /// The grammar file to use
    #[clap(short = 'f', long = "grammar-file", parse(from_os_str))]
    grammar_file: PathBuf,
    /// Use NtGrammarGraph's dot representation instead
    #[clap(short = 'n', long = "nt-grammar-graph")]
    use_nt_grammar_graph: bool,
}

pub fn main(args: &Args) -> Result<()> {
    let file_name = &args.grammar_file;
    let grammar_config = parol::obtain_grammar_config(&file_name, false)?;
    let dot = 
    if args.use_nt_grammar_graph {
        let nt_graph: parol::NtGrammarGraph = (&grammar_config.cfg).into();
        format!("{:?}", Dot::with_config(&nt_graph.0, &[]))
    } else {
        parol::render_nt_dot_string(&grammar_config)
    };
    println!("{}", dot);
    Ok(())
}
