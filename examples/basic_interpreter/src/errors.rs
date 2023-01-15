use std::fs;
use std::ops::Range;

use parol_runtime::{
    codespan_reporting::{
        self,
        diagnostic::{Diagnostic, Label},
        files::SimpleFiles,
        term::{self, termcolor::StandardStream},
    },
    FileSource, Location, Report,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BasicError {
    #[error("{context}: value parse error")]
    ParseFloat {
        context: String,
        input: FileSource,
        token: Location,
    },

    #[error("{context}: line number parse error")]
    ParseLineNumber {
        context: String,
        input: FileSource,
        token: Location,
    },

    #[error("{context}: line number too large")]
    LineNumberTooLarge {
        context: String,
        input: FileSource,
        token: Location,
    },

    #[error("{context}: line number already defined")]
    LineNumberDefinedTwice {
        context: String,
        input: FileSource,
        token: Location,
    },

    #[error("{context}: line not accessible")]
    LineNumberBeyondLastLine {
        context: String,
        input: FileSource,
        token: Location,
    },
}

pub struct BasicErrorReporter {}

impl Report for BasicErrorReporter {
    fn report_user_error(err: &anyhow::Error) -> anyhow::Result<()> {
        let files: SimpleFiles<String, String> = SimpleFiles::new();
        let writer = StandardStream::stderr(term::termcolor::ColorChoice::Auto);
        let config = codespan_reporting::term::Config::default();
        // config.chars.note_bullet = '•';

        if let Some(err) = err.downcast_ref::<BasicError>() {
            match err {
                BasicError::ParseFloat {
                    context,
                    input,
                    token,
                } => {
                    let mut files = SimpleFiles::new();
                    let content = fs::read_to_string(input.file_name.as_ref()).unwrap_or_default();
                    let file_id = files.add(input.file_name.display().to_string(), content);

                    return Ok(term::emit(
                        &mut writer.lock(),
                        &config,
                        &files,
                        &Diagnostic::error()
                            .with_message(format!("{context}: value parse error"))
                            .with_code("basic::parse_float")
                            .with_labels(vec![Label::primary(
                                file_id,
                                Into::<Range<usize>>::into(token),
                            )
                            .with_message("Wrong f32 value")]),
                    )?);
                }
                BasicError::ParseLineNumber {
                    context,
                    input,
                    token,
                } => {
                    let mut files = SimpleFiles::new();
                    let content = fs::read_to_string(input.file_name.as_ref()).unwrap_or_default();
                    let file_id = files.add(input.file_name.display().to_string(), content);

                    return Ok(term::emit(
                        &mut writer.lock(),
                        &config,
                        &files,
                        &Diagnostic::error()
                            .with_message(format!("{context}: line number parse error"))
                            .with_code("basic::parse_line_number")
                            .with_labels(vec![Label::primary(
                                file_id,
                                Into::<Range<usize>>::into(token),
                            )
                            .with_message("Wrong i16 value")])
                            .with_notes(vec![
                                "Error parsing line number token as valid u16".to_string()
                            ]),
                    )?);
                }
                BasicError::LineNumberTooLarge {
                    context,
                    input,
                    token,
                } => {
                    let mut files = SimpleFiles::new();
                    let content = fs::read_to_string(input.file_name.as_ref()).unwrap_or_default();
                    let file_id = files.add(input.file_name.display().to_string(), content);

                    return Ok(term::emit(
                        &mut writer.lock(),
                        &config,
                        &files,
                        &Diagnostic::error()
                            .with_message(format!("{context}: line number too large"))
                            .with_code("basic::line_number_too_large")
                            .with_labels(vec![Label::primary(
                                file_id,
                                Into::<Range<usize>>::into(token),
                            )
                            .with_message("Line number too large")])
                            .with_notes(vec!["Line number exceeds maximum of 63999".to_string()]),
                    )?);
                }
                BasicError::LineNumberDefinedTwice {
                    context,
                    input,
                    token,
                } => {
                    let mut files = SimpleFiles::new();
                    let content = fs::read_to_string(input.file_name.as_ref()).unwrap_or_default();
                    let file_id = files.add(input.file_name.display().to_string(), content);

                    return Ok(term::emit(
                        &mut writer.lock(),
                        &config,
                        &files,
                        &Diagnostic::error()
                            .with_message(format!("{context}: line number already defined"))
                            .with_code("basic::line_number_already_defined")
                            .with_labels(vec![Label::primary(
                                file_id,
                                Into::<Range<usize>>::into(token),
                            )
                            .with_message("Line number is already defined")])
                            .with_notes(vec!["Define a new line number".to_string()]),
                    )?);
                }
                BasicError::LineNumberBeyondLastLine {
                    context,
                    input,
                    token,
                } => {
                    let mut files = SimpleFiles::new();
                    let content = fs::read_to_string(input.file_name.as_ref()).unwrap_or_default();
                    let file_id = files.add(input.file_name.display().to_string(), content);

                    return Ok(term::emit(
                        &mut writer.lock(),
                        &config,
                        &files,
                        &Diagnostic::error()
                            .with_message(format!("{context}: line not accessible"))
                            .with_code("basic::line_number_beyond_last_line")
                            .with_labels(vec![Label::primary(
                                file_id,
                                Into::<Range<usize>>::into(token),
                            )
                            .with_message("Line number is beyond last line")])
                            .with_notes(vec![
                                "Check the jump destination's line number".to_string()
                            ]),
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
            result.map_err(|e| anyhow::anyhow!(e))
        }
    }
}
