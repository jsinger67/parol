use std::fs;
use std::ops::Range;
use std::path::Path;

use crate::{LexerError, ParolError, ParserError, Span};
use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term::{self, termcolor::StandardStream};

/// Trait for parol's error reporting
/// Implement this trait when you want to provide your own error reporting for your own error types
pub trait Report {
    ///
    /// Implement this method if you want to provide your own error reporting for your own error
    /// types.
    /// Doing so you can hook into the error reporting process.
    ///
    /// Examples are `parol`'s `ParolErrorReporter` or `basic_interpreter`'s `BasicErrorReporter`.
    ///
    /// The method's argument value is obtained from a `parol_runtime::ParolError::UserError`'s
    /// content. It should return Ok(()) if reporting succeeds and an error value if the reporting
    /// itself fails somehow.
    ///
    fn report_user_error(_err: &anyhow::Error) -> anyhow::Result<()> {
        Ok(())
    }

    /// You don't need to implement this method because it contains the reporting functionality for
    /// errors from parol_runtime itself.
    fn report_error<T>(err: &ParolError, file_name: T) -> anyhow::Result<()>
    where
        T: AsRef<Path>,
    {
        let writer = StandardStream::stderr(term::termcolor::ColorChoice::Auto);
        let config = codespan_reporting::term::Config::default();

        let mut files = SimpleFiles::new();
        let content = fs::read_to_string(file_name.as_ref()).unwrap_or_default();
        let file_id = files.add(file_name.as_ref().display().to_string(), content);

        let report_lexer_error = |err: &LexerError| -> anyhow::Result<()> {
            match err {
                LexerError::DataError(e) => Ok(term::emit(
                    &mut writer.lock(),
                    &config,
                    &files,
                    &Diagnostic::bug()
                        .with_message(format!("Data error: {e}"))
                        .with_code("parol_runtime::lexer::internal_error")
                        .with_notes(vec!["Error in generated source".to_string()]),
                )?),
                LexerError::PredictionError { cause } => Ok(term::emit(
                    &mut writer.lock(),
                    &config,
                    &files,
                    &Diagnostic::error()
                        .with_message("Error in input")
                        .with_code("parol_runtime::lookahead::production_prediction_error")
                        .with_notes(vec![cause.to_string()]),
                )?),
                LexerError::TokenBufferEmptyError => Ok(term::emit(
                    &mut writer.lock(),
                    &config,
                    &files,
                    &Diagnostic::bug()
                        .with_message("No valid token read")
                        .with_code("parol_runtime::lexer::empty_token_buffer")
                        .with_notes(vec!["Token buffer is empty".to_string()]),
                )?),
                LexerError::InternalError(e) => Ok(term::emit(
                    &mut writer.lock(),
                    &config,
                    &files,
                    &Diagnostic::bug()
                        .with_message(format!("Internal lexer error: {e}"))
                        .with_code("parol_runtime::lexer::internal_error"),
                )?),
                LexerError::LookaheadExceedsMaximum => Ok(term::emit(
                    &mut writer.lock(),
                    &config,
                    &files,
                    &Diagnostic::bug()
                        .with_message("Lookahead exceeds maximum".to_string())
                        .with_code("parol_runtime::lexer::lookahead_exceeds_maximum"),
                )?),
                LexerError::LookaheadExceedsTokenBufferLength => Ok(term::emit(
                    &mut writer.lock(),
                    &config,
                    &files,
                    &Diagnostic::bug()
                        .with_message("Lookahead exceeds token buffer length".to_string())
                        .with_code("parol_runtime::lexer::lookahead_exceeds_token_buffer_length"),
                )?),
                LexerError::ScannerStackEmptyError => Ok(term::emit(
                    &mut writer.lock(),
                    &config,
                    &files,
                    &Diagnostic::bug()
                        .with_message("Tried to pop from empty scanner stack".to_string())
                        .with_code("parol_runtime::lexer::pop_from_empty_scanner_stack")
                        .with_notes(vec![
                            "Check balance of %push and %pop directives in your grammar"
                                .to_string(),
                        ]),
                )?),
            }
        };

        let report_parser_error = |err: &ParserError| -> anyhow::Result<()> {
            match err {
                ParserError::IdTreeError { source } => Ok(term::emit(
                    &mut writer.lock(),
                    &config,
                    &files,
                    &Diagnostic::bug()
                        .with_message(format!("Error from id_tree crate: {}", source))
                        .with_code("parol_runtime::parser::id_tree_error")
                        .with_notes(vec!["Internal error".to_string()]),
                )?),
                ParserError::PredictionErrorWithExpectations {
                    cause,
                    unexpected_tokens,
                    expected_tokens,
                    source,
                    ..
                } => {
                    if let Some(source) = source {
                        Self::report_error(source, file_name)?;
                    }
                    let range = unexpected_tokens
                        .iter()
                        .fold(Range::default(), |mut acc, un| {
                            let un_span: Span = (Into::<Range<usize>>::into(&un.token)).into();
                            let acc_span: Span = acc.into();
                            acc = (acc_span + un_span).into();
                            acc
                        });
                    let unexpected_tokens_labels =
                        unexpected_tokens.iter().fold(vec![], |mut acc, un| {
                            acc.push(
                                Label::secondary(file_id, Into::<Range<usize>>::into(&un.token))
                                    .with_message(un.token_type.clone()),
                            );
                            acc
                        });
                    Ok(term::emit(
                        &mut writer.lock(),
                        &config,
                        &files,
                        &Diagnostic::error()
                            .with_message("Syntax error")
                            .with_code("parol_runtime::parser::syntax_error")
                            .with_labels(vec![Label::primary(file_id, range).with_message("Found")])
                            .with_labels(unexpected_tokens_labels)
                            .with_notes(vec![
                                "Expecting one of".to_string(),
                                expected_tokens.to_string(),
                            ])
                            .with_notes(vec![cause.to_string()]),
                    )?)
                }
                ParserError::UnprocessedInput { last_token, .. } => {
                    let un_span: Span = (Into::<Range<usize>>::into(last_token)).into();
                    Ok(term::emit(
                        &mut writer.lock(),
                        &config,
                        &files,
                        &Diagnostic::error()
                            .with_message("Unprocessed input is left after parsing has finished")
                            .with_code("parol_runtime::parser::unprocessed_input")
                            .with_labels(vec![
                                Label::primary(file_id, un_span).with_message("Unprocessed")
                            ])
                            .with_notes(vec![
                                "Unprocessed input could be a problem in your grammar.".to_string(),
                            ]),
                    )?)
                }
                ParserError::PopOnEmptyScannerStateStack {
                    context, source, ..
                } => {
                    report_lexer_error(source)?;
                    Ok(term::emit(
                        &mut writer.lock(),
                        &config,
                        &files,
                        &Diagnostic::error()
                            .with_message(format!(
                                "{context}Tried to pop from an empty scanner stack"
                            ))
                            .with_code("parol_runtime::parser::pop_on_empty_scanner_stack"),
                    )?)
                }
                ParserError::InternalError(e) => Ok(term::emit(
                    &mut writer.lock(),
                    &config,
                    &files,
                    &Diagnostic::bug()
                        .with_message(format!("Internal parser error: {e}"))
                        .with_code("parol_runtime::parser::internal_error")
                        .with_notes(vec!["This may be a bug. Please report it!".to_string()]),
                )?),
            }
        };

        match err {
            ParolError::ParserError(e) => report_parser_error(e),
            ParolError::LexerError(e) => report_lexer_error(e),
            ParolError::UserError(e) => Self::report_user_error(e),
        }
    }
}
