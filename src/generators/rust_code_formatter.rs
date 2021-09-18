use std::process::Command;

pub fn try_format(path_to_file: &str) {
    let result = Command::new("rustfmt").args(&[path_to_file]).status();
    if let Err(e) = result {
        println!("Failed to format source {}: {}", path_to_file, e);
    }
}
