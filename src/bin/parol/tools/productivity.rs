use miette::Result;
use parol::analysis::non_productive_non_terminals;
use parol::obtain_grammar_config;

pub fn main(args: &[&str]) -> Result<()> {
    // Logger already initialized
    // env_logger::init();
    // $env:RUST_LOG="parol,parol_runtime=off,productivity=debug"
    // debug!("env logger started");

    if args.len() < 2 {
        println!("Missing arguments <par-file>!");
        println!(
            "Example:\n\
            cargo run --bin {} ./src/parser/parol-grammar-exp.par",
            module_path!()
        );
    } else {
        let file_name = args[1].clone();
        let grammar_config = obtain_grammar_config(&file_name, false)?;

        let non_productive_non_terminals = non_productive_non_terminals(&grammar_config.cfg);
        if non_productive_non_terminals.is_empty() {
            println!("No non-productive non-terminals found!");
        } else {
            println!("Non-productive non-terminals:");
            for nt in non_productive_non_terminals {
                println!("  {}", nt);
            }
        }
    }
    Ok(())
}
