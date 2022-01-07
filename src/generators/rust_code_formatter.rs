use std::path::Path;
use std::process::Command;

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
///
/// Uses rust's rustfmt command.
/// Fails if rustfmt can't be called successfully.
///
pub fn try_format(path_to_file: &Path) {
    let result = Command::new("rustfmt").args(&[path_to_file]).status();
    if let Err(e) = result {
        println!("Failed to format source {}: {}", path_to_file.display(), e);
    }
}
