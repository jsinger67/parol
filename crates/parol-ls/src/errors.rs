use std::path::PathBuf;

use lsp_types::Range;
use miette::Diagnostic;

///
/// Error types used by the language server
///
#[derive(Error, Diagnostic, Debug)]
pub enum ServerError {
    /// Unknown document referred during a request
    #[error("Unknown document {path}")]
    #[diagnostic(
        help("Unknown document referred during a request. Protocol error"),
        code(parol_ls::server::unknown_document)
    )]
    UnknownDocument {
        /// Path of the unknown document
        path: PathBuf,
    },
    /// Left recursions in input grammar
    #[error("Left recursions in input grammar")]
    #[diagnostic(
        help("Left recursions detected in input grammar."),
        code(parol_ls::server::left_recursions)
    )]
    LeftRecursions {
        /// Recursion paths
        paths: Vec<Vec<Range>>,
    },
    /// Unexpected error in communication protocol
    #[error("Unexpected error in communication protocol {err}")]
    #[diagnostic(help("Protocol error"), code(parol_ls::server::protocol_error))]
    ProtocolError { err: Box<dyn std::error::Error> },
    /// Unexpected error in communication protocol
    #[error("Unexpected error in communication protocol {err}")]
    #[diagnostic(help("Protocol error"), code(parol_ls::server::communication_error))]
    CommunicationError { err: String },
}
