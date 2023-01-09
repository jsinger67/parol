use std::fs;
use std::ops::Range;
use std::path::Path;

use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term::{self, termcolor::StandardStream};
use parol_errors::{GrammarAnalysisError, ParolParserError};
use parol_runtime::{LexerError, ParolError, ParserError, Span};

pub fn report_parol_error<T>(err: &ParolError, file_name: T) -> anyhow::Result<()>
where
    T: AsRef<Path>,
{
    report_error(err, file_name, Some(&parol_error_reporter))
}

pub type UserErrorReporter = dyn Fn(&anyhow::Error) -> anyhow::Result<()>;

pub fn report_error<T>(
    err: &ParolError,
    file_name: T,
    report_user_error: Option<&UserErrorReporter>,
) -> anyhow::Result<()>
where
    T: AsRef<Path>,
{
    let writer = StandardStream::stderr(term::termcolor::ColorChoice::Auto);
    let config = codespan_reporting::term::Config::default();
    // config.chars.note_bullet = '•';

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
                        "Check balance of %push and %pop directives in your grammar".to_string(),
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
                    report_error(source, file_name, report_user_error)?;
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
                        .with_message(format!("{context}Tried to pop from an empty scanner stack"))
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
        ParolError::UserError(e) => report_user_error.map(|f| f(e)).unwrap(),
    }
}

pub fn parol_error_reporter(err: &anyhow::Error) -> anyhow::Result<()> {
    let files: SimpleFiles<String, String> = SimpleFiles::new();
    let writer = StandardStream::stderr(term::termcolor::ColorChoice::Auto);
    let config = codespan_reporting::term::Config::default();
    // config.chars.note_bullet = '•';

    if let Some(err) = err.downcast_ref::<ParolParserError>() {
        match err {
            ParolParserError::UnknownScanner {
                context,
                name,
                input,
                token,
            } => {
                let mut files = SimpleFiles::new();
                let content = fs::read_to_string(input).unwrap_or_default();
                let file_id = files.add(input.display().to_string(), content);

                return Ok(term::emit(
                        &mut writer.lock(),
                        &config,
                        &files,
                        &Diagnostic::error()
                            .with_message(format!("{context} - Unknown scanner {name}"))
                            .with_code("parol::parser::unknown_scanner")
                            .with_labels(vec![Label::primary(file_id, Into::<Range<usize>>::into(token))])
                            .with_notes(vec!["Undeclared scanner found. Please declare a scanner via %scanner name {{...}}".to_string()])
                    )?);
            }
            ParolParserError::EmptyGroup {
                context,
                input,
                start,
                end,
            } => {
                let mut files = SimpleFiles::new();
                let content = fs::read_to_string(input).unwrap_or_default();
                let file_id = files.add(input.display().to_string(), content);

                return Ok(term::emit(
                    &mut writer.lock(),
                    &config,
                    &files,
                    &Diagnostic::error()
                        .with_message(format!("{context} - Empty Group not allowed"))
                        .with_code("parol::parser::empty_group")
                        .with_labels(vec![
                            Label::primary(file_id, Into::<Range<usize>>::into(start)),
                            Label::primary(file_id, Into::<Range<usize>>::into(end)),
                        ])
                        .with_notes(vec!["Empty groups can be safely removed.".to_string()]),
                )?);
            }
            ParolParserError::EmptyOptional {
                context,
                input,
                start,
                end,
            } => {
                let mut files = SimpleFiles::new();
                let content = fs::read_to_string(input).unwrap_or_default();
                let file_id = files.add(input.display().to_string(), content);

                return Ok(term::emit(
                    &mut writer.lock(),
                    &config,
                    &files,
                    &Diagnostic::error()
                        .with_message(format!("{context} - Empty Optionals not allowed"))
                        .with_code("parol::parser::empty_optional")
                        .with_labels(vec![
                            Label::primary(file_id, Into::<Range<usize>>::into(start)),
                            Label::primary(file_id, Into::<Range<usize>>::into(end)),
                        ])
                        .with_notes(vec!["Empty optionals can be safely removed.".to_string()]),
                )?);
            }
            ParolParserError::EmptyRepetition {
                context,
                input,
                start,
                end,
            } => {
                let mut files = SimpleFiles::new();
                let content = fs::read_to_string(input).unwrap_or_default();
                let file_id = files.add(input.display().to_string(), content);

                return Ok(term::emit(
                    &mut writer.lock(),
                    &config,
                    &files,
                    &Diagnostic::error()
                        .with_message(format!("{context} - Empty Repetitions not allowed"))
                        .with_code("parol::parser::empty_repetition")
                        .with_labels(vec![
                            Label::primary(file_id, Into::<Range<usize>>::into(start)),
                            Label::primary(file_id, Into::<Range<usize>>::into(end)),
                        ])
                        .with_notes(vec!["Empty repetitions can be safely removed.".to_string()]),
                )?);
            }
            ParolParserError::ConflictingTokenAliases {
                first_alias,
                second_alias,
                expanded,
                input,
                first,
                second,
            } => {
                let mut files = SimpleFiles::new();
                let content = fs::read_to_string(input).unwrap_or_default();
                let file_id = files.add(input.display().to_string(), content);

                return Ok(term::emit(
                    &mut writer.lock(),
                    &config,
                    &files,
                    &Diagnostic::error()
                        .with_message(format!(
                            r"Multiple token aliases that expand to the same text:
'{first_alias}' and '{second_alias}' expand both to '{expanded}'."
                        ))
                        .with_code("parol::parser::conflicting_token_aliases")
                        .with_labels(vec![
                            Label::primary(file_id, Into::<Range<usize>>::into(first))
                                .with_message("First alias"),
                            Label::primary(file_id, Into::<Range<usize>>::into(second))
                                .with_message("Second alias"),
                        ])
                        .with_notes(vec![
                            "Consider using only one single terminal instead of two.".to_string(),
                        ]),
                )?);
            }
            ParolParserError::EmptyScanners { empty_scanners } => {
                return Ok(term::emit(
                    &mut writer.lock(),
                    &config,
                    &files,
                    &Diagnostic::error()
                        .with_message(format!("Empty scanner states ({empty_scanners:?}) found"))
                        .with_code("parol::parser::empty_scanner_states")
                        .with_notes(vec![
                            "Assign at least one terminal or remove them.".to_string()
                        ]),
                )?);
            }
        }
    } else if let Some(err) = err.downcast_ref::<GrammarAnalysisError>() {
        match err {
            GrammarAnalysisError::LeftRecursion { recursions } => {
                let non_terminals = recursions
                    .iter()
                    .map(|r| r.name.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                return Ok(term::emit(
                    &mut writer.lock(),
                    &config,
                    &files,
                    &Diagnostic::error()
                        .with_message("Grammar contains left-recursions")
                        .with_code("parol::analysis::left_recursion")
                        .with_notes(vec![
                            "Left-recursions detected.".to_string(),
                            non_terminals,
                            "Please rework your grammar to remove these recursions.".to_string(),
                        ]),
                )?);
            }
            GrammarAnalysisError::UnreachableNonTerminals { non_terminals } => {
                let non_terminals = non_terminals
                    .iter()
                    .map(|r| r.hint.clone())
                    .collect::<Vec<String>>()
                    .join(", ");
                return Ok(term::emit(
                    &mut writer.lock(),
                    &config,
                    &files,
                    &Diagnostic::error()
                        .with_message("Grammar contains unreachable non-terminals")
                        .with_code("parol::analysis::unreachable_non_terminals")
                        .with_notes(vec![
                            "Non-terminals:".to_string(),
                            non_terminals,
                            "Unreachable non-terminals are not allowed. If not used they can be safely removed.".to_string(),
                        ]),
                )?);
            }
            GrammarAnalysisError::NonProductiveNonTerminals { non_terminals } => {
                let non_terminals = non_terminals
                    .iter()
                    .map(|r| r.hint.clone())
                    .collect::<Vec<String>>()
                    .join(", ");
                return Ok(term::emit(
                    &mut writer.lock(),
                    &config,
                    &files,
                    &Diagnostic::error()
                        .with_message("Grammar contains nonproductive non-terminals")
                        .with_code("parol::analysis::nonproductive_non_terminals")
                        .with_notes(vec![
                            "Non-terminals:".to_string(),
                            non_terminals,
                            "Nonproductive non-terminals are not allowed. If not used they can be safely removed.".to_string(),
                        ]),
                )?);
            }
            GrammarAnalysisError::MaxKExceeded { max_k } => {
                return Ok(term::emit(
                    &mut writer.lock(),
                    &config,
                    &files,
                    &Diagnostic::error()
                        .with_message(format!("Maximum lookahead of {max_k} exceeded"))
                        .with_code("parol::analysis::max_k_exceeded")
                        .with_notes(vec!["Please examine your grammar.".to_string()]),
                )?);
            }
        }
    } else {
        let result = term::emit(
            &mut writer.lock(),
            &config,
            &files,
            &Diagnostic::error()
                .with_message("Parol error")
                .with_notes(vec![
                    err.to_string(),
                    err.source()
                        .map_or("No details".to_string(), |s| s.to_string()),
                ]),
        );
        return result.map_err(|e| anyhow::anyhow!(e));
    };
}
