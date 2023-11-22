use crate::{generators::NamingHelper as NmHlp, obtain_grammar_config, LanguageGenerator};
use assert_cmd::Command;
use std::io::Write;
use std::path::Path;
use tempfile::NamedTempFile;

/// This function is used for acceptor testing of a given grammar.
///
/// ATTENTION!
/// Please note that this function should only be used in test code and NOT IN PRODUCTION CODE
/// because due to panics the executing process could unexpectedly terminate.
///
/// This function will generate an acceptor for the given grammar description.
/// Randomly created input that formally adheres to the grammar description is then frequently
/// fed to the acceptor.
///
/// Note, that error handling is minimalistic as you would do it when you write test cases.
///
/// Please, see test below for a practical application.
///
pub fn acceptor_test<T>(
    grammar_file_path: T,
    test_count: usize,
    generated_max_length: Option<usize>,
) where
    T: AsRef<Path>,
{
    let input = grammar_file_path.as_ref();
    // Extract the grammar name from the file name
    let upper_case_grammar_name = NmHlp::to_upper_camel_case(
        input
            .file_stem()
            .expect("Expecting a valid file name")
            .to_str()
            .expect("Can't handle OS specific file name"),
    );
    let module_name = NmHlp::to_lower_snake_case(
        input
            .file_stem()
            .expect("Expecting a valid file name")
            .to_str()
            .expect("Can't handle OS specific file name"),
    );
    // Create a temp dir
    let temp_dir = tempfile::Builder::new()
        .prefix("acceptor_test_")
        .tempdir()
        .expect("Couldn't create a directory inside of `std::env::temp_dir()");
    Command::cargo_bin("parol")
        .unwrap()
        .current_dir(&temp_dir)
        .args([
            "new",
            "--bin",
            "--path",
            &module_name,
            "--name",
            &upper_case_grammar_name,
        ])
        .assert()
        .success();
    let generated_crate_dir = Path::new(temp_dir.as_ref()).join(&module_name);
    Command::new("cargo")
        .current_dir(&generated_crate_dir)
        .arg("build")
        .assert()
        .success();

    let grammar_config =
        obtain_grammar_config(input, false).expect("Error compiling input grammar");
    let mut generator = LanguageGenerator::new(&grammar_config.cfg);

    (0..test_count).for_each(|_| {
        let generated_source = generator
            .generate(generated_max_length)
            .expect("Error generating random source");
        let mut file = NamedTempFile::new().expect("Couldn't create temporary file");
        write!(file, "{}", generated_source).expect("Error writing generated source");
        Command::new("cargo")
            .current_dir(&generated_crate_dir)
            .arg("run")
            .arg(file.path())
            .assert()
            .success();
    });
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use super::acceptor_test;

    // To run this test, execute the following command:
    // cargo test -- --ignored --test test_acceptor_test
    #[test]
    #[ignore = "Long running test!"]
    fn test_acceptor_test() {
        // Please note that this environment variable is only set if cargo executes the tests!
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let grammar_file_path = Path::new(&manifest_dir)
            .join("..")
            .join("..")
            .join("examples")
            .join("list")
            .join("list.par");
        acceptor_test(grammar_file_path, 250, Some(100_000));
    }
}
