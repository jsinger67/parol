use derive_builder::Builder;

#[derive(Builder, Debug, Default)]
pub(crate) struct MainRsData<'a> {
    crate_name: &'a str,
    grammar_name: String,
    tree_gen: bool,
}

impl std::fmt::Display for MainRsData<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let MainRsData {
            crate_name,
            grammar_name,
            tree_gen,
        } = self;

        f.write_fmt(ume::ume!(
            extern crate parol_runtime;
        ))?;

        write!(f, "\n\n")?;

        write!(
            f,
            "\
mod {crate_name}_grammar;
// The output is version controlled
mod {crate_name}_grammar_trait;
mod {crate_name}_parser;
"
        )?;

        write!(f, "\n\n")?;

        write!(
            f,
            "\
use crate::{crate_name}_grammar::{grammar_name}Grammar;
use crate::{crate_name}_parser::parse;
"
        )?;
        if *tree_gen {
            f.write_fmt(ume::ume! {
                use parol_runtime::ParseTree;
                use parol_runtime::syntree_layout::Layouter;
            })?;
        }
        f.write_fmt(ume::ume! {
            use parol_runtime::{log::debug, Report};
            use anyhow::{anyhow, Context, Result};
            use std::{env, fs, time::Instant};
        })?;

        write!(f, "\n\n")?;

        write!(f, "\
// To generate:
// parol -f ./{crate_name}.par -e ./{crate_name}-exp.par -p ./src/{crate_name}_parser.rs -a ./src/{crate_name}_grammar_trait.rs -t {grammar_name}Grammar -m {crate_name}_grammar
")?;

        write!(
            f,
            "\n
        struct ErrorReporter;
        impl Report for ErrorReporter {{}}
\n"
        )?;

        let grammar = format!("{grammar_name}Grammar");
        let crate_name_grammar = format!("{crate_name}_grammar");
        let syntax_tree = if *tree_gen { "syntax_tree" } else { "_" };
        let generate_tree_layout = if *tree_gen {
            ume::ume!(generate_tree_layout(&syntax_tree, &input, &file_name)?;).to_string()
        } else {
            "".into()
        };
        let blank_line = "\n\n";
        f.write_fmt(ume::ume! {
            fn main() -> Result<()> {
                env_logger::init();
                debug!("env logger started");
                #blank_line
                let args: Vec<String> = env::args().collect();
                if args.len() >= 2 {
                    let file_name = args[1].clone();
                    let input = fs::read_to_string(file_name.clone())
                        .with_context(|| format!("Can't read file {}", file_name))?;
                    let mut #crate_name_grammar = #grammar::new();
                    let now = Instant::now();
                    match parse(&input, &file_name, &mut #crate_name_grammar) {
                        Ok(#syntax_tree) => {
                            let elapsed_time = now.elapsed();
                            println!("Parsing took {} milliseconds.", elapsed_time.as_millis());
                            if args.len() > 2 && args[2] == "-q" {
                                Ok(())
                            } else {
                                #generate_tree_layout
                                println!("Success!\n{}", #crate_name_grammar);
                                Ok(())
                            }
                        }
                        Err(e) => ErrorReporter::report_error(&e, file_name),
                    }
                } else {
                    Err(anyhow!("Please provide a file name as first parameter!"))
                }
            }
        })?;

        if *tree_gen {
            write!(f, "\n\n")?;

            f.write_fmt(ume::ume! {
                fn generate_tree_layout(syntax_tree: &ParseTree, input: &str, input_file_name: &str) -> parol_runtime::syntree_layout::Result<()> {
                    let mut svg_full_file_name = std::path::PathBuf::from(input_file_name);
                    svg_full_file_name.set_extension("svg");

                    Layouter::new(syntax_tree)
                        .with_file_path(&svg_full_file_name)
                        .embed_with_source_and_display(input)?
                        .write()
                }
            })?;
        }
        Ok(())
    }
}
