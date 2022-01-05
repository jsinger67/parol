//! Allows programmatically invoking parol from a `build.rs` script
//!
//! The process of invoking a grammar starts with a [`Builder`] and one of two output modes:
//! 1. Cargo build script output mode, via [Builder::with_cargo_script_output] (easiest)
//! 2. Explicitly specifying an output directory via [Builder::with_explicit_output_dir]
//!
//! ## Cargo integration
//! If this API detects it is running inside a [Cargo `build.rs` script](https://doc.rust-lang.org/stable/cargo/reference/build-scripts.html),
//! then it implicitly enables cargo integration.
//!
//! This has Cargo *automatically* regenerate the parser sources whenever the grammar changes. This is done by
//! implicitly outputting the appropriate [`rerun-if-changed=<grammar>`](https://doc.rust-lang.org/stable/cargo/reference/build-scripts.html#change-detection) instructions to Cargo.
//!
//! ### Defaults
//! When using [`Builder::with_cargo_script_output`], a number of reasonable defaults are set:
//!
//! By default, the output directory is set to the `OUT_DIR` environment variable.
//! By default, the generated parser name is `parser.rs` and the generated grammar action file is `
//!
//! You can
//! ```ignore
//! mod parser {
//!     include!(concat!(env!("OUT_DIR"), "/parser.rs"));
//! }
//! ```
//!
//! ### Tradeoffs
//! The disadvantage of using this mode (or using Cargo build scripts in general),
//! is that it adds the `parol` crate as an explicit build dependency.
//!
//! Although this doesn't increase the runtime binary size, it does increase the initial compile times.
//! If someone just wants to `cargo install <your crate>`, Cargo will have to download and execute `parol` to generate your parser code.
//!
//! Contributors to your project (who modify your grammar) will have to download and invoke parol anyways,
//! so this cost primarily affects initial compile times. Also cargo is very intelligent about caching build script outputs,
//! so it really only affects
//!
//! Despite the impact on initial compiles, this is somewhat traditional in the Rust community.
//! It's [the recommended way to use `bindgen`](https://rust-lang.github.io/rust-bindgen/library-usage.html)
//! and it's the only way to use [`pest`](https://pest.rs/).
//!
//! If you are really concerned about compile times,
//! you can use explicit output (below) to avoid invoking pest.
//!
//! ## Explicitly controlling Output Locations
//! If you want more control over the location of generated grammar files,
//! you can invoke [`Builder::with_explicit_output_dir`] to explicitly set an output directory.
//!
//! In addition you must explicitly name your output parser and action files,
//! or the configuration will give an error.
//!
//! This is used to power the command line `parol` tool, and is useful for additional control.
//!
//! Any configured *output* paths (including generated parsers, expanded grammars, etc)
//! are resolved relative to this base output using [Path::join]. This means that specifying absolute paths
//! overrides this explicit base directory.
//!
//! The grammar input file is resolved in the regular manner.
//! It does not use the "output" directory.
//!
//! ### Interaction with version control
//! When using [`Builder::with_cargo_script_output`], the output is put in a subdir of the `target`
//! directory and excluded from version control.
//!
//! This is useful if you want to ignore changes in machine-generated code.
//!
//! However, when specifying an explicit output directory (with [`Builder::with_explicit_output_dir`]),
//!
//! In this case, you would probably set the output to a sub-directory of `src`.
//! This means that files are version controlled
//! and you would have to commit them whenever changes are made.
//!
//! ## Using the CLI directly
//! Note that explicitly specifying the output directory doesn't avoid running parol on `cargo install`.
//!
//! It does not increase the initial build speed, and still requires compiling and invoking `parol`.
//!
//! If you really want to avoid adding `parol` as a build dependency,
//! you need to invoke the CLI manually to generate the parser sources ahead of time.
//!
//! Using a build script requires adding a build dependency, and cargo will unconditionally execute build scripts it on first install.
//! While Cargo's build script caching is excellent, it only activates on recompiles.
//!
//! As such, using the CLI manually is really the only way to improve (initial) compile times.
//!
//! It is (often) not worth it, because it is inconvenient, and the impact only happens on *initial* compiles.
//!
//! ## API Completeness
//! Anything you can do with the main `parol` executable, you should also be able to do with this API.
//!
//! That is because the main executable is just a wrapper around the API
//!
//! However, a couple more advanced features use unstable/internal APIs (see below).
//!
//! As a side note, the CLI does not require you to specify an output location.
//! You can run `parol -f grammar.parol` just fine and it will generate no output.
//!
//! In build scripts, this is typically an mistake (so it errors by default).
//! If you want to disable this sanity check, use [`Builder::disable_output_sanity_checks`]
//!
//! ### Internal APIs
//! The main `parol` command needs a couple of features that do not fit nicely into this API (or interact closely with the crate's internals).
//!
//!
//! Because of that, there are a number of APIs explicitly marked as unstable or internal.
//! Some of these are public and some are private.
//!
//! Expect breaking changes both before and after 1.0 (but especially before).
#![deny(
    missing_docs, // Building should be documented :)
)]

use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::path::{Path, PathBuf};
use std::{env, fs};

use id_tree::Tree;
use miette::{Context, IntoDiagnostic};
use parol_runtime::parser::ParseTreeType;

use crate::analysis::LookaheadDFA;
use crate::{GrammarConfig, ParolGrammar, MAX_K};

/// The default maximum lookahead
///
/// This is used both for the CLI and for the builder.
pub const DEFAULT_MAX_LOOKAHEAD: usize = 5;
/// The default name of the generated grammar module.
pub const DEFAULT_MODULE_NAME: &str = "grammar";
/// The default name of the user type that implements grammar parsing.
pub const DEFAULT_USER_TYPE_NAME: &str = "Grammar";

fn is_build_script() -> bool {
    // Although only `OUT_DIR` is necessary for our purposes, it's possible someone else set it.
    // Check for a second one to make sure we're actually running under cargo
    // See full list of environment variables here: https://is.gd/K6LyzQ
    env::var_os("OUT_DIR").is_some() && env::var_os("CARGO_MANIFEST_DIR").is_some()
}

/// Builds the configuration for generating and analyzing `parol` grammars.
///
/// A grammar file is required for almost all possible operations (set with [Builder::grammar_file])
///
/// Does not actually generate anything until finished.
#[derive(Clone)]
pub struct Builder {
    /// The base output directory
    output_dir: PathBuf,
    grammar_file: Option<PathBuf>,
    /// Output file for expanded grammar
    expanded_grammar_output_file: Option<PathBuf>,
    /// Output file for the generated parser source
    parser_output_file: Option<PathBuf>,
    /// Output file for the generated actions files.
    actions_output_file: Option<PathBuf>,
    user_type_name: String,
    module_name: String,
    cargo_integration: bool,
    max_lookahead: usize,
    /// By default, we want to require that the parser output file is specified.
    /// Otherwise we're just wasting time outputting to /dev/null.
    ///
    /// The CLI needs to be able to override this (mostly for debugging), hence the option.
    output_sanity_checks: bool,
    /// Internal debugging for CLI.
    debug_verbose: bool,
}
impl Builder {
    /// Create a new builder fr use in a Cargo build script (`build.rs`).
    ///
    /// This is the recommended default way to get started.
    ///
    /// All the outputs are set relative to the `OUT_DIR` environment variable,
    /// as is standard for [Cargo build script outputs](https://doc.rust-lang.org/stable/cargo/reference/build-scripts.html#outputs-of-the-build-script).
    ///
    /// This sets sensible defaults for every output file name.
    ///
    /// | Method name                    | CLI Option           | Default (relative) name |
    /// | -------------------------------|----------------------|-------------------------|
    /// | `parser_output_file`           | `--parser` or `-p`   | "parser.rs"             |
    /// | `actions_output_file`          | `--actions` or `-a`  | "grammar.rs"            |
    /// | `expanded_grammar_output_file` | `--expanded` or `-e` | "grammar-exp.par"       |
    ///
    ///
    /// See the module documentation for how to include these files into your project.
    ///
    /// Panics if used outside of a cargo build script.
    pub fn with_cargo_script_output() -> Self {
        assert!(is_build_script(), "Cannot use outside of a cargo script");
        // Don't worry! $OUT_DIR is unique for every
        let out_dir = env::var_os("OUT_DIR").unwrap();
        let mut builder = Self::with_explicit_output_dir(out_dir);
        // Set those reasonable defaults we promised
        builder
            .parser_output_file("parser.rs")
            .actions_output_file("grammar.rs")
            .expanded_grammar_output_file("grammar-exp.par");
        // Cargo integration should already be enabled (because we are a build script)
        assert!(builder.cargo_integration);
        builder
    }
    /// Internal utility to resolve a path relative to the output directory
    fn resolve_output_path(&self, p: impl AsRef<Path>) -> PathBuf {
        self.output_dir.join(p)
    }
    /// Create a new builder with an explicitly specified output directory.
    ///
    /// This requires that output files be specified explicitly,
    /// unless this check is disabled with [`Builder::disable_output_sanity_checks`]
    ///
    /// If this detects running inside a build script,
    /// it will automatically enable cargo integration.
    ///
    /// If output files are specified using absolute paths,
    /// it overrides this explicit output dir.
    ///
    /// See module docs on "explicit output mode" for more details.
    pub fn with_explicit_output_dir(output: impl AsRef<Path>) -> Self {
        /*
         * Most of these correspond to CLI options.
         */
        Builder {
            output_dir: PathBuf::from(output.as_ref()),
            grammar_file: None,
            cargo_integration: is_build_script(),
            debug_verbose: false,
            max_lookahead: DEFAULT_MAX_LOOKAHEAD,
            module_name: String::from(DEFAULT_MODULE_NAME),
            user_type_name: String::from(DEFAULT_USER_TYPE_NAME),
            // In this mode, the user must specify explicit outputs.
            // The default is /dev/null (`None`)
            parser_output_file: None,
            actions_output_file: None,
            expanded_grammar_output_file: None,
            // By default, we require that output files != /dev/null
            output_sanity_checks: true,
        }
    }
    /// By default, we require that the generated parser and action files are not discarded.
    ///
    /// This disables that check (used for the CLI).
    ///
    /// NOTE: When using [`Builder::with_cargo_script_output`], these are automatically inferred.
    pub fn disable_output_sanity_checks(&mut self) -> &mut Self {
        self.output_sanity_checks = false;
        self
    }
    /// Set the output location for the generated parser.
    ///
    /// If you are using [Builder::with_cargo_script_output],
    /// the default output is "$OUT_DIR/parser.rs".
    ///
    /// If you are using an explicitly specified output directory, then this option is *required*.
    pub fn parser_output_file(&mut self, p: impl AsRef<Path>) -> &mut Self {
        self.parser_output_file = Some(self.resolve_output_path(p));
        self
    }
    /// Set the actions output location for the generated parser.
    ///
    /// If you are using [Builder::with_cargo_script_output],
    /// the default output is "$OUT_DIR/grammar.rs".
    ///
    /// If you are using an explicitly specified output directory, then this option is *required*.
    pub fn actions_output_file(&mut self, p: impl AsRef<Path>) -> &mut Self {
        self.actions_output_file = Some(self.resolve_output_path(p));
        self
    }
    /// Set the actions output location for the generated parser.
    ///
    /// If you are using [Builder::with_cargo_script_output],
    /// the default output is "$OUT_DIR/grammar-exp.par".
    ///
    /// Otherwise, this is ignored.
    pub fn expanded_grammar_output_file(&mut self, p: impl AsRef<Path>) -> &mut Self {
        self.expanded_grammar_output_file = Some(self.resolve_output_path(p));
        self
    }
    /// Explicitly enable/disable cargo integration.
    ///
    /// This is automatically set to true if you are running a build script,
    /// and is `false` otherwise.
    pub fn set_cargo_integration(&mut self, enabled: bool) -> &mut Self {
        self.cargo_integration = enabled;
        self
    }
    /// Set the grammar file used as input for parol.
    ///
    /// This is required for most operations.
    ///
    /// Does not check that the file exists.
    pub fn grammar_file(&mut self, grammar: impl AsRef<Path>) -> &mut Self {
        self.grammar_file = Some(PathBuf::from(grammar.as_ref()));
        self
    }
    /// Set the name of the user type that implements the language processing
    pub fn user_type_name(&mut self, name: &str) -> &mut Self {
        self.user_type_name = name.into();
        self
    }
    /// Set the name of the user module that implements the language processing
    ///
    /// This is the module that contains the [Self::user_type_name]
    pub fn user_trait_module_name(&mut self, name: &str) -> &mut Self {
        self.module_name = name.into();
        self
    }
    /// Set the maximum lookahead for the generated parser.
    ///
    /// If nothing is specified, the default lookahead is [DEFAULT_MAX_LOOKAHEAD].
    ///
    /// Returns a [BuilderError] if the lookahead is greater than [crate::MAX_K].
    pub fn max_lookahead(&mut self, k: usize) -> Result<&mut Self, BuilderError> {
        if k > MAX_K {
            return Err(BuilderError::LookaheadTooLarge);
        }
        self.max_lookahead = k;
        Ok(self)
    }
    /// Debug verbose information to the standard output
    ///
    /// This is an internal method, and is only intended for the CLI.
    #[doc(hidden)]
    pub fn debug_verbose(&mut self) -> &mut Self {
        self.debug_verbose = true;
        self
    }

    /// Begin the process of generating the grammar
    /// using the specified listener (or None if no listener is desired).
    ///
    /// Returns an error if the build is *configured* incorrectly.
    /// In a build script, this is typically a programmer error.
    pub fn begin_generation_with<'l>(
        &mut self,
        listener: Option<&'l mut dyn BuildListener>,
    ) -> Result<GrammarGenerator<'l>, BuilderError> {
        /*
         * For those concerned about performance:
         *
         * The overhead of all these copies and dyn dispatch is marginal
         * in comparison to the actual grammar generation.
         */
        let grammar_file = self
            .grammar_file
            .as_ref()
            .ok_or(BuilderError::MissingGrammarFile)?
            .clone();
        if self.output_sanity_checks {
            // Check that we have outputs
            if self.parser_output_file.is_none() {
                return Err(BuilderError::MissingParserOutputFile);
            } else if self.actions_output_file.is_none() {
                return Err(BuilderError::MissingActionOutputFile);
            }
            // Missing expanded grammar file is fine. They might not want that.
        }
        Ok(GrammarGenerator {
            listener: MaybeBuildListener(listener),
            grammar_file,
            builder: self.clone(),
            state: None,
            grammar_config: None,
            lookahead_dfa_s: None,
        })
    }
    /// Generate the parser, writing it to the pre-configured output files.
    pub fn generate_parser(&mut self) -> miette::Result<()> {
        self.begin_generation_with(None)
            .wrap_err("Misconfigured parol generation")?
            .generate_parser()
    }
}

/// Represents in-process grammar generation.
///
/// Most of the time you will want to use [Builder::generate_parser] to bypass this completely.
///
/// This is an advanced API, and unless stated otherwise, all its methods are unstable (see module docs).
pub struct GrammarGenerator<'l> {
    /// The build listener
    ///
    /// This is a fairly advanced feature
    listener: MaybeBuildListener<'l>,
    grammar_file: PathBuf,
    builder: Builder,
    state: Option<State>,
    grammar_config: Option<GrammarConfig>,
    lookahead_dfa_s: Option<BTreeMap<String, LookaheadDFA>>,
}
impl GrammarGenerator<'_> {
    /// Generate the parser, writing it to the pre-configured output files.
    pub fn generate_parser(&mut self) -> miette::Result<()> {
        self.parse()?;
        self.expand()?;
        self.post_process()?;
        self.write_output()?;
        Ok(())
    }

    //
    // Internal APIs
    //

    #[doc(hidden)]
    pub fn parse(&mut self) -> miette::Result<()> {
        assert_eq!(self.state, None);
        let input = fs::read_to_string(&self.grammar_file)
            .into_diagnostic()
            .wrap_err(format!(
                "Can't read grammar file {}",
                self.grammar_file.display()
            ))?;
        if self.builder.cargo_integration {
            println!("cargo:rerun-if-changed=");
        }
        let mut parol_grammar = ParolGrammar::new();
        let syntax_tree = crate::parser::parse(&input, &self.grammar_file, &mut parol_grammar)
            .wrap_err(format!(
                "Failed parsing grammar file {}",
                self.grammar_file.display()
            ))?;
        self.listener
            .on_initial_grammar_parse(&syntax_tree, &parol_grammar)?;
        self.grammar_config = Some(GrammarConfig::try_from(parol_grammar)?);
        self.state = Some(State::Parsed);
        Ok(())
    }
    #[doc(hidden)]
    pub fn expand(&mut self) -> miette::Result<()> {
        assert_eq!(self.state, Some(State::Parsed));
        let grammar_config = self.grammar_config.as_mut().unwrap();
        // NOTE: it's up to the listener to add appropriate error context
        self.listener
            .on_intermediate_grammar(IntermediateGrammar::Untransformed, &*grammar_config)?;
        let cfg = crate::check_and_transform_grammar(&grammar_config.cfg)
            .wrap_err("Basic grammar checks and transformations failed!")?;

        // Exchange original grammar with transformed one
        grammar_config.update_cfg(cfg);

        self.listener
            .on_intermediate_grammar(IntermediateGrammar::Transformed, &*grammar_config)?;
        if let Some(ref expanded_file) = self.builder.expanded_grammar_output_file {
            fs::write(
                expanded_file,
                crate::render_par_string(grammar_config, /* add_index_comment */ true),
            )
            .into_diagnostic()
            .wrap_err("Error writing left-factored grammar!")?;
        }
        self.state = Some(State::Expanded);
        Ok(())
    }
    #[doc(hidden)]
    pub fn post_process(&mut self) -> miette::Result<()> {
        assert_eq!(self.state, Some(State::Expanded));
        let grammar_config = self.grammar_config.as_mut().unwrap();
        self.lookahead_dfa_s = Some(
            crate::calculate_lookahead_dfas(grammar_config, self.builder.max_lookahead)
                .wrap_err("Lookahead calculation for the given grammar failed!")?,
        );

        if self.builder.debug_verbose {
            print!(
                "Lookahead DFAs:\n{:?}",
                self.lookahead_dfa_s.as_ref().unwrap()
            );
        }

        // Update maximum lookahead size for scanner generation
        grammar_config.update_lookahead_size(
            self.lookahead_dfa_s
                .as_ref()
                .unwrap()
                .iter()
                .max_by_key(|(_, dfa)| dfa.k)
                .unwrap()
                .1
                .k,
        );

        if self.builder.debug_verbose {
            print!("\nGrammar config:\n{:?}", grammar_config);
        }
        self.state = Some(State::PostProcessed);
        Ok(())
    }
    #[doc(hidden)]
    pub fn write_output(&mut self) -> miette::Result<()> {
        assert_eq!(self.state, Some(State::PostProcessed));
        let grammar_config = self.grammar_config.as_mut().unwrap();
        let lexer_source = crate::generate_lexer_source(grammar_config)
            .wrap_err("Failed to generate lexer source!")?;

        let parser_source = crate::generate_parser_source(
            grammar_config,
            &lexer_source,
            self.lookahead_dfa_s.as_ref().unwrap(),
        )
        .wrap_err("Failed to generate parser source!")?;

        if let Some(ref parser_file_out) = self.builder.parser_output_file {
            fs::write(parser_file_out, parser_source)
                .into_diagnostic()
                .wrap_err("Error writing generated lexer source!")?;
            crate::try_format(&*parser_file_out);
        } else if self.builder.debug_verbose {
            println!("\nParser source:\n{}", parser_source);
        }

        let user_trait_source = crate::generate_user_trait_source(
            &self.builder.user_type_name,
            &self.builder.module_name,
            grammar_config,
        )
        .wrap_err("Failed to generate user trait source!")?;
        if let Some(ref user_trait_file_out) = self.builder.actions_output_file {
            fs::write(user_trait_file_out, user_trait_source)
                .into_diagnostic()
                .wrap_err("Error writing generated user trait source!")?;
            crate::try_format(user_trait_file_out);
        } else if self.builder.debug_verbose {
            println!("\nSource for semantic actions:\n{}", user_trait_source);
        }
        self.state = Some(State::Finished);
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Parsed,
    Expanded,
    PostProcessed,
    Finished,
}

/// A build listener, for advanced customization of the parser generation.
///
/// This is used by the CLI to implement some of its more advanced options (without cluttering up the main interface).
///
/// The details of this trait are considered unstable.
#[allow(
    unused_variables, // All these variables are going to be unused because these are NOP impls....
    missing_docs, // This is fine because this is internal.
)]
pub trait BuildListener {
    fn on_initial_grammar_parse(
        &mut self,
        syntax_tree: &Tree<ParseTreeType>,
        grammar: &ParolGrammar,
    ) -> miette::Result<()> {
        Ok(())
    }
    fn on_intermediate_grammar(
        &mut self,
        stage: IntermediateGrammar,
        config: &GrammarConfig,
    ) -> miette::Result<()> {
        Ok(())
    }
}
#[derive(Default)]
struct MaybeBuildListener<'l>(Option<&'l mut dyn BuildListener>);
impl<'l> BuildListener for MaybeBuildListener<'l> {
    fn on_initial_grammar_parse(
        &mut self,
        syntax_tree: &Tree<ParseTreeType>,
        grammar: &ParolGrammar,
    ) -> miette::Result<()> {
        if let Some(ref mut inner) = self.0 {
            inner.on_initial_grammar_parse(syntax_tree, grammar)
        } else {
            Ok(())
        }
    }

    fn on_intermediate_grammar(
        &mut self,
        stage: IntermediateGrammar,
        config: &GrammarConfig,
    ) -> miette::Result<()> {
        if let Some(ref mut inner) = self.0 {
            inner.on_intermediate_grammar(stage, config)
        } else {
            Ok(())
        }
    }
}

/// Marks an intermediate stage of the grammar, in between the various transformations that parol does.
///
/// The last transformation is returned by [IntermediateGrammar::LAST]
///
/// This enum gives some degree of access to the individual transformations that parol does.
/// As such, the specific variants are considered unstable.
#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum IntermediateGrammar {
    /// Writes the untransformed parsed grammar
    ///
    /// NOTE: This is different then the initially parsed syntax tree
    Untransformed,
    /// Writes the transformed parsed grammar
    Transformed,
}
impl IntermediateGrammar {
    /// The last transformation.
    pub const LAST: IntermediateGrammar = IntermediateGrammar::Transformed;
}

/// An error that occurs configuring the [Builder].
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[non_exhaustive]
pub enum BuilderError {
    /// Indicates that the operation needs a grammar file as input,
    /// but that one has not been specified.
    #[error("Missing an input grammar file")]
    MissingGrammarFile,
    /// Indicates that no parser output file has been specified.
    ///
    /// This would discard the generated parser, which is typically a mistake.
    #[error("No parser output file specified")]
    MissingParserOutputFile,
    /// Indicates that no parser output file has been specified.
    ///
    /// This would discard the generated parser, which is typically a mistake.
    #[error("No action output file specified")]
    MissingActionOutputFile,
    /// Indicates that the specified lookahead is too large
    #[error("Maximum lookahead is {}", MAX_K)]
    LookaheadTooLarge,
}
