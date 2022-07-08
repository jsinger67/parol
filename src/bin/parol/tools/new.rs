use clap::ArgGroup;
use derive_builder::Builder;
use miette::{bail, miette, Context, IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use parol::generators::NamingHelper as NmHlp;
use semver::{BuildMetadata, Prerelease, Version};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Creates a new crate that uses `parol`.
#[derive(clap::Parser)]
#[clap(name = "new")]
#[clap(group(ArgGroup::new("lib_or_bin").args(&["lib", "bin"]).multiple(false).required(true)))]
pub struct Args {
    /// The directory where to create the new crate
    #[clap(short, long, parse(from_os_str))]
    path: PathBuf,

    /// The new crate should be a binary executable
    #[clap(short, long)]
    bin: bool,

    /// The new crate should be a library
    #[clap(short, long)]
    lib: bool,

    /// The name of the new crate. Defaults to the directory name.
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
    let crate_name =
        NmHlp::to_lower_snake_case(&NmHlp::purge_name(if let Some(name) = args.name.as_ref() {
            name
        } else {
            args.path
                .as_path()
                .file_name()
                .ok_or_else(|| miette!("Trouble to handle path"))?
                .to_str()
                .ok_or_else(|| miette!("Trouble to handle path"))?
        }));

    let creation_data = CreationDataBuilder::default()
        .crate_name(&crate_name)
        .grammar_name(NmHlp::to_upper_camel_case(&crate_name))
        .path(args.path.clone())
        .is_bin(args.bin)
        .build()
        .into_diagnostic()?;

    apply_cargo(&creation_data)?;

    print!(
        "Generating crate {} for grammar {}...",
        creation_data.crate_name.green(),
        creation_data.grammar_name.green()
    );

    generate_crate(creation_data)?;

    Ok(())
}

const DEPENDENCIES: &[&[&str]] = &[
    &["add", "derive_builder@0.11.2"],
    &["add", "env_logger@0.9.0"],
    &["add", "function_name@0.3.0"],
    &["add", "id_tree@^1.8"],
    &["add", "lazy_static@^1.4"],
    &["add", "log@0.4.17"],
    &["add", "miette@^5.1", "--features", "fancy"],
    &["add", "parol_runtime@0.7.0"],
    &["add", "thiserror@^1.0"],
    &[
        "add",
        concat!("parol@", env!("CARGO_PKG_VERSION")),
        "--build",
    ],
];

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
            .ok_or_else(|| miette!("Please provide a path"))?,
    );

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
            if !cargo_args[1].contains("-") {
                Command::new("cargo")
                    .current_dir(&creation_data.path)
                    .args(*cargo_args)
                    .status()
                    .map(|_| ())
                    .into_diagnostic()
                    .wrap_err("Maybe you have to install cargo-edit: `cargo install cargo-edit`?")
            } else {
                let mut prev_version =
                    Version::parse(env!("CARGO_PKG_VERSION")).into_diagnostic()?;
                prev_version.pre = Prerelease::EMPTY;
                prev_version.build = BuildMetadata::EMPTY;
                if prev_version.patch > 0 {
                    prev_version.patch -= 1;
                } else {
                    bail!(
                        r"Can't handle a prerelease version of parol with patch version 0!
Please, install the latest released version of parol (`cargo install parol`)."
                    )
                }
                let cargo_args = format!("add parol@{} --build", prev_version);
                Command::new("cargo")
                    .current_dir(&creation_data.path)
                    .args(cargo_args.split(' '))
                    .status()
                    .map(|_| ())
                    .into_diagnostic()
            }
        })?;
    Ok(())
}

fn generate_crate(creation_data: CreationData) -> Result<()> {
    generate_build_rs(&creation_data)?;
    generate_grammar_par(&creation_data)?;
    if creation_data.is_bin {
        generate_main_rs(&creation_data)?;
    } else {
        generate_lib_rs(&creation_data)?;
    }
    generate_grammar_rs(&creation_data)?;
    generate_test_txt(&creation_data)?;

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

#[derive(BartDisplay, Builder, Debug, Default)]
#[template = "src/bin/parol/tools/templates/test.txt"]
struct TestTxtData<'a> {
    crate_name: &'a str,
}

fn generate_test_txt(creation_data: &CreationData) -> Result<()> {
    let mut test_file = creation_data.path.clone();
    test_file.push("test.txt");
    let grammar_data = TestTxtDataBuilder::default()
        .crate_name(creation_data.crate_name)
        .build()
        .into_diagnostic()?;
    let test_content = format!("{}", grammar_data);
    fs::write(test_file, test_content)
        .into_diagnostic()
        .wrap_err("Error writing test file!")?;

    Ok(())
}
