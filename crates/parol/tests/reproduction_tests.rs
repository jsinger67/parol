use anyhow::Result;
use parol::parser::{parse, ParolGrammar};
use regex::Regex;

use std::fs;
use std::path;

///
/// Checks the internal representation after parsing.
/// The internal representation is a ParolGrammar struct.
///
#[test]
fn reproduction_test() -> Result<()> {
    let rx_newline: Regex = Regex::new(r"\r?\n|\r").unwrap();
    for file_result in path::PathBuf::from("./tests/data/valid").read_dir()? {
        let dir_entry = file_result?;
        let mut file_path = dir_entry.path();
        if file_path.extension().unwrap() == "par" {
            println!("Checking {}", file_path.display());
            let input = fs::read_to_string(&file_path)?;
            let mut parol_grammar = ParolGrammar::new();
            parse(&input, &file_path, &mut parol_grammar)?;
            let representation = format!("{parol_grammar}");
            file_path.set_extension("expected");
            let expected = fs::read_to_string(&file_path)?;
            assert_eq!(
                rx_newline.replace_all(&expected, "\n"),
                rx_newline.replace_all(&representation, "\n"),
                "parse result mismatch!"
            );
        }
    }
    Ok(())
}
