use miette::{Diagnostic, NamedSource, SourceSpan};

///
/// Error types used by the [crate::parser::ParolGrammar]'s semantic actions
#[derive(Error, Diagnostic, Debug)]
pub enum ParolParserError {
    /// Undeclared scanner found. Pease declare a scanner via %scanner name {{...}}
    #[error("{context} - Unknown scanner {name}")]
    #[diagnostic(
        help("Undeclared scanner found. Please declare a scanner via %scanner name {{...}}"),
        code(parol::parser::unknown_scanner)
    )]
    UnknownScanner {
        /// Context (semantic action) where the error was issued
        context: String,
        /// Name of the unknown scanner state
        name: String,
        /// Source
        #[source_code]
        input: NamedSource,
        /// Location
        #[label("Undeclared")]
        token: SourceSpan,
    },

    /// Empty Groups () are not allowed.
    #[error("{context} - Empty Group not allowed")]
    #[diagnostic(
        help("Empty Groups () are not allowed."),
        code(parol::parser::empty_group)
    )]
    EmptyGroup {
        /// Context (semantic action) where the error was issued
        context: String,
        /// Source
        #[source_code]
        input: NamedSource,
        /// Start location
        #[label("Start")]
        start: SourceSpan,
        /// End location
        #[label("End")]
        end: SourceSpan,
    },

    /// Empty Optionals [] are not allowed.
    #[error("{context} - Empty Optionals not allowed")]
    #[diagnostic(
        help("Empty Optionals [] are not allowed."),
        code(parol::parser::empty_optional)
    )]
    EmptyOptional {
        /// Context (semantic action) where the error was issued
        context: String,
        /// Source
        #[source_code]
        input: NamedSource,
        /// Start location
        #[label("Start")]
        start: SourceSpan,
        /// End location
        #[label("End")]
        end: SourceSpan,
    },

    /// Empty Repetitions {{}} are not allowed.
    #[error("{context} - Empty Repetitions not allowed")]
    #[diagnostic(
        help("Empty Repetitions {{}} are not allowed."),
        code(parol::parser::empty_repetition)
    )]
    EmptyRepetition {
        /// Context (semantic action) where the error was issued
        context: String,
        /// Source
        #[source_code]
        input: NamedSource,
        /// Start location
        #[label("Start")]
        start: SourceSpan,
        /// End location
        #[label("End")]
        end: SourceSpan,
    },

    /// Multiple token aliases that expand to the same text will produce a terminal conflict.
    #[error(
        r"Multiple token aliases that expand to the same text:
'{first_alias}' and '{second_alias}' expand both to '{expanded}'."
    )]
    #[diagnostic(
        help(
            r"Multiple token aliases that expand to the same text will produce a terminal conflict.
Consider using only one single terminal instead of two."
        ),
        code(parol::parser::conflicting_token_aliases)
    )]
    ConflictingTokenAliases {
        /// First
        first_alias: String,
        /// Second
        second_alias: String,
        /// Expanded
        expanded: String,
        /// Source
        #[source_code]
        input: NamedSource,
        /// First alias
        #[label("First alias")]
        first: SourceSpan,
        /// Second alias
        #[label("Second alias")]
        second: SourceSpan,
    },

    /// Empty Scanner states are not allowed.
    #[error("Empty scanner states ({empty_scanners:?}) found")]
    #[diagnostic(
        help("Assign at least one terminal or remove it!"),
        code(parol::parser::empty_scanner_states)
    )]
    EmptyScanners {
        /// Names of the empty scanner states
        empty_scanners: Vec<String>,
    },
}
