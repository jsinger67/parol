use parol_runtime::{FileSource, Location};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BasicError {
    #[error("{context} value parse error")]
    ParseFloat {
        context: String,
        input: FileSource,
        token: Location,
    },

    #[error("{context} line number parse error")]
    ParseLineNumber {
        context: String,
        input: FileSource,
        token: Location,
    },

    #[error("{context} line number too large")]
    LineNumberTooLarge {
        context: String,
        input: FileSource,
        token: Location,
    },

    #[error("{context} line number already defined")]
    LineNumberDefinedTwice {
        context: String,
        input: FileSource,
        token: Location,
    },

    #[error("{context} line not accessible")]
    LineNumberBeyondLastLine {
        context: String,
        input: FileSource,
        token: Location,
    },
}
