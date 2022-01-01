use miette::{IntoDiagnostic, Result};
use parol::{obtain_grammar_config, LanguageGenerator};

// $env:RUST_LOG="parol::generators::language_generator=trace"  

pub fn main(args: &[&str]) -> Result<()> {
    if args.len() > 1 {
        let file_name = args[1].to_owned();
        let grammar_config = obtain_grammar_config(&file_name, false)?;
        let max_repeat = if args.len() > 2 {
            args[2].parse::<u32>().into_diagnostic()?
        } else {
            10
        };
        let mut generator = LanguageGenerator::new(&grammar_config.cfg);
        let mut buffer = vec![];
        generator.generate(&mut buffer, max_repeat)?;
        let output = String::from_utf8(buffer).unwrap();
        println!("{}", output);
    } else {
        println!("Missing arguments <par-file> <max_rx_repeat>!");
        println!(
            "Example:\n\
            parol generate ./src/parser/parol-grammar-exp.par 10"
        );
    }
    Ok(())
}
