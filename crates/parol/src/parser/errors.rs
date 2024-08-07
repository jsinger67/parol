use std::path::PathBuf;

use anyhow::anyhow;
use parol_runtime::Location;
use thiserror::Error;

///
/// Error types used by the [crate::parser::ParolGrammar]'s semantic actions
#[derive(Error, Debug)]
pub enum ParolParserError {
    /// Undeclared scanner found. Please declare a scanner via %scanner name {{...}}
    #[error("{context} - Unknown scanner {name}")]
    UnknownScanner {
        /// Context (semantic action) where the error was issued
        context: String,
        /// Name of the unknown scanner state
        name: String,
        /// Source
        input: PathBuf,
        /// Location
        token: Location,
    },

    /// Empty Groups () are not allowed.
    #[error("{context} - Empty Group not allowed")]
    EmptyGroup {
        /// Context (semantic action) where the error was issued
        context: String,
        /// Source
        input: PathBuf,
        /// Start location
        start: Location,
        /// End location
        end: Location,
    },

    /// Empty Optionals [] are not allowed.
    #[error("{context} - Empty Optionals not allowed")]
    EmptyOptional {
        /// Context (semantic action) where the error was issued
        context: String,
        /// Source
        input: PathBuf,
        /// Start location
        start: Location,
        /// End location
        end: Location,
    },

    /// Empty Repetitions {{}} are not allowed.
    #[error("{context} - Empty Repetitions not allowed")]
    EmptyRepetition {
        /// Context (semantic action) where the error was issued
        context: String,
        /// Source
        input: PathBuf,
        /// Start location
        start: Location,
        /// End location
        end: Location,
    },

    /// Multiple token aliases that expand to the same text will produce a terminal conflict.
    #[error(
        r"Multiple token aliases that expand to the same text:
'{first_alias}' and '{second_alias}' expand both to '{expanded}'."
    )]
    ConflictingTokenAliases {
        /// First
        first_alias: String,
        /// Second
        second_alias: String,
        /// Expanded
        expanded: String,
        /// Source
        input: PathBuf,
        /// First alias
        first: Location,
        /// Second alias
        second: Location,
    },

    /// Empty Scanner states are not allowed.
    #[error("Empty scanner states ({empty_scanners:?}) found")]
    EmptyScanners {
        /// Names of the empty scanner states
        empty_scanners: Vec<String>,
    },

    /// Unsupported grammar type
    #[error("{grammar_type} - Unsupported grammar type")]
    UnsupportedGrammarType {
        /// The grammar type found
        grammar_type: String,
        /// Source
        input: PathBuf,
        /// Location
        token: Location,
    },

    /// Unsupported feature
    #[error("{feature} - Unsupported feature")]
    UnsupportedFeature {
        /// The feature found
        feature: String,
        /// Hint
        hint: String,
        /// Source
        input: PathBuf,
        /// Location
        token: Location,
    },

    /// Invalid token in transition, e.g. a token that is not defined in the grammar
    /// is used in a transition. Use a primary non-terminal for the token.
    #[error(
        "{context} - Invalid token '{token}' in transition. Use a primary non-terminal for the token."
    )]
    InvalidTokenInTransition {
        /// Context where the error was issued
        context: String,
        /// Token that is not defined matched against a valid primary non-terminal
        token: String,
        /// Source file
        input: PathBuf,
        /// Location of the token
        location: Location,
    },

    /// The token that is used to initiate a transition is not defined in this scanner.
    #[error("{context} - Token '{token}' is not defined in scanner '{scanner}'")]
    TokenIsNotInScanner {
        /// Context where the error was issued
        context: String,
        /// The scanner where the token is not defined
        scanner: String,
        /// Token that is not defined in the scanner
        token: String,
        /// Source file
        input: PathBuf,
        /// Location of the token
        location: Location,
    },

    /// Mixed scanner switching is not allowed - use either parser-based or scanner-based switching.
    /// Parser-based switching is done via the %sc, %push and %pop directives productions.
    /// Scanner-based switching is done via the %on directive in the header of the grammar file.
    #[error("{context} - Mixed scanner switching is not allowed")]
    MixedScannerSwitching {
        /// Context where the error was issued
        context: String,
        /// Source file
        input: PathBuf,
        /// Location of the token
        location: Location,
    },
}

impl From<ParolParserError> for parol_runtime::ParolError {
    fn from(err: ParolParserError) -> Self {
        parol_runtime::ParolError::UserError(anyhow!(err))
    }
}
