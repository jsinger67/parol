use clap::ArgGroup;
use derive_builder::Builder;
use miette::{miette, Context, IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use parol::generators::NamingHelper as NmHlp;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Creates a new crate that uses `parol`.
#[derive(clap::Parser)]
#[clap(name = "new")]
#[clap(group(ArgGroup::new("lib_or_bin").args(&["lib", "bin"]).multiple(false).required(true)))]
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

#[derive(Debug, Builder)]
struct CreationData<'a> {
    crate_name: &'a str,
    grammar_name: String,
    path: PathBuf,
    is_bin: bool,
}

pub fn main(args: &Args) -> Result<()> {
    apply_cargo(args)?;

    let crate_name = NmHlp::purge_name(if let Some(name) = args.name.as_ref() {
        name
    } else {
        args.path
            .as_path()
            .file_name()
            .ok_or_else(|| miette!("Trouble to handle path"))?
            .to_str()
            .ok_or_else(|| miette!("Trouble to handle path"))?
    });

    let creation_data = CreationDataBuilder::default()
        .crate_name(&crate_name)
        .grammar_name(NmHlp::to_upper_camel_case(&crate_name))
        .path(args.path.clone())
        .is_bin(args.bin)
        .build()
        .into_diagnostic()?;

    print!(
        "Generating crate {} for grammar {}...",
        creation_data.crate_name.green(),
        creation_data.grammar_name.green()
    );

    generate_bin_crate(creation_data)?;

    Ok(())
}

const DEPENDENCIES: &[&[&str]] = &[
    &["add", "derive_builder", "--vers=0.11.1"],
    &["add", "env_logger", "--vers=0.9.0"],
    &["add", "function_name"],
    &["add", "id_tree", "--vers=^1.8"],
    &["add", "lazy_static", "--vers=^1.4"],
    &["add", "log", "--vers=0.4.14"],
    &["add", "miette", "--vers=^4.0", "--features", "fancy"],
    &["add", "parol_runtime", "--vers=0.5.9"],
    &["add", "thiserror", "--vers=^1.0"],
    &["add", "parol", "--build", "--vers=^0.8.1"],
];

fn apply_cargo(args: &Args) -> Result<()> {
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
        cargo_args.push(name);
    }
    cargo_args.push(args.path.to_str().ok_or_else(|| miette!("Please provide a path"))?);

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
                .wrap_err("Maybe you have to install cargo-edit: `cargo install cargo-edit`?")
        })?;

    Ok(())
}

fn generate_bin_crate(creation_data: CreationData) -> Result<()> {
    generate_build_rs(&creation_data)?;
    generate_grammar_par(&creation_data)?;
    if creation_data.is_bin {
        generate_main_rs(&creation_data)?;
    } else {
        generate_lib_rs(&creation_data)?;
    }
    generate_grammar_rs(&creation_data)?;

    Ok(())
}

#[derive(BartDisplay, Builder, Debug, Default)]
#[template = "src/bin/parol/tools/templates/build.rs.tpl"]
struct BuildRsData<'a> {
    crate_name: &'a str,
    grammar_name: String,
}

fn generate_build_rs(creation_data: &CreationData) -> Result<()> {
    let mut build_file_out = creation_data.path.clone();
    build_file_out.push("build.rs");
    let build_data = BuildRsDataBuilder::default()
        .crate_name(creation_data.crate_name)
        .grammar_name(NmHlp::to_upper_camel_case(creation_data.crate_name))
        .build()
        .into_diagnostic()?;
    let build_source = format!("{}", build_data);
    fs::write(build_file_out, build_source)
        .into_diagnostic()
        .wrap_err("Error writing generated user trait source!")?;

    Ok(())
}

#[derive(BartDisplay, Builder, Debug, Default)]
#[template = "src/bin/parol/tools/templates/grammar.par"]
struct GrammarParData {
    grammar_name: String,
}

fn generate_grammar_par(creation_data: &CreationData) -> Result<()> {
    let mut grammar_file_out = creation_data.path.clone();
    grammar_file_out.push(format!("{}.par", creation_data.crate_name));
    let grammar_data = GrammarParDataBuilder::default()
        .grammar_name(NmHlp::to_upper_camel_case(creation_data.crate_name))
        .build()
        .into_diagnostic()?;
    let grammar_source = format!("{}", grammar_data);
    fs::write(grammar_file_out, grammar_source)
        .into_diagnostic()
        .wrap_err("Error writing generated user trait source!")?;

    Ok(())
}

#[derive(BartDisplay, Builder, Debug, Default)]
#[template = "src/bin/parol/tools/templates/main.rs.tpl"]
struct MainRsData<'a> {
    crate_name: &'a str,
    grammar_name: String,
}

fn generate_main_rs(creation_data: &CreationData) -> Result<()> {
    let mut main_file_out = creation_data.path.clone();
    main_file_out.push("src");
    main_file_out.push("main.rs");
    let main_data = MainRsDataBuilder::default()
        .crate_name(creation_data.crate_name)
        .grammar_name(NmHlp::to_upper_camel_case(creation_data.crate_name))
        .build()
        .into_diagnostic()?;
    let main_source = format!("{}", main_data);
    fs::write(main_file_out, main_source)
        .into_diagnostic()
        .wrap_err("Error writing generated user trait source!")?;

    Ok(())
}

#[derive(BartDisplay, Builder, Debug, Default)]
#[template = "src/bin/parol/tools/templates/lib.rs.tpl"]
struct LibRsData<'a> {
    crate_name: &'a str,
    grammar_name: String,
}

fn generate_lib_rs(creation_data: &CreationData) -> Result<()> {
    let mut lib_file_out = creation_data.path.clone();
    lib_file_out.push("src");
    lib_file_out.push("lib.rs");
    let lib_data = LibRsDataBuilder::default()
        .crate_name(creation_data.crate_name)
        .grammar_name(NmHlp::to_upper_camel_case(creation_data.crate_name))
        .build()
        .into_diagnostic()?;
    let lib_source = format!("{}", lib_data);
    fs::write(lib_file_out, lib_source)
        .into_diagnostic()
        .wrap_err("Error writing generated user trait source!")?;

    Ok(())
}

#[derive(BartDisplay, Builder, Debug, Default)]
#[template = "src/bin/parol/tools/templates/grammar.rs.tpl"]
struct GrammarRsData<'a> {
    crate_name: &'a str,
    grammar_name: String,
}

fn generate_grammar_rs(creation_data: &CreationData) -> Result<()> {
    let mut grammar_file_out = creation_data.path.clone();
    grammar_file_out.push("src");
    grammar_file_out.push(format!("{}_grammar.rs", creation_data.crate_name));
    let grammar_data = GrammarRsDataBuilder::default()
        .crate_name(creation_data.crate_name)
        .grammar_name(NmHlp::to_upper_camel_case(creation_data.crate_name))
        .build()
        .into_diagnostic()?;
    let grammar_source = format!("{}", grammar_data);
    fs::write(grammar_file_out, grammar_source)
        .into_diagnostic()
        .wrap_err("Error writing generated user trait source!")?;

    Ok(())
}

