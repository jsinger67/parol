use parol::build::Builder;

fn main() {
    // CLI equivalent is:
    // parol -f ./{{crate_name}}.par -e ./{{crate_name}}-exp.par -p ./src/{{crate_name}}_parser.rs -a ./src/{{crate_name}}_grammar_trait.rs -t {{grammar_name}}Grammar -m {{crate_name}}_grammar -g
    Builder::with_explicit_output_dir("src")
        .grammar_file("{{crate_name}}.par")
        .expanded_grammar_output_file("../{{crate_name}}-exp.par")
        .parser_output_file("{{crate_name}}_parser.rs")
        .actions_output_file("{{crate_name}}_grammar_trait.rs")
        .enable_auto_generation()
        .user_type_name("{{grammar_name}}Grammar")
        .user_trait_module_name("{{crate_name}}_grammar")
        .generate_parser()
        .unwrap();
}
