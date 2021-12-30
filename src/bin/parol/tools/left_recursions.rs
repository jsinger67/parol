use miette::Result;
use parol::{detect_left_recursions, obtain_grammar_config};

pub fn main(args: &[&str]) -> Result<()> {
    if args.len() > 1 {
        let file_name = args[1].clone();
        let grammar_config = obtain_grammar_config(&file_name, false)?;
        let recursions = detect_left_recursions(&grammar_config.cfg);
        if recursions.is_empty() {
            println!("No left recursions found!\n");
        } else {
            println!("Found {} left recursions:\n", recursions.len());
            recursions.iter().for_each(|n| {
                let p = n
                    .iter()
                    .map(|s| format!("{}", s))
                    .collect::<Vec<String>>()
                    .join(" => ");
                println!("{}", p);
            });
        }
    } else {
        println!("Missing arguments <par-file>!");
        println!(
            "Example:\n\
            cargo run --bin {} ./src/parser/parol-grammar-exp.par",
            module_path!()
        );
    }
    Ok(())
}
