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
            use parol_runtime::id_tree::Tree;
            use parol_runtime::id_tree_layout::Layouter;
            use parol_runtime::parser::ParseTreeType;
                    })?;
        }
        f.write_fmt(ume::ume! {
        use parol_runtime::log::debug;
        use parol_runtime::miette::{miette, IntoDiagnostic, Result, WrapErr};
        use std::env;
        use std::fs;
        use std::time::Instant;
                })?;

        write!(f, "\n\n")?;

        write!(f, "\
// To generate:
// parol -f ./{crate_name}.par -e ./{crate_name}-exp.par -p ./src/{crate_name}_parser.rs -a ./src/{crate_name}_grammar_trait.rs -t {grammar_name}Grammar -m {crate_name}_grammar -g
")?;

        write!(f, "\n\n")?;

        let grammar = format!("{grammar_name}Grammar");
        let crate_name_grammar = format!("{crate_name}_grammar");
        let let_syntax_tree = if *tree_gen { "let syntax_tree = " } else { "" };
        let generate_tree_layout = if *tree_gen {
            ume::ume!(generate_tree_layout(&syntax_tree, &file_name)?;).to_string()
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
                        .into_diagnostic()
                        .wrap_err(format!("Can't read file {}", file_name))?;
                    let mut #crate_name_grammar = #grammar::new();
                    let now = Instant::now();
                    #let_syntax_tree parse(&input, &file_name, &mut #crate_name_grammar)
                        .wrap_err(format!("Failed parsing file {}", file_name))?;
                    let elapsed_time = now.elapsed();
                    println!("Parsing took {} milliseconds.", elapsed_time.as_millis());
                    if args.len() > 2 && args[2] == "-q" {
                        Ok(())
                    } else {
                        #generate_tree_layout
                        println!("Success!\n{}", #crate_name_grammar);
                        Ok(())
                    }
                } else {
                    Err(miette!("Please provide a file name as first parameter!"))
                }
            }
        })?;

        if *tree_gen {
            write!(f, "\n\n")?;

            f.write_fmt(ume::ume! {
				fn generate_tree_layout(syntax_tree: &Tree<ParseTreeType>, input_file_name: &str) -> Result<()> {
					let mut svg_full_file_name = std::path::PathBuf::from(input_file_name);
					svg_full_file_name.set_extension("svg");

					Layouter::new(syntax_tree)
						.with_file_path(&svg_full_file_name)
						.write()
						.into_diagnostic()
						.wrap_err("Failed writing layout")
				}
			})?;
        }
        Ok(())
    }
}
