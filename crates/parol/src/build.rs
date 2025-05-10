//! Allows programmatically invoking parol from a `build.rs` script
//!
//! The process of invoking a grammar starts with a [`struct@Builder`] and one of two output modes:
//! 1. Cargo build script output mode, via [Builder::with_cargo_script_output] (easiest)
//! 2. Explicitly specifying an output directory via [Builder::with_explicit_output_dir]
//!
//! ## Cargo integration
//! If this API detects it is running inside a
//! [Cargo `build.rs` script](https://doc.rust-lang.org/stable/cargo/reference/build-scripts.html),
//! then it implicitly enables cargo integration.
//!
//! This has Cargo *automatically* regenerate the parser sources whenever the grammar changes. This
//! is done by implicitly outputting the appropriate
//! [`rerun-if-changed=<grammar>`](https://doc.rust-lang.org/stable/cargo/reference/build-scripts.html#change-detection)
//! instructions to Cargo.
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
//! Although this doesn't increase the runtime binary size, it does increase the initial compile
//! times.
//! If someone just wants to `cargo install <your crate>`, Cargo will have to download and execute
//! `parol` to generate your parser code.
//!
//! Contributors to your project (who modify your grammar) will have to download and invoke parol
//! anyways, so this cost primarily affects initial compile times. Also cargo is very intelligent
//! about caching build script outputs.
//!
//! Despite the impact on initial compiles, this is somewhat traditional in the Rust community.
//! It's [the recommended way to use `bindgen`](https://rust-lang.github.io/rust-bindgen/library-usage.html)
//! and it's the only way to use [`pest`](https://pest.rs/).
//!
//! If you are really concerned about compile times, you can use explicit output (below).
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
//! are resolved relative to this base output using [Path::join]. This means that specifying
//! absolute paths overrides this explicit base directory.
//!
//! The grammar input file is resolved in the regular manner.
//! It does not use the "output" directory.
//!
//! ### Interaction with version control
//! When using [`Builder::with_cargo_script_output`], the output is put in a subdir of the `target`
//! directory and excluded from version control.
//!
//! This is useful if you want to ignore changes in generated code.
//!
//! However, when specifying an explicit output directory (with [`Builder::with_explicit_output_dir`]),
//! you may have to include the generated sources explicitly into the build process. One way is
//! indicated above where the include! macro is used.
//!
//! Otherwise, you would probably set the output to a sub-directory of `src`.
//! This means that files are version controlled and you would have to commit them whenever changes
//! are made.
//!
//! ## Using the CLI directly
//! Note that explicitly specifying the output directory doesn't avoid running parol on `cargo
//! install`.
//!
//! It does not increase the initial build speed, and still requires compiling and invoking `parol`.
//!
//! If you really want to avoid adding `parol` as a build dependency,
//! you need to invoke the CLI manually to generate the parser sources ahead of time.
//!
//! Using a build script requires adding a build dependency, and cargo will unconditionally execute
//! build scripts on first install.
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
//! In build scripts, this is typically a mistake (so it errors by default).
//! If you want to disable this sanity check, use [`Builder::disable_output_sanity_checks`]
//!
//! ### Internal APIs
//! The main `parol` command needs a couple of features that do not fit nicely into this API
//! (or interact closely with the crate's internals).
//!
//!
//! Because of that, there are a number of APIs explicitly marked as unstable or internal.
//! Some of these are public and some are private.
//!
//! Expect breaking changes both before and after 1.0 (but especially before).
#![deny(missing_docs)]

use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::path::{Path, PathBuf};
use std::{env, fs};

use crate::config::{CommonGeneratorConfig, ParserGeneratorConfig, UserTraitGeneratorConfig};
use crate::generators::export_node_types::{NodeTypesExporter, NodeTypesInfo};
use crate::generators::node_kind_enum_generator::NodeKindTypesGenerator;
use crate::parser::GrammarType;
use crate::{
    GrammarConfig, GrammarTypeInfo, LRParseTable, LookaheadDFA, MAX_K, ParolGrammar,
    UserTraitGenerator,
};
use clap::{Parser, ValueEnum};
use parol_macros::parol;
use parol_runtime::{ParseTree, Result};

/// Contains all attributes that should be inserted optionally on top of the generated trait source.
/// * Used in the Builder API. Therefore it mus be public
#[derive(Clone, Debug, Parser, ValueEnum)]
pub enum InnerAttributes {
    /// Suppresses clippy warnings like these: `warning: this function has too many arguments (9/7)`
    AllowTooManyArguments,
}

impl std::fmt::Display for InnerAttributes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InnerAttributes::AllowTooManyArguments => {
                write!(f, "#![allow(clippy::too_many_arguments)]")
            }
        }
    }
}

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
    /// The output file for the generated syntree node wrappers
    node_kind_enum_output_file: Option<PathBuf>,
    pub(crate) user_type_name: String,
    pub(crate) module_name: String,
    cargo_integration: bool,
    max_lookahead: usize,
    /// By default, we want to require that the parser output file is specified.
    /// Otherwise we're just wasting time outputting to /dev/null.
    ///
    /// The CLI needs to be able to override this (mostly for debugging), hence the option.
    output_sanity_checks: bool,
    /// Activate the minimization of boxed types in the generated parser
    pub(crate) minimize_boxed_types: bool,
    /// Internal debugging for CLI.
    debug_verbose: bool,
    /// Generate range information for AST types
    range: bool,
    /// Generate typed syntree node wrappers
    enum_kind: bool,
    /// Inner attributes to insert at the top of the generated trait source.
    inner_attributes: Vec<InnerAttributes>,
    /// Enables trimming of the parse tree during parsing.
    /// Generates the call to trim_parse_tree on the parser object before the call of parse.
    pub(crate) trim_parse_tree: bool,
    /// Disbales the error recovery mechanism in the generated parser
    pub(crate) disable_recovery: bool,
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
    /// | `actions_output_file`          | `--actions` or `-a`  | "grammar_trait.rs"      |
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
            .actions_output_file("grammar_trait.rs")
            .node_kind_enum_output_file("node_kind.rs")
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
            range: false,
            enum_kind: false,
            max_lookahead: DEFAULT_MAX_LOOKAHEAD,
            module_name: String::from(DEFAULT_MODULE_NAME),
            user_type_name: String::from(DEFAULT_USER_TYPE_NAME),
            // In this mode, the user must specify explicit outputs.
            // The default is /dev/null (`None`)
            parser_output_file: None,
            actions_output_file: None,
            node_kind_enum_output_file: None,
            expanded_grammar_output_file: None,
            minimize_boxed_types: false,
            inner_attributes: Vec::new(),
            // By default, we require that output files != /dev/null
            output_sanity_checks: true,
            trim_parse_tree: false,
            disable_recovery: false,
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
    /// the default output is "$OUT_DIR/grammar_trait.rs".
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
    /// Set the output location for the generated node kind enum.
    /// The output does not contain any `parol_runtime` dependencies, so you can specify "../other_crate/src/node_kind.rs" as the output file while the other crate does not have `parol_runtime` as a dependency.
    ///
    /// The default location is "$OUT_DIR/node_kind.rs".
    pub fn node_kind_enum_output_file(&mut self, p: impl AsRef<Path>) -> &mut Self {
        self.node_kind_enum_output_file = Some(self.resolve_output_path(p));
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
    pub fn max_lookahead(&mut self, k: usize) -> std::result::Result<&mut Self, BuilderError> {
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
    /// Generate range information for AST types
    ///
    pub fn range(&mut self) -> &mut Self {
        self.range = true;
        self
    }
    /// Generate node kind enums `TerminalKind` and `NonTerminalKind`
    pub fn node_kind_enums(&mut self) -> &mut Self {
        self.enum_kind = true;
        self
    }
    /// Inserts the given inner attributes at the top of the generated trait source.
    pub fn inner_attributes(&mut self, inner_attributes: Vec<InnerAttributes>) -> &mut Self {
        self.inner_attributes = inner_attributes;
        self
    }
    /// Activate the minimization of boxed types in the generated parser
    pub fn minimize_boxed_types(&mut self) -> &mut Self {
        self.minimize_boxed_types = true;
        self
    }
    /// Enables trimming of the parse tree during parsing.
    /// Generates the call to trim_parse_tree on the parser object before the call of parse.
    ///
    pub fn trim_parse_tree(&mut self) -> &mut Self {
        self.trim_parse_tree = true;
        self
    }

    /// Disables the error recovery mechanism in the generated parser
    pub fn disable_recovery(&mut self) -> &mut Self {
        self.disable_recovery = true;
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
    ) -> std::result::Result<GrammarGenerator<'l>, BuilderError> {
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
            parse_table: None,
            type_info: None,
        })
    }
    /// Generate the parser, writing it to the pre-configured output files.
    pub fn generate_parser(&mut self) -> Result<()> {
        self.begin_generation_with(None)
            .map_err(|e| parol!("Misconfigured parol generation: {}", e))?
            .generate_parser()
    }
    /// Generate the parser, writing it to the pre-configured output files. And export the node info.
    pub fn generate_parser_and_export_node_infos(&mut self) -> Result<NodeTypesInfo> {
        self.begin_generation_with(None)
            .map_err(|e| parol!("Misconfigured parol generation: {}", e))?
            .generate_parser_and_export_node_infos()
    }
}

impl CommonGeneratorConfig for Builder {
    fn user_type_name(&self) -> &str {
        &self.user_type_name
    }

    fn module_name(&self) -> &str {
        &self.module_name
    }

    fn minimize_boxed_types(&self) -> bool {
        self.minimize_boxed_types
    }

    fn range(&self) -> bool {
        self.range
    }

    fn node_kind_enums(&self) -> bool {
        self.enum_kind
    }
}

impl ParserGeneratorConfig for Builder {
    fn trim_parse_tree(&self) -> bool {
        self.trim_parse_tree
    }

    fn recovery_disabled(&self) -> bool {
        self.disable_recovery
    }
}

impl UserTraitGeneratorConfig for Builder {
    fn inner_attributes(&self) -> &[InnerAttributes] {
        &self.inner_attributes
    }
}

/// Represents in-process grammar generation.
///
/// Most of the time you will want to use [Builder::generate_parser] to bypass this completely.
///
/// This is an advanced API, and unless stated otherwise, all its methods are unstable (see module docs).
///
/// The lifetime parameter `'l` refers to the lifetime of the optional listener.
pub struct GrammarGenerator<'l> {
    /// The build listener
    ///
    /// This is a fairly advanced feature
    listener: MaybeBuildListener<'l>,
    pub(crate) grammar_file: PathBuf,
    builder: Builder,
    state: Option<State>,
    pub(crate) grammar_config: Option<GrammarConfig>,
    lookahead_dfa_s: Option<BTreeMap<String, LookaheadDFA>>,
    parse_table: Option<LRParseTable>,
    type_info: Option<GrammarTypeInfo>,
}
impl GrammarGenerator<'_> {
    /// Generate the parser, writing it to the pre-configured output files.
    pub fn generate_parser(&mut self) -> Result<()> {
        self.parse()?;
        self.expand()?;
        self.post_process()?;
        self.write_output()?;
        Ok(())
    }

    /// Generate the parser, writing it to the pre-configured output files. And export the node info.
    pub fn generate_parser_and_export_node_infos(&mut self) -> Result<NodeTypesInfo> {
        self.parse()?;
        self.expand()?;
        self.post_process()?;
        self.write_output()?;
        self.export_node_infos()
    }

    //
    // Internal APIs
    //

    #[doc(hidden)]
    pub fn parse(&mut self) -> Result<()> {
        assert_eq!(self.state, None);
        let input = fs::read_to_string(&self.grammar_file).map_err(|e| {
            parol!(
                "Can't read grammar file {}: {}",
                self.grammar_file.display(),
                e
            )
        })?;
        if self.builder.cargo_integration {
            println!("cargo:rerun-if-changed={}", self.grammar_file.display());
        }
        let mut parol_grammar = ParolGrammar::new();
        let syntax_tree = crate::parser::parse(&input, &self.grammar_file, &mut parol_grammar)?;
        self.listener
            .on_initial_grammar_parse(&syntax_tree, &input, &parol_grammar)?;
        self.grammar_config = Some(GrammarConfig::try_from(parol_grammar)?);
        self.state = Some(State::Parsed);
        Ok(())
    }
    #[doc(hidden)]
    pub fn expand(&mut self) -> Result<()> {
        assert_eq!(self.state, Some(State::Parsed));
        let grammar_config = self.grammar_config.as_mut().unwrap();
        // NOTE: it's up to the listener to add appropriate error context
        self.listener
            .on_intermediate_grammar(IntermediateGrammar::Untransformed, &*grammar_config)?;
        let cfg =
            crate::check_and_transform_grammar(&grammar_config.cfg, grammar_config.grammar_type)?;

        // To have at least a preliminary version of the expanded grammar,
        // even when the next checks fail, we write out the expanded grammar here.
        // In most cases it will be overwritten further on.
        if let Some(ref expanded_file) = self.builder.expanded_grammar_output_file {
            fs::write(
                expanded_file,
                crate::render_par_string(grammar_config, /* add_index_comment */ true)?,
            )
            .map_err(|e| parol!("Error writing left-factored grammar! {}", e))?;
        }

        // Exchange original grammar with transformed one
        grammar_config.update_cfg(cfg);

        self.listener
            .on_intermediate_grammar(IntermediateGrammar::Transformed, &*grammar_config)?;
        if let Some(ref expanded_file) = self.builder.expanded_grammar_output_file {
            fs::write(
                expanded_file,
                crate::render_par_string(grammar_config, /* add_index_comment */ true)?,
            )
            .map_err(|e| parol!("Error writing left-factored grammar!: {}", e))?;
        }
        self.state = Some(State::Expanded);
        Ok(())
    }
    #[doc(hidden)]
    pub fn post_process(&mut self) -> Result<()> {
        assert_eq!(self.state, Some(State::Expanded));
        let grammar_config = self.grammar_config.as_mut().unwrap();
        match grammar_config.grammar_type {
            GrammarType::LLK => {
                self.lookahead_dfa_s = Some(
                    crate::calculate_lookahead_dfas(grammar_config, self.builder.max_lookahead)
                        .map_err(|e| {
                            parol!("Lookahead calculation for the given grammar failed!: {}", e)
                        })?,
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
            }
            GrammarType::LALR1 => {
                self.parse_table = Some(crate::calculate_lalr1_parse_table(grammar_config)?.0);
                grammar_config.update_lookahead_size(1);
            }
        }

        if self.builder.debug_verbose {
            print!("\nGrammar config:\n{:?}", grammar_config);
        }
        self.state = Some(State::PostProcessed);
        Ok(())
    }
    #[doc(hidden)]
    pub fn write_output(&mut self) -> Result<()> {
        assert_eq!(self.state, Some(State::PostProcessed));
        let grammar_config = self.grammar_config.as_mut().unwrap();
        let lexer_source = crate::generate_lexer_source(grammar_config)
            .map_err(|e| parol!("Failed to generate lexer source!: {}", e))?;

        let user_trait_generator = UserTraitGenerator::new(grammar_config);
        let mut type_info: GrammarTypeInfo =
            GrammarTypeInfo::try_new(&self.builder.user_type_name)?;
        let user_trait_source = user_trait_generator.generate_user_trait_source(
            &self.builder,
            grammar_config.grammar_type,
            &mut type_info,
        )?;
        if let Some(ref user_trait_file_out) = self.builder.actions_output_file {
            fs::write(user_trait_file_out, user_trait_source)
                .map_err(|e| parol!("Error writing generated user trait source!: {}", e))?;
            crate::try_format(user_trait_file_out)?;
        } else if self.builder.debug_verbose {
            println!("\nSource for semantic actions:\n{}", user_trait_source);
        }

        let ast_type_has_lifetime = type_info.symbol_table.has_lifetime(type_info.ast_enum_type);

        let parser_source = match grammar_config.grammar_type {
            GrammarType::LLK => crate::generate_parser_source(
                grammar_config,
                &lexer_source,
                &self.builder,
                self.lookahead_dfa_s.as_ref().unwrap(),
                ast_type_has_lifetime,
            )?,
            GrammarType::LALR1 => crate::generate_lalr1_parser_source(
                grammar_config,
                &lexer_source,
                &self.builder,
                self.parse_table.as_ref().unwrap(),
                ast_type_has_lifetime,
            )?,
        };

        if let Some(ref parser_file_out) = self.builder.parser_output_file {
            fs::write(parser_file_out, parser_source)
                .map_err(|e| parol!("Error writing generated lexer source!: {}", e))?;
            crate::try_format(parser_file_out)?;
        } else if self.builder.debug_verbose {
            println!("\nParser source:\n{}", parser_source);
        }

        if let Some(ref syntree_node_wrappers_output_file) = self.builder.node_kind_enum_output_file
        {
            let mut f = fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(syntree_node_wrappers_output_file)
                .map_err(|e| parol!("Error opening generated syntree node wrappers!: {}", e))?;
            let syntree_node_types_generator =
                NodeKindTypesGenerator::new(grammar_config, &type_info);
            syntree_node_types_generator
                .generate(&mut f)
                .map_err(|e| parol!("Error generating syntree node wrappers!: {}", e))?;
            crate::try_format(syntree_node_wrappers_output_file)?;
        }

        self.state = Some(State::Finished);
        self.type_info = Some(type_info);

        Ok(())
    }

    fn export_node_infos(&self) -> Result<NodeTypesInfo> {
        let node_types_exporter = NodeTypesExporter::new(
            self.grammar_config.as_ref().unwrap(),
            self.type_info.as_ref().unwrap(),
        );
        Ok(node_types_exporter.generate())
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
        syntax_tree: &ParseTree,
        input: &str,
        grammar: &ParolGrammar,
    ) -> Result<()> {
        Ok(())
    }
    fn on_intermediate_grammar(
        &mut self,
        stage: IntermediateGrammar,
        config: &GrammarConfig,
    ) -> Result<()> {
        Ok(())
    }
}
#[derive(Default)]
struct MaybeBuildListener<'l>(Option<&'l mut dyn BuildListener>);
impl BuildListener for MaybeBuildListener<'_> {
    fn on_initial_grammar_parse(
        &mut self,
        syntax_tree: &ParseTree,
        input: &str,
        grammar: &ParolGrammar,
    ) -> Result<()> {
        if let Some(ref mut inner) = self.0 {
            inner.on_initial_grammar_parse(syntax_tree, input, grammar)
        } else {
            Ok(())
        }
    }

    fn on_intermediate_grammar(
        &mut self,
        stage: IntermediateGrammar,
        config: &GrammarConfig,
    ) -> Result<()> {
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

/// An error that occurs configuring the [struct@Builder].
#[derive(Debug, thiserror::Error)]
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
