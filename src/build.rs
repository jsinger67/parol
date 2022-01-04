//! Allows programatically invoking parol from a `build.rs` script
//!
//! The process of invoking a grammar starts with a [`Builder`] and one of two modes:
//! 1. Cargo build script output mode, via [Builder::for_cargo_script] (recommended)
//! 2. Explicit output mode, specifying a root directory [Builder::with_output_dir]
//!
//! ## Cargo Build Script Mode
//! This mode is intended to be used from within a [Cargo `build.rs` script](https://doc.rust-lang.org/stable/cargo/reference/build-scripts.html)
//!
//! It has Cargo *automatically regenerates the parser sources* whenever the grammar changes. This is done by
//! implicitly outputing the appropriate [`rerun-if-changed=<grammar>`](https://doc.rust-lang.org/stable/cargo/reference/build-scripts.html#change-detection) instructions to Cargo.
//!
//! By default, the output directory is set to the `OUT_DIR` environment variable.
//! Assuming your grammar file has a relative path "grammar.rs", you can use the following code to include
//! the generated parser file:
//!
//! ```ignore
//! mod grammar {
//!     include!(concat!(env!("OUT_DIR"), "/grammar.rs"));
//! }
//! ```
//!
//! ### Disadvantages
//! The disadvantage of using this mode is that it adds the `parol` crate is an explicit build dependency.
//!
//! Although this doesn't increase the runtime binary size, it does increase the initial compile times.
//! If someone just wants to `cargo install <your crate>`, Cargo will have to download and execute `parol` to generate your parser code.
//!
//! Contributors to your project (who modify your grammar) will have to download and invoke pest anyways,
//! so this cost primarily affects initial compile times. (Also cargo is very intellegent about caching build script outputs).
//!
//! Although , this is somewhat tradtional in the Rust community.
//! It's [the recommended way to use `bindgen`](https://rust-lang.github.io/rust-bindgen/library-usage.html)
//! and it's the only way to use [`pest`](https://pest.rs/).
//!
//! If you are really concerned about compile times,
//! you can use explicit output (below) to avoid compiling .
//!
//! ## Explicit Output Mode
//! If you want more control over the location of generated grammar files,
//! you can invoke [`Builder::with_output_dir`] to explictly set an output location.
//!
//!
//! This is a very used to power the command line `parol` tool, and is useful for maximum control.
//!
//! By default, it does not make any attempt to integrate with cargo (unless explicitly asked too - see below).
//!
//! Any configured output paths (including generated parsers, expanded grammers, etc)
//! are resolved relative to this base using [Path::join]. This means that specifiying absolute paths
//! overrides this explicit base directory.
//!
//! See the source code for `bin/parol/main.rs` for a detailed example on how to use this.
//!
//! ### Combining with Cargo Integration
//! By default, cargo integration is turned off if you specify an explicit directory.
//! However, it is possible to change this by calling [`Builder::enable_cargo_integration`].
//!
//! In this case, you would probably set the output to a sub-directory of `src`.
//! This means that files are version controlled (instead of put in cargo's `OUT_DIR`)
//! and you would have to commit them whenver changes are made.
//!
//! The disadvantage of this is more machine-generated in your commits.
//! Also if you put it in `build.rs`, you would still require `parol` as a build-dependency,
//! you would just avoid :)
//!
//! ## Internal APIs
//! Because this is used from the main `parol` generate command, this has a number of internal APIs.
//!
//! There are a number of APIs explicitly marked as unstable or internal.
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
pub const DEFUALT_USER_TYPE_NAME: &str = "Grammar";

/// Builds the configuration for generating and analysing `parol` grammars.
///
/// A grammar file is requiered for almost all possible operations (set with [Builder::grammar_file])
///
/// Does not actually generate anything until finished.
#[derive(Clone)]
pub struct Builder {
    /// The base output directory
    output_dir: PathBuf,
    grammar_file: Option<PathBuf>,
    /// Output file for the generated parser source
    parser_output_file: Option<PathBuf>,
    /// Ouptut file for the generated actions files.
    actions_output_file: Option<PathBuf>,
    user_type_name: String,
    module_name: String,
    cargo_integration: bool,
    max_lookahead: usize,
    /// Internal debugging for CLI.
    debug_verbose: bool,
}
impl Builder {
    /// Create a new builder for use in a Cargo build script (`build.rs`).
    ///
    /// This is the recommended default way to get started.
    ///
    /// Panics if used outside of a cargo build script.
    pub fn for_cargo_script() -> Self {
        let mut builder = Self::with_output_dir(
            env::var_os("OUT_DIR").expect("Missing Cargo OUT_DIR. Are you using a build script?"),
        );
        builder.enable_cargo_integration();
        builder
    }
    /// Internal utility to resolve a path relative to the output directory
    fn resolve_path(&self, p: impl AsRef<Path>) -> PathBuf {
        self.output_dir.join(p)
    }
    /// Create a new builder with an explicitly speicfied output directory.
    ///
    /// Disables cargo integration.
    ///
    /// If output files are specified using absolute paths,
    /// it overrides this explicit output dir.
    ///
    /// See module docs on "explicit output mode" for more details.
    pub fn with_output_dir(output: impl AsRef<Path>) -> Self {
        /*
         * Most of these correspond to CLI options.
         */
        Builder {
            output_dir: PathBuf::from(output.as_ref()),
            grammar_file: None,
            cargo_integration: false,
            debug_verbose: false,
            max_lookahead: DEFAULT_MAX_LOOKAHEAD,
            module_name: String::from(DEFAULT_MODULE_NAME),
            user_type_name: String::from(DEFUALT_USER_TYPE_NAME),
            parser_output_file: None,
            actions_output_file: None,
        }
    }
    /// Set the output location for the generated parser.
    ///
    /// By default, the generated parser is output nowhere.
    pub fn parser_output_file(&mut self, p: impl AsRef<Path>) -> &mut Self {
        self.parser_output_file = Some(self.resolve_path(p));
        self
    }
    /// Set the actions output location for the generated parser.
    ///
    /// By default, the generated actions file is output nowhere.
    pub fn actions_output_file(&mut self, p: impl AsRef<Path>) -> &mut Self {
        self.actions_output_file = Some(self.resolve_path(p));
        self
    }
    /// Enable cargo intergration.
    ///
    /// This is automatically set when using [Self::for_cargo_script].
    ///
    /// Does nothing if already enabled.
    pub fn enable_cargo_integration(&mut self) -> &mut Self {
        self.cargo_integration = true;
        self
    }
    /// Set the grammar file used as input for parol.
    ///
    /// This is required for most operations.
    ///
    /// Does not check that the file exists.
    pub fn grammar_file(&mut self, grammar: impl AsRef<Path>) -> &mut Self {
        self.grammar_file = Some(self.resolve_path(grammar));
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
    /// Returns a [BuilderError] if the lookahead is gerater than [crate::MAX_K].
    pub fn max_lookahead(&mut self, k: usize) -> Result<&mut Self, BuilderError> {
        if k > MAX_K {
            return Err(BuilderError::LookaheadTooLarge);
        }
        self.max_lookahead = k;
        Ok(self)
    }
    /// Debug vebrose information to the standard output
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
    /// Returns an error if the build
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
        if self.cargo_integration && self.parser_output_file.is_none() {
            eprintln!("WARNING: Cargo integration but no parser output file");
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
    pub fn generate_grammar(&mut self) -> miette::Result<()> {
        self.begin_generation_with(None)?.generate_grammar()
    }
}

/// Represents in-process grammar generation.
///
/// Most of the time you will want to use [Builder::generate_grammar] to bypass this completely.
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
    /// Generate the parser, writing it to the pre-configured grammar files.
    pub fn generate_grammar(&mut self) -> miette::Result<()> {
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

        // We need to fire twice. Once for `Transformed` to mark intiial transformation and once for `Expanded` to mark
        // our final transformation.
        self.listener
            .on_intermediate_grammar(IntermediateGrammar::Transformed, &*grammar_config)?;
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
        self.state = Some(State::PostPorcessed);
        Ok(())
    }
    #[doc(hidden)]
    pub fn write_output(&mut self) -> miette::Result<()> {
        assert_eq!(self.state, Some(State::PostPorcessed));
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
    PostPorcessed,
    Finished,
}

/// A build listener, for advanced customization of the parser generation.
///
/// This is used by the CLI to implement some of its more advanced options (without cluttering up the main interface).
///
/// The details of this trait are conisdered unstable.
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

/// Marks an intermediate stage of the grammar, in beteween the various transformations that parol does.
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
    /// Indicates that the specified lookahead is too large
    #[error("Maximum lookahead is {}", MAX_K)]
    LookaheadTooLarge,
}
