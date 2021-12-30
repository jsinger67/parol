use miette::Result;
use parol::obtain_grammar_config;

pub fn main(args: &[&str]) -> Result<()> {
    if args.len() > 1 {
        let file_name = args[1].to_owned();
        let grammar_config = obtain_grammar_config(&file_name, false)?;
        let serialized = serde_json::to_string(&grammar_config).unwrap();
        println!("{}", serialized);
        let cfg_ext1 = serde_json::from_str(&serialized).unwrap();
        assert_eq!(grammar_config, cfg_ext1);
    } else {
        println!("Missing arguments <par-file>!");
        println!(
            "Example:\n\
            cargo run --bin parol serialize ./src/parser/parol-grammar-exp.par"
        );
    }
    Ok(())
}
