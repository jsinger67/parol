use miette::Result;
use parol::obtain_grammar_config;
use std::env;

fn main() -> Result<()> {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let file_name = args[1].clone();
        let grammar_config = obtain_grammar_config(&file_name, false)?;
        let serialized = serde_json::to_string(&grammar_config).unwrap();
        println!("{}", serialized);
        let cfg_ext1 = serde_json::from_str(&serialized).unwrap();
        assert_eq!(grammar_config, cfg_ext1);
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
