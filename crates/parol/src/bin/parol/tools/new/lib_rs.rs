use derive_builder::Builder;

#[derive(Builder, Debug, Default)]
pub struct LibRsData<'a> {
    crate_name: &'a str,
    grammar_name: String,
    tree_gen: bool,
}

impl std::fmt::Display for LibRsData<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let LibRsData {
            crate_name,
            grammar_name,
            tree_gen,
        } = self;

        if *tree_gen {
            f.write_fmt(ume::ume! {
                use parol_runtime::{ParseTree, syntree_layout::Layouter};
            })?;

            write!(f, "\n\n")?;
        }

        f.write_fmt(ume::ume!(
            extern crate parol_runtime;
        ))?;

        write!(f, "\n\n")?;

        write!(
            f,
            "\
mod {crate_name}_grammar;
pub use {crate_name}_grammar::{grammar_name}Grammar;

mod {crate_name}_grammar_trait;
pub use {crate_name}_grammar_trait::ASTType;

mod {crate_name}_parser;
pub use {crate_name}_parser::parse;
"
        )?;

        if *tree_gen {
            write!(f, "\n\n")?;
            f.write_fmt(ume::ume! {
                pub fn generate_tree_layout(syntax_tree: &ParseTree, input: &str, input_file_name: &str) -> parol_runtime::syntree_layout::Result<()> {
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
