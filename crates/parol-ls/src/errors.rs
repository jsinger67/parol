use std::path::PathBuf;

use lsp_types::Range;
use thiserror::Error;

///
/// Error types used by the language server
///
#[derive(Error, Debug)]
pub enum ServerError {
    /// Unknown document referred during a request
    #[error("Unknown document {path}")]
    UnknownDocument {
        /// Path of the unknown document
        path: PathBuf,
    },
    /// Left recursions in input grammar
    #[error("Left recursions in input grammar")]
    LeftRecursions {
        /// Recursion paths
        paths: Vec<Vec<Range>>,
    },
    /// Unexpected error in communication protocol
    #[error("Unexpected error in communication protocol {err}")]
    ProtocolError { err: Box<dyn std::error::Error> },
    /// Unexpected error in communication protocol
    #[error("Unexpected error in communication protocol {err}")]
    CommunicationError { err: String },
}
