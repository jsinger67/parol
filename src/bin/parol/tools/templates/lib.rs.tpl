{{#tree_gen?}}use parol_runtime::id_tree::Tree;
use parol_runtime::id_tree_layout::Layouter;
use parol_runtime::parser::ParseTreeType;{{/tree_gen}}

extern crate parol_runtime;

mod {{crate_name}}_grammar;
pub use {{crate_name}}_grammar::{{grammar_name}}Grammar;

mod {{crate_name}}_grammar_trait;
pub use {{crate_name}}_grammar_trait::ASTType;

mod {{crate_name}}_parser;
pub use {{crate_name}}_parser::parse;

{{#tree_gen?}}pub fn generate_tree_layout(
    syntax_tree: &Tree<ParseTreeType>,
    input_file_name: &str,
) -> id_tree_layout::layouter::Result {
    let mut svg_full_file_name = std::path::PathBuf::from(input_file_name);
    svg_full_file_name.set_extension("svg");

    Layouter::new(syntax_tree)
        .with_file_path(&svg_full_file_name)
        .write()
}
{{/tree_gen}}
