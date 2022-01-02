use miette::{IntoDiagnostic, Result, WrapErr};
use parol::{obtain_grammar_config, LanguageGenerator};

// $env:RUST_LOG="parol::generators::language_generator=trace"

pub fn main(args: &[&str]) -> Result<()> {
    if args.len() > 1 {
        let file_name = args[1].to_owned();
        let grammar_config = obtain_grammar_config(&file_name, false)?;
        let mut generator = LanguageGenerator::new(&grammar_config.cfg);
        let max_sentence_length = if args.len() > 2 {
            args[2]
                .parse::<usize>()
                .into_diagnostic()
                .wrap_err("Provide a valid usize value for second argument")
                .map(Some)?
        } else {
            None
        };
        let result = generator.generate(max_sentence_length)?;
        println!("{}", result);
    } else {
        println!("Missing arguments <par-file> <max_sentence_length = 100000>!");
        println!(
            "Example:\n\
            parol generate ./src/parser/parol-grammar-exp.par"
        );
    }
    Ok(())
}
