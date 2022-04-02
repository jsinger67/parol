use miette::{miette, IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use parol::generators::NamingHelper as NmHlp;
use std::path::PathBuf;
use std::process::Command;

/// Create a new `parol` package
#[derive(clap::Parser)]
#[clap(name = "new")]
pub struct Args {
    /// The directory where to create the new package
    #[clap(short, long, parse(from_os_str))]
    path: PathBuf,

    /// The new package should be a binary executable
    #[clap(short, long)]
    bin: bool,

    /// The new package should be a library
    #[clap(short, long)]
    lib: bool,

    /// The name of the new package. Defaults to the directory name.
    #[clap(short, long)]
    name: Option<String>,
}

pub fn main(args: &Args) -> Result<()> {
    apply_cargo(args)?;

    let crate_name = if let Some(name) = args.name.as_ref() {
        name
    } else {
        args.path
            .as_path()
            .file_name()
            .ok_or(miette!("Trouble to handle path"))?
            .to_str()
            .ok_or(miette!("Trouble to handle path"))?
    };

    let grammar_name = NmHlp::to_upper_camel_case(crate_name);

    print!(
        "Generating crate {} for grammar {}...",
        crate_name.green(),
        grammar_name.green()
    );
    Ok(())
}

const DEPENDENCIES: &[&[&'static str]; 8] = &[
    &["add", "derive_builder", "--vers=0.11.1"],
    &["add", "env_logger", "--vers=0.9.0"],
    &["add", "id_tree", "--vers=^1.8"],
    &["add", "lazy_static", "--vers=^1.4"],
    &["add", "log", "--vers=0.4.14"],
    &["add", "miette", "--vers=^4.0", "--features", "fancy"],
    &["add", "parol_runtime", "--vers=0.5.9"],
    &["add", "thiserror", "--vers=^1.0"],
];

fn apply_cargo(args: &Args) -> Result<()> {
    if args.bin && args.lib {
        return Err(miette!(
            "Please use only one of `bin` and `lib` and not both!"
        ));
    }

    // Prepare arguments for the `cargo new` command
    let mut cargo_args = vec!["new"];
    if args.bin {
        cargo_args.push("--bin");
    }
    if args.lib {
        cargo_args.push("--lib");
    }
    if let Some(name) = args.name.as_ref() {
        cargo_args.push("--name");
        cargo_args.push(&name);
    }
    cargo_args.push(args.path.to_str().ok_or(miette!("Please provide a path"))?);

    // Call the `cargo new` command
    Command::new("cargo")
        .args(&cargo_args)
        .status()
        .map(|_| ())
        .into_diagnostic()?;

    // Add dependencies
    DEPENDENCIES
        .iter()
        .fold(Ok(()), |res: Result<()>, cargo_args| {
            res?;
            Command::new("cargo")
                .current_dir(&args.path)
                .args(*cargo_args)
                .status()
                .map(|_| ())
                .into_diagnostic()
        })?;

    Ok(())
}
