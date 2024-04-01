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
}

impl From<ParolParserError> for parol_runtime::ParolError {
    fn from(err: ParolParserError) -> Self {
        parol_runtime::ParolError::UserError(anyhow!(err))
    }
}
