use std::fs;
use std::ops::Range;
use std::path::Path;

use crate::{LexerError, ParolError, ParserError, Span, SyntaxError};
use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term::{self, termcolor::StandardStream};

/// *Trait for parol's error reporting*
///
/// Implement this trait and provide an own implementation for [Report::report_user_error] when you
/// want to contribute your own error reporting for your error types.
///
/// If you don't want to report own errors then simply use it's default implementation like this:
/// ```
/// use parol_runtime::Report;
/// use parol_macros::parol;
///
/// struct MyErrorReporter;
/// impl Report for MyErrorReporter {};
///
/// let err = parol!("Crucial problem!");   // Suppose that this error comes from a call of `parse`
/// MyErrorReporter::report_error(&err, "my_file.xyz").unwrap_or_default();
/// ```
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
    fn report_user_error(err: &anyhow::Error) -> anyhow::Result<()> {
        let mut writer = StandardStream::stderr(term::termcolor::ColorChoice::Auto);
        let config = codespan_reporting::term::Config::default();
        let files = SimpleFiles::<String, String>::new();
        let result = term::emit_to_write_style(
            &mut writer,
            &config,
            &files,
            &Diagnostic::error()
                .with_message("User error")
                .with_notes(vec![
                    err.to_string(),
                    err.source()
                        .map_or("No details".to_string(), |s| s.to_string()),
                ]),
        );
        result.map_err(|e| anyhow::anyhow!(e))
    }

    /// You don't need to implement this method because it contains the reporting functionality for
    /// errors from parol_runtime itself.
    fn report_error<T>(err: &ParolError, file_name: T) -> anyhow::Result<()>
    where
        T: AsRef<Path>,
    {
        let config = codespan_reporting::term::Config::default();

        let mut files = SimpleFiles::new();
        let content = fs::read_to_string(file_name.as_ref()).unwrap_or_default();
        let file_id = files.add(file_name.as_ref().display().to_string(), content);

        let report_lexer_error = |err: &LexerError| -> anyhow::Result<()> {
            let mut writer = StandardStream::stderr(term::termcolor::ColorChoice::Auto);
            match err {
                LexerError::TokenBufferEmptyError => Ok(term::emit_to_write_style(
                    &mut writer,
                    &config,
                    &files,
                    &Diagnostic::bug()
                        .with_message("No valid token read")
                        .with_code("parol_runtime::lexer::empty_token_buffer")
                        .with_notes(vec!["Token buffer is empty".to_string()]),
                )?),
                LexerError::InternalError(e) => Ok(term::emit_to_write_style(
                    &mut writer,
                    &config,
                    &files,
                    &Diagnostic::bug()
                        .with_message(format!("Internal lexer error: {e}"))
                        .with_code("parol_runtime::lexer::internal_error"),
                )?),
                LexerError::LookaheadExceedsMaximum => Ok(term::emit_to_write_style(
                    &mut writer,
                    &config,
                    &files,
                    &Diagnostic::bug()
                        .with_message("Lookahead exceeds maximum".to_string())
                        .with_code("parol_runtime::lexer::lookahead_exceeds_maximum"),
                )?),
                LexerError::LookaheadExceedsTokenBufferLength => Ok(term::emit_to_write_style(
                    &mut writer,
                    &config,
                    &files,
                    &Diagnostic::bug()
                        .with_message("Lookahead exceeds token buffer length".to_string())
                        .with_code("parol_runtime::lexer::lookahead_exceeds_token_buffer_length"),
                )?),
                LexerError::ScannerStackEmptyError => Ok(term::emit_to_write_style(
                    &mut writer,
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
                LexerError::RecoveryError(e) => Ok(term::emit_to_write_style(
                    &mut writer,
                    &config,
                    &files,
                    &Diagnostic::bug()
                        .with_message(format!("Lexer recovery error: {e}"))
                        .with_code("parol_runtime::lexer::recovery"),
                )?),
            }
        };

        let report_parser_error = |err: &ParserError| -> anyhow::Result<()> {
            let mut writer = StandardStream::stderr(term::termcolor::ColorChoice::Auto);
            match err {
                ParserError::TreeError { source } => Ok(term::emit_to_write_style(
                    &mut writer,
                    &config,
                    &files,
                    &Diagnostic::bug()
                        .with_message(format!("Error from syntree crate: {source}"))
                        .with_code("parol_runtime::parser::syntree_error")
                        .with_notes(vec!["Internal error".to_string()]),
                )?),
                ParserError::DataError(e) => Ok(term::emit_to_write_style(
                    &mut writer,
                    &config,
                    &files,
                    &Diagnostic::bug()
                        .with_message(format!("Data error: {e}"))
                        .with_code("parol_runtime::lexer::internal_error")
                        .with_notes(vec!["Error in generated source".to_string()]),
                )?),
                ParserError::PredictionError { cause } => Ok(term::emit_to_write_style(
                    &mut writer,
                    &config,
                    &files,
                    &Diagnostic::error()
                        .with_message("Error in input")
                        .with_code("parol_runtime::lookahead::production_prediction_error")
                        .with_notes(vec![cause.to_string()]),
                )?),
                ParserError::SyntaxErrors { entries } => {
                    entries.iter().try_for_each(
                        |SyntaxError {
                             cause,
                             error_location,
                             unexpected_tokens,
                             expected_tokens,
                             source,
                             ..
                         }|
                         -> anyhow::Result<()> {
                            if let Some(source) = source {
                                Self::report_error(source, file_name.as_ref())?;
                            }
                            let range: Range<usize> = if unexpected_tokens.is_empty() {
                                (&**error_location).into()
                            } else {
                                (&unexpected_tokens[0].token).into()
                            };
                            let unexpected_tokens_labels =
                                unexpected_tokens.iter().fold(vec![], |mut acc, un| {
                                    acc.push(
                                        Label::secondary(
                                            file_id,
                                            Into::<Range<usize>>::into(&un.token),
                                        )
                                        .with_message(un.token_type.clone()),
                                    );
                                    acc
                                });
                            Ok(term::emit_to_write_style(
                                &mut writer,
                                &config,
                                &files,
                                &Diagnostic::error()
                                    .with_message("Syntax error")
                                    .with_code("parol_runtime::parser::syntax_error")
                                    .with_labels(vec![
                                        Label::primary(file_id, range).with_message("Found"),
                                    ])
                                    .with_labels(unexpected_tokens_labels)
                                    .with_notes(vec![
                                        format!("Expecting {}", expected_tokens),
                                        cause.to_string(),
                                    ]),
                            )?)
                        },
                    )?;
                    Ok(term::emit_to_write_style(
                        &mut writer,
                        &config,
                        &files,
                        &Diagnostic::error()
                            .with_message(format!("{} syntax error(s) found", entries.len())),
                    )?)
                }
                ParserError::UnprocessedInput { last_token, .. } => {
                    let un_span: Span = (Into::<Range<usize>>::into(&**last_token)).into();
                    Ok(term::emit_to_write_style(
                        &mut writer,
                        &config,
                        &files,
                        &Diagnostic::error()
                            .with_message("Unprocessed input is left after parsing has finished")
                            .with_code("parol_runtime::parser::unprocessed_input")
                            .with_labels(vec![
                                Label::primary(file_id, un_span).with_message("Unprocessed"),
                            ])
                            .with_notes(vec![
                                "Unprocessed input could be a problem in your grammar.".to_string(),
                            ]),
                    )?)
                }
                ParserError::Unsupported {
                    context,
                    error_location,
                } => {
                    let range: Range<usize> = (&**error_location).into();
                    Ok(term::emit_to_write_style(
                        &mut writer,
                        &config,
                        &files,
                        &Diagnostic::error()
                            .with_message("Unsupported language feature")
                            .with_code("parol_runtime::parser::unsupported")
                            .with_labels(vec![
                                Label::primary(file_id, range).with_message("Unsupported"),
                            ])
                            .with_notes(vec![format!("Context: {context}")]),
                    )?)
                }
                ParserError::InternalError(e) => Ok(term::emit_to_write_style(
                    &mut writer,
                    &config,
                    &files,
                    &Diagnostic::bug()
                        .with_message(format!("Internal parser error: {e}"))
                        .with_code("parol_runtime::parser::internal_error")
                        .with_notes(vec!["This may be a bug. Please report it!".to_string()]),
                )?),
                ParserError::TooManyErrors { count } => Ok(term::emit_to_write_style(
                    &mut writer,
                    &config,
                    &files,
                    &Diagnostic::error()
                        .with_message(format!("Too many errors: {count}"))
                        .with_code("parol_runtime::parser::too_many_errors")
                        .with_notes(vec![
                            "The parser has stopped because too many errors occurred.".to_string(),
                        ]),
                )?),
                ParserError::RecoveryFailed => Ok(term::emit_to_write_style(
                    &mut writer,
                    &config,
                    &files,
                    &Diagnostic::error()
                        .with_message("Error recovery failed")
                        .with_code("parol_runtime::parser::recovery_failed")
                        .with_notes(vec![
                            "The parser has stopped because error recovery failed.".to_string(),
                        ]),
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
