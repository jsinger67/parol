//! ------------------------------------------------------------------------------------------------
//! This module provides a thin wrapper for miette errors to leverage miette's fancy error messages
//! ------------------------------------------------------------------------------------------------
use miette::{miette, Diagnostic, MietteError, NamedSource, SourceCode, SourceSpan, SpanContents};
use parol_runtime::TokenVec;
use thiserror::Error;

// -------------------------------------------------------------------------------------------------
// Errors from crate `parol_runtime`
// -------------------------------------------------------------------------------------------------

#[derive(Error, Diagnostic, Debug)]
pub enum ParserError {
    #[error(transparent)]
    #[diagnostic(
        help("Error from id_tree crate"),
        code(parol_runtime::parser::id_tree_error)
    )]
    IdTreeError {
        #[from]
        source: id_tree::NodeIdError,
    },

    #[error("{cause}Expecting one of {expected_tokens}")]
    #[diagnostic(
        help("Syntax error in input prevents prediction of next production"),
        code(parol_runtime::parser::syntax_error)
    )]
    PredictionErrorWithExpectations {
        cause: String,
        #[source_code]
        input: NamedSource,
        #[label("Error location")]
        error_location: SourceSpan,
        #[related("Unexpected tokens")]
        unexpected_tokens: Vec<UnexpectedToken>,
        expected_tokens: TokenVec,
        source: Option<anyhow::Error>,
    },

    #[error("Unprocessed input is left after parsing has finished")]
    #[diagnostic(
        help("Unprocessed input is left after parsing has finished"),
        code(parol_runtime::parser::unprocessed_input)
    )]
    UnprocessedInput {
        #[source_code]
        input: NamedSource,
        #[label("Last processed token")]
        last_token: SourceSpan,
    },

    #[error("{context}Tried to pop from an empty scanner stack")]
    #[diagnostic(
        help("Tried to pop from an empty scanner stack"),
        code(parol_runtime::parser::pop_on_empty_scanner_stack)
    )]
    PopOnEmptyScannerStateStack {
        context: String,
        #[source_code]
        input: NamedSource,
        source: anyhow::Error,
    },

    #[error("{0}")]
    #[diagnostic(
        help("Unexpected internal state"),
        code(parol_runtime::parser::internal_error)
    )]
    InternalError(String),
}

impl From<parol_runtime::ParserError> for ParserError {
    fn from(value: parol_runtime::ParserError) -> Self {
        match value {
            parol_runtime::ParserError::IdTreeError { source } => {
                ParserError::IdTreeError { source }
            }
            parol_runtime::ParserError::PredictionErrorWithExpectations {
                cause,
                input,
                error_location,
                unexpected_tokens,
                expected_tokens,
                source,
            } => ParserError::PredictionErrorWithExpectations {
                cause: cause.into(),
                input: MyFileSource(input).into(),
                error_location: MyLocation(error_location).into(),
                unexpected_tokens: MyUnexpectedToken(unexpected_tokens).into(),
                expected_tokens,
                source,
            },
            parol_runtime::ParserError::UnprocessedInput { input, last_token } => {
                ParserError::UnprocessedInput {
                    input: MyFileSource(input).into(),
                    last_token: MyLocation(last_token).into(),
                }
            }
            parol_runtime::ParserError::PopOnEmptyScannerStateStack {
                context,
                input,
                source,
            } => ParserError::PopOnEmptyScannerStateStack {
                context,
                input: MyFileSource(input).into(),
                source,
            },
            parol_runtime::ParserError::InternalError(msg) => ParserError::InternalError(msg),
        }
    }
}

#[derive(Error, Diagnostic, Debug)]
pub enum LexerError {
    #[error("{0}")]
    #[diagnostic(
        help("Error in generated source"),
        code(parol_runtime::lookahead::generation_error)
    )]
    DataError(&'static str),

    #[error("{cause}")]
    #[diagnostic(
        help("Error in input"),
        code(parol_runtime::lookahead::production_prediction_error)
    )]
    PredictionError { cause: String },

    #[error("No valid token read")]
    #[diagnostic(
        help("No valid token read"),
        code(parol_runtime::lookahead::empty_token_buffer)
    )]
    TokenBufferEmptyError,
}

impl From<parol_runtime::LexerError> for LexerError {
    fn from(value: parol_runtime::LexerError) -> Self {
        match value {
            parol_runtime::LexerError::DataError(msg) => LexerError::DataError(msg),
            parol_runtime::LexerError::PredictionError { cause } => {
                LexerError::PredictionError { cause }
            }
            parol_runtime::LexerError::TokenBufferEmptyError => LexerError::TokenBufferEmptyError,
        }
    }
}

#[derive(Error, Diagnostic, Debug)]
#[error("Unexpected token: {name} ({token_type})")]
#[diagnostic(help("Unexpected token"), code(parol_runtime::unexpected_token))]
pub struct UnexpectedToken {
    name: String,
    token_type: String,
    #[label("Unexpected token")]
    pub(crate) token: SourceSpan,
}

// -------------------------------------------------------------------------------------------------
// Errors from module `parol::parser`
// -------------------------------------------------------------------------------------------------

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

impl From<parol::ParolParserError> for ParolParserError {
    fn from(value: parol::ParolParserError) -> Self {
        match value {
            parol::ParolParserError::UnknownScanner {
                context,
                name,
                input,
                token,
            } => ParolParserError::UnknownScanner {
                context,
                name,
                input: MyFileSource(input).into(),
                token: MyLocation(token).into(),
            },
            parol::ParolParserError::EmptyGroup {
                context,
                input,
                start,
                end,
            } => ParolParserError::EmptyGroup {
                context,
                input: MyFileSource(input).into(),
                start: MyLocation(start).into(),
                end: MyLocation(end).into(),
            },
            parol::ParolParserError::EmptyOptional {
                context,
                input,
                start,
                end,
            } => ParolParserError::EmptyOptional {
                context,
                input: MyFileSource(input).into(),
                start: MyLocation(start).into(),
                end: MyLocation(end).into(),
            },
            parol::ParolParserError::EmptyRepetition {
                context,
                input,
                start,
                end,
            } => ParolParserError::EmptyRepetition {
                context,
                input: MyFileSource(input).into(),
                start: MyLocation(start).into(),
                end: MyLocation(end).into(),
            },
            parol::ParolParserError::ConflictingTokenAliases {
                first_alias,
                second_alias,
                expanded,
                input,
                first,
                second,
            } => ParolParserError::ConflictingTokenAliases {
                first_alias,
                second_alias,
                expanded,
                input: MyFileSource(input).into(),
                first: MyLocation(first).into(),
                second: MyLocation(second).into(),
            },
            parol::ParolParserError::EmptyScanners { empty_scanners } => {
                ParolParserError::EmptyScanners { empty_scanners }
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------
// Errors from module `parol::analysis`
// -------------------------------------------------------------------------------------------------

/// Error type used by the [crate::analysis] module
#[derive(Error, Diagnostic, Debug)]
pub enum GrammarAnalysisError {
    /// Left-recursions detected
    #[error("Grammar contains left-recursions")]
    #[diagnostic(
        help("Left-recursions detected. Please rework your grammar to remove these recursions"),
        code(parol::analysis::left_recursion)
    )]
    LeftRecursion {
        /// Recursions
        #[related]
        recursions: Vec<RecursiveNonTerminal>,
    },

    /// Unreachable non-terminals are not allowed.
    #[error("Grammar contains unreachable non-terminals")]
    #[diagnostic(
        help("If not used they can safely be removed"),
        code(parol::analysis::unreachable_non_terminals)
    )]
    UnreachableNonTerminals {
        /// Non-terminals
        #[related]
        non_terminals: Vec<RelatedHint>,
    },

    /// Nonproductive non-terminals are not allowed.
    #[error("Grammar contains nonproductive non-terminals")]
    #[diagnostic(
        help("If not used they can safely be removed"),
        code(parol::analysis::nonproductive_non_terminals)
    )]
    NonProductiveNonTerminals {
        /// Non-terminals
        #[related]
        non_terminals: Vec<RelatedHint>,
    },

    /// Maximum lookahead exceeded.
    #[error("Maximum lookahead of {max_k} exceeded")]
    #[diagnostic(
        help("Please examine your grammar"),
        code(parol::analysis::max_k_exceeded)
    )]
    MaxKExceeded {
        /// Maximum lookahead
        max_k: usize,
    },
}

impl From<parol::GrammarAnalysisError> for GrammarAnalysisError {
    fn from(value: parol::GrammarAnalysisError) -> Self {
        match value {
            parol::GrammarAnalysisError::LeftRecursion { recursions } => {
                GrammarAnalysisError::LeftRecursion {
                    recursions: MyRecursiveNonTerminals(recursions).into(),
                }
            }
            parol::GrammarAnalysisError::UnreachableNonTerminals { non_terminals } => {
                GrammarAnalysisError::UnreachableNonTerminals {
                    non_terminals: MyRelatedHints(non_terminals).into(),
                }
            }
            parol::GrammarAnalysisError::NonProductiveNonTerminals { non_terminals } => {
                GrammarAnalysisError::NonProductiveNonTerminals {
                    non_terminals: MyRelatedHints(non_terminals).into(),
                }
            }
            parol::GrammarAnalysisError::MaxKExceeded { max_k } => {
                GrammarAnalysisError::MaxKExceeded { max_k }
            }
        }
    }
}

/// A single recursive non-terminal
#[derive(Error, Diagnostic, Debug)]
#[error("Recursive non-terminal #{number}: '{name}'")]
pub struct RecursiveNonTerminal {
    /// The number of the recursion path
    pub number: usize,
    /// non-terminal
    pub name: String,
}

/// Related information
#[derive(Error, Diagnostic, Debug)]
#[error("{topic}: {hint}")]
pub struct RelatedHint {
    /// A topic or a category to describe the hint
    pub topic: String,
    /// Information
    pub hint: String,
}

// -------------------------------------------------------------------------------------------------
// Conversion types and functions
// -------------------------------------------------------------------------------------------------

struct MyFileSource(parol_runtime::FileSource);

impl SourceCode for MyFileSource {
    fn read_span<'a>(
        &'a self,
        span: &SourceSpan,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> Result<Box<dyn SpanContents<'a> + 'a>, MietteError> {
        <str as SourceCode>::read_span(
            &self.0.input,
            span,
            context_lines_before,
            context_lines_after,
        )
    }
}

impl From<MyFileSource> for NamedSource {
    fn from(file_source: MyFileSource) -> Self {
        let file_name = file_source.0.file_name.clone();
        let file_name = file_name.to_str().unwrap_or("<Bad file name>");
        Self::new(file_name, file_source)
    }
}

struct MyLocation(parol_runtime::Location);

impl From<MyLocation> for SourceSpan {
    fn from(location: MyLocation) -> Self {
        SourceSpan::new(
            (location.0.scanner_switch_pos + location.0.offset).into(),
            location.0.length.into(),
        )
    }
}

struct MyUnexpectedToken(Vec<parol_runtime::UnexpectedToken>);

impl From<MyUnexpectedToken> for Vec<UnexpectedToken> {
    fn from(value: MyUnexpectedToken) -> Self {
        value
            .0
            .into_iter()
            .map(|v| UnexpectedToken {
                name: v.name,
                token_type: v.token_type,
                token: MyLocation(v.token).into(),
            })
            .collect::<Vec<UnexpectedToken>>()
    }
}

struct MyRecursiveNonTerminals(Vec<parol::RecursiveNonTerminal>);

impl From<MyRecursiveNonTerminals> for Vec<RecursiveNonTerminal> {
    fn from(value: MyRecursiveNonTerminals) -> Self {
        value
            .0
            .into_iter()
            .map(|v| RecursiveNonTerminal {
                number: v.number,
                name: v.name,
            })
            .collect::<Vec<RecursiveNonTerminal>>()
    }
}

struct MyRelatedHints(Vec<parol::RelatedHint>);

impl From<MyRelatedHints> for Vec<RelatedHint> {
    fn from(value: MyRelatedHints) -> Self {
        value
            .0
            .into_iter()
            .map(|v| RelatedHint {
                topic: v.topic,
                hint: v.hint,
            })
            .collect::<Vec<RelatedHint>>()
    }
}

pub(crate) fn to_report(err: anyhow::Error) -> miette::Report {
    let err = match err.downcast::<parol_runtime::ParserError>() {
        Ok(err) => return miette!(<parol_runtime::ParserError as Into<ParserError>>::into(err)),
        Err(err) => err,
    };

    let err = match err.downcast::<parol_runtime::LexerError>() {
        Ok(err) => return miette!(<parol_runtime::LexerError as Into<LexerError>>::into(err)),
        Err(err) => err,
    };

    let err = match err.downcast::<parol::ParolParserError>() {
        Ok(err) => {
            return miette!(<parol::ParolParserError as Into<ParolParserError>>::into(
                err
            ))
        }
        Err(err) => err,
    };

    let err = match err.downcast::<parol::GrammarAnalysisError>() {
        Ok(err) => {
            return miette!(<parol::GrammarAnalysisError as Into<
                GrammarAnalysisError,
            >>::into(err))
        }
        Err(err) => err,
    };

    miette!(err)
}
