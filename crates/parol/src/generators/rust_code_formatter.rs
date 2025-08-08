use anyhow::{Result, anyhow};
use std::path::Path;

use std::process::Command;

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
/// Tries to format the source code of a given file.
pub fn try_format(path_to_file: &Path) -> Result<()> {
    Command::new("rustfmt")
        .args([path_to_file])
        .args(["--edition", "2024"])
        .status()
        .map(|_| ())
        .map_err(|e| anyhow!("Error during source formatting!: {}", e))
}
