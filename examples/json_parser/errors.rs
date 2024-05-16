use std::{fs, num::ParseFloatError, ops::Range};

use anyhow::anyhow;
use parol_runtime::{
    codespan_reporting::{
        diagnostic::{Diagnostic, Label},
        files::SimpleFiles,
        term::{
            self,
            termcolor::{ColorChoice, StandardStream},
            Config,
        },
    },
    FileSource, Location, Report,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum JsonError {
    #[error("f64 parse error")]
    ParseF64Failed {
        context: String,
        input: FileSource,
        token: Location,
        source: ParseFloatError,
    },
}

pub(crate) struct JSONErrorReporter;
impl Report for JSONErrorReporter {
    fn report_user_error(err: &anyhow::Error) -> anyhow::Result<()> {
        let files: SimpleFiles<String, String> = SimpleFiles::new();
        let writer = StandardStream::stderr(ColorChoice::Auto);
        let config = Config::default();
        if let Some(err) = err.downcast_ref::<JsonError>() {
            match err {
                JsonError::ParseF64Failed {
                    context,
                    input,
                    token,
                    source,
                } => {
                    let mut files = SimpleFiles::new();
                    let content = fs::read_to_string(input.file_name.as_ref()).unwrap_or_default();
                    let file_id = files.add(input.file_name.display().to_string(), content);

                    term::emit(
                        &mut writer.lock(),
                        &config,
                        &files,
                        &Diagnostic::error()
                            .with_message(format!("{context}: value parse error: {source}"))
                            .with_code("basic::parse_float")
                            .with_labels(vec![Label::primary(
                                file_id,
                                Into::<Range<usize>>::into(token),
                            )
                            .with_message("Wrong f32 value")]),
                    )
                    .map_err(|e| anyhow!(e))
                }
            }
        } else {
            term::emit(
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
            )
            .map_err(|e| anyhow!(e))
        }
    }
}
