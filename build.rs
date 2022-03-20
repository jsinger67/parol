use parol::build::Builder;

fn main() {
    if cfg!(feature = "use-cargo-output") {
        // This is the recommended default for most users,
        // it puts the output in the $OUT_DIR instead of in version control
        //
        // See sources for how to include this
        Builder::with_cargo_script_output()
            .grammar_file("json.par")
            // NOTE: You can use `parser_output_file` and friends
            // to override the generated file names
            //
            // However, it is optional.
            .enable_auto_generation()
            .user_type_name("JsonGrammar")
            .user_trait_module_name("json_grammar")
            .generate_parser()
            .unwrap();
    } else {
        // CLI equivalent
        // parol -f ./json.par -e ./json-exp.par -p ./src/json_parser.rs -a ./src/json_grammar_trait.rs -t JsonGrammar -m json_grammar -g

        // Unlike Builder::with_cargo_output, explicit names for the parser and actions files are
        // required!
        Builder::with_explicit_output_dir("src")
            .grammar_file("json.par")
            .expanded_grammar_output_file("../json-exp.par")
            .parser_output_file("json_parser.rs")
            .actions_output_file("json_grammar_trait.rs")
            .enable_auto_generation()
            .user_type_name("JsonGrammar")
            .user_trait_module_name("json_grammar")
            .generate_parser()
            .unwrap();
    }
}
