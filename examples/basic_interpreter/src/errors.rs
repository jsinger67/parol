use miette::{Diagnostic, NamedSource, SourceSpan};
use parol::utils::miette_support;
use thiserror::Error;

#[derive(Error, Diagnostic, Debug)]
pub enum BasicError {
    #[error("{context} value parse error")]
    #[diagnostic(
        help("Error parsing number token as valid f32"),
        code(basic::parse_float)
    )]
    ParseFloat {
        context: String,
        #[source_code]
        input: NamedSource,
        #[label("Wrong f32 value")]
        token: SourceSpan,
    },

    #[error("{context} line number parse error")]
    #[diagnostic(
        help("Error parsing line number token as valid u16"),
        code(basic::parse_line_number)
    )]
    ParseLineNumber {
        context: String,
        #[source_code]
        input: NamedSource,
        #[label("Wrong i16 value")]
        token: SourceSpan,
    },

    #[error("{context} line number too large")]
    #[diagnostic(
        help("Line number exceeds maximum of 63999"),
        code(basic::line_number_too_large)
    )]
    LineNumberTooLarge {
        context: String,
        #[source_code]
        input: NamedSource,
        #[label("Line number too large")]
        token: SourceSpan,
    },

    #[error("{context} line number already defined")]
    #[diagnostic(
        help("Line number is already defined"),
        code(basic::line_number_already_defined)
    )]
    LineNumberDefinedTwice {
        context: String,
        #[source_code]
        input: NamedSource,
        #[label("Line number is already defined")]
        token: SourceSpan,
    },

    #[error("{context} line not accessible")]
    #[diagnostic(
        help("Line number is beyond last line"),
        code(basic::line_number_beyond_last_line)
    )]
    LineNumberBeyondLastLine {
        context: String,
        #[source_code]
        input: NamedSource,
        #[label("Line number is beyond last line")]
        token: SourceSpan,
    },
}

/// This is an example of how to chain error conversions from opaque `anyhow::Error` objects to
/// custom `miette::Report` types.
/// The first step is to call `miette_support::to_report` to let their error types be converted.
/// Then we can check for our own error types (actually miette errors wrapped in anyhow errors) and
/// extract them.
pub fn to_report(err: anyhow::Error) -> std::result::Result<miette::Report, anyhow::Error> {
    let err = match miette_support::to_report(err) {
        Ok(err) => return Ok(err),
        Err(err) => err,
    };

    let err = match err.downcast::<BasicError>() {
        Ok(err) => return Ok(miette::Report::new(err)),
        Err(err) => err,
    };

    Err(err)
}
