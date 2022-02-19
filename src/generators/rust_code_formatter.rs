use miette::{IntoDiagnostic, Result};
use std::path::Path;

cfg_if::cfg_if! {
    if #[cfg(feature = "prettyplease")] {
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
    } else {
use std::process::Command;
    }
}

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
///
pub fn try_format(path_to_file: &Path) -> Result<()> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "prettyplease")] {
            let mut file = File::open(path_to_file).into_diagnostic()?;
            let mut content = String::new();
            file.read_to_string(&mut content).into_diagnostic()?;

            let syntax_tree = syn::parse_file(&content).into_diagnostic()?;
            let formatted = prettyplease::unparse(&syntax_tree);

            file = OpenOptions::new()
                .write(true)
                .open(path_to_file)
                .into_diagnostic()?;

            file.write_all(formatted.as_bytes())
                .map(|_| ())
                .into_diagnostic()
        } else {
            Command::new("rustfmt")
                .args(&[path_to_file])
                .status()
                .map(|_| ())
                .into_diagnostic()
        }
    }
}
