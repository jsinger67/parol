mod build_rs;
mod grammar_rs;
mod lib_rs;
mod main_rs;

use build_rs::BuildRsDataBuilder;
use grammar_rs::GrammarRsDataBuilder;
use lib_rs::LibRsDataBuilder;
use main_rs::MainRsDataBuilder;

use anyhow::{Context, Result, anyhow};
use clap::ArgGroup;
use derive_builder::Builder;
use owo_colors::OwoColorize;
use parol::generators::NamingHelper as NmHlp;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

/// Creates a new crate that uses `parol`.
#[derive(clap::Parser)]
#[clap(name = "new")]
#[clap(group(ArgGroup::new("lib_or_bin").args(&["lib", "bin"]).multiple(false).required(true)))]
pub struct Args {
    /// The directory where to create the new crate
    #[clap(short, long)]
    path: PathBuf,

    /// The new crate should be a binary executable
    #[clap(short, long)]
    bin: bool,

    /// The new crate should be a library
    #[clap(short, long)]
    lib: bool,

    /// The name of the new crate, defaults to the directory name
    #[clap(short, long)]
    name: Option<String>,

    /// Add support for generating visualized parse trees
    #[clap(short, long)]
    tree: bool,

    /// Track the generated files in git
    #[clap(long)]
    track_generated_files: bool,
}

#[derive(Debug, Builder)]
struct CreationData<'a> {
    crate_name: &'a str,
    grammar_name: String,
    path: PathBuf,
    is_bin: bool,
    tree_gen: bool,
    track_generated_files: bool,
}

pub fn main(args: &Args) -> Result<()> {
    let crate_name =
        NmHlp::to_lower_snake_case(&NmHlp::purge_name(if let Some(name) = args.name.as_ref() {
            name
        } else {
            args.path
                .as_path()
                .file_name()
                .ok_or_else(|| anyhow!("Trouble to handle path"))?
                .to_str()
                .ok_or_else(|| anyhow!("Trouble to handle path"))?
        }));

    let creation_data = CreationDataBuilder::default()
        .crate_name(&crate_name)
        .grammar_name(NmHlp::to_upper_camel_case(&crate_name))
        .path(args.path.clone())
        .is_bin(args.bin)
        .tree_gen(args.tree)
        .track_generated_files(args.track_generated_files)
        .build()?;

    apply_cargo(&creation_data)?;

    print!(
        "Generating crate {} for grammar {}...",
        creation_data.crate_name.green(),
        creation_data.grammar_name.green()
    );

    generate_crate(&creation_data)?;

    Ok(())
}

const DEPENDENCIES: &[&[&str]] = &[
    &["add", "env_logger@0.11"],
    &["add", "parol_runtime@4.0"],
    &["add", "thiserror@2.0"],
    &["add", "anyhow@1.0"],
    &["add", "scnr2@0.3.2"],
    &[
        "add",
        concat!("parol@", env!("CARGO_PKG_VERSION")),
        "--build",
    ],
];

const TREE_GEN_DEPENDENCY: &str = "add syntree_layout@0.4.0";

fn apply_cargo(creation_data: &CreationData) -> Result<()> {
    // Prepare arguments for the `cargo new` command
    let mut cargo_args = vec!["new"];
    if creation_data.is_bin {
        cargo_args.push("--bin");
    } else {
        cargo_args.push("--lib");
    }
    cargo_args.push("--name");
    cargo_args.push(creation_data.crate_name);
    cargo_args.push(
        creation_data
            .path
            .to_str()
            .ok_or_else(|| anyhow!("Please provide a path"))?,
    );

    // Call the `cargo new` command
    Command::new("cargo")
        .args(&cargo_args)
        .status()
        .map(|_| ())?;

    // Add dependencies
    DEPENDENCIES.iter().try_for_each(|cargo_args| {
        if !cargo_args[1].contains('-') {
            Command::new("cargo")
                .current_dir(&creation_data.path)
                .args(*cargo_args)
                .status()
                .map(|_| ())
                .context("Maybe you have to install cargo-edit: `cargo install cargo-edit`?")
        } else {
            let cargo_args = "add parol --build --git https://github.com/jsinger67/parol.git";
            Command::new("cargo")
                .current_dir(&creation_data.path)
                .args(cargo_args.split(' '))
                .status()
                .map(|_| ())
                .context("Maybe you have to install cargo-edit: `cargo install cargo-edit`?")
        }
    })?;

    // Add dependency to syntree_layout
    if creation_data.tree_gen {
        Command::new("cargo")
            .current_dir(&creation_data.path)
            .args(TREE_GEN_DEPENDENCY.split(' '))
            .status()
            .map(|_| ())?
    }

    Ok(())
}

fn generate_crate(creation_data: &CreationData) -> Result<()> {
    generate_build_rs(creation_data)?;
    generate_grammar_par(creation_data)?;
    touch_modules(creation_data)?;
    generate_grammar_rs(creation_data)?;
    if creation_data.is_bin {
        generate_main_rs(creation_data)?;
    } else {
        generate_lib_rs(creation_data)?;
    }
    generate_test_txt(creation_data)?;
    // Generate the .gitignore file
    if !creation_data.track_generated_files {
        generate_gitignore(creation_data)?;
    }

    Ok(())
}

fn generate_build_rs(creation_data: &CreationData) -> Result<()> {
    let mut build_file_out = creation_data.path.clone();
    build_file_out.push("build.rs");
    let build_data = BuildRsDataBuilder::default()
        .crate_name(creation_data.crate_name)
        .grammar_name(NmHlp::to_upper_camel_case(creation_data.crate_name))
        .tree_gen(creation_data.tree_gen)
        .build()?;
    let build_source = format!("{build_data}");
    fs::write(build_file_out, build_source)
        .context("Error writing generated user trait source!")?;

    Ok(())
}

fn generate_grammar_par(creation_data: &CreationData) -> Result<()> {
    let mut grammar_file_out = creation_data.path.clone();
    grammar_file_out.push(format!("{}.par", creation_data.crate_name));
    let grammar_name = NmHlp::to_upper_camel_case(creation_data.crate_name);
    let grammar_source = format!(
        r#"%start {grammar_name}
%title "{grammar_name} grammar"
%comment "Initial grammar generated by `parol`"
%line_comment "//"

%%

// Start symbol
{grammar_name}: "Hello world!";
"#
    );
    fs::write(grammar_file_out, grammar_source)
        .context("Error writing generated user trait source!")?;

    Ok(())
}

fn touch_modules(creation_data: &CreationData) -> Result<()> {
    let parser_file_out = creation_data
        .path
        .join("src")
        .join(format!("{}_parser.rs", creation_data.crate_name));
    let grammar_trait_file_out = creation_data
        .path
        .join("src")
        .join(format!("{}_grammar_trait.rs", creation_data.crate_name));
    fs::write(
        parser_file_out,
        "// This file will be generated on the first build",
    )
    .context("Error writing generated user trait source!")?;
    fs::write(
        grammar_trait_file_out,
        "// This file will be generated on the first build",
    )
    .context("Error writing generated user trait source!")?;

    Ok(())
}

fn generate_main_rs(creation_data: &CreationData) -> Result<()> {
    let mut main_file_out = creation_data.path.clone();
    main_file_out.push("src");
    main_file_out.push("main.rs");
    let main_data = MainRsDataBuilder::default()
        .crate_name(creation_data.crate_name)
        .grammar_name(NmHlp::to_upper_camel_case(creation_data.crate_name))
        .tree_gen(creation_data.tree_gen)
        .build()?;
    let main_source = format!("{main_data}");
    fs::write(&main_file_out, main_source).context("Error writing generated user trait source!")?;
    fmt(&main_file_out)?;

    Ok(())
}

fn generate_lib_rs(creation_data: &CreationData) -> Result<()> {
    let mut lib_file_out = creation_data.path.clone();
    lib_file_out.push("src");
    lib_file_out.push("lib.rs");
    let lib_data = LibRsDataBuilder::default()
        .crate_name(creation_data.crate_name)
        .grammar_name(NmHlp::to_upper_camel_case(creation_data.crate_name))
        .tree_gen(creation_data.tree_gen)
        .build()?;
    let lib_source = format!("{lib_data}");
    fs::write(&lib_file_out, lib_source).context("Error writing generated user trait source!")?;
    fmt(&lib_file_out)?;

    Ok(())
}

fn generate_grammar_rs(creation_data: &CreationData) -> Result<()> {
    let mut grammar_file_out = creation_data.path.clone();
    grammar_file_out.push("src");
    grammar_file_out.push(format!("{}_grammar.rs", creation_data.crate_name));
    let grammar_data = GrammarRsDataBuilder::default()
        .crate_name(creation_data.crate_name)
        .grammar_name(NmHlp::to_upper_camel_case(creation_data.crate_name))
        .build()?;
    let grammar_source = format!("{grammar_data}");
    fs::write(&grammar_file_out, grammar_source)
        .context("Error writing generated user trait source!")?;
    fmt(&grammar_file_out)?;

    Ok(())
}

fn generate_test_txt(creation_data: &CreationData) -> Result<()> {
    let mut test_file = creation_data.path.clone();
    test_file.push("test.txt");
    let test_content = "\
// To run the test please issue:
// cargo run ./test.txt

    Hello world!

// End
"
    .to_string();
    fs::write(test_file, test_content).context("Error writing test file!")?;

    Ok(())
}

fn generate_gitignore(creation_data: &CreationData) -> Result<()> {
    let path = creation_data.path.clone().join(".gitignore");
    let crate_name = creation_data.crate_name;
    let mut file = fs::OpenOptions::new()
        // Cargo skips to generate .gitignore inside a existing repository.
        .create(true)
        .append(true)
        .open(path)
        .context("Error opening .gitignore file!")?;

    write!(
        file,
        "\
        # Generated by parol\n\
        {crate_name}-exp.par\n\
        src/{crate_name}_parser.rs\n\
        src/{crate_name}_grammar_trait.rs\n"
    )
    .context("Error writing to .gitignore file!")?;

    Ok(())
}

fn fmt<T: AsRef<std::path::Path>>(path: T) -> Result<()> {
    Command::new("rustfmt")
        .arg(path.as_ref().to_str().unwrap())
        .status()
        .map(|_| ())
        .context("Error running rustfmt")
}
