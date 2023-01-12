use crate::lexer::token_stream::TokenStream;
use crate::lexer::{Location, Token};
use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::path::Path;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, ParolError>;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error(transparent)]
    IdTreeError { source: id_tree::NodeIdError },

    #[error("{cause}Expecting one of {expected_tokens}")]
    PredictionErrorWithExpectations {
        cause: String,
        input: FileSource,
        error_location: Location,
        unexpected_tokens: Vec<UnexpectedToken>,
        expected_tokens: TokenVec,
        source: Option<Box<ParolError>>,
    },

    #[error("Unprocessed input is left after parsing has finished")]
    UnprocessedInput {
        input: FileSource,
        last_token: Location,
    },

    #[error("{context}Tried to pop from an empty scanner stack")]
    PopOnEmptyScannerStateStack {
        context: String,
        input: FileSource,
        source: LexerError,
    },

    #[error("{0}")]
    InternalError(String),
}

#[derive(Error, Debug)]
pub enum LexerError {
    #[error("Error in generated source: {0}")]
    DataError(&'static str),

    #[error("Error in input: {cause}")]
    PredictionError { cause: String },

    #[error("No valid token read")]
    TokenBufferEmptyError,

    #[error("{0}")]
    InternalError(String),

    #[error("Lookahead exceeds its maximum")]
    LookaheadExceedsMaximum,

    #[error("Lookahead exceeds token buffer length")]
    LookaheadExceedsTokenBufferLength,

    #[error("pop_scanner: Tried to pop from an empty scanner stack!")]
    ScannerStackEmptyError,
}

#[derive(Error, Debug)]
pub enum ParolError {
    #[error(transparent)]
    ParserError(#[from] ParserError),
    #[error(transparent)]
    LexerError(#[from] LexerError),
    #[error(transparent)]
    UserError(#[from] anyhow::Error),
}

#[derive(Debug)]
pub struct UnexpectedToken {
    pub name: String,
    pub token_type: String,
    pub token: Location,
}

impl UnexpectedToken {
    pub fn new(name: String, token_type: String, token: &Token<'_>) -> Self {
        let token = token.into();
        Self {
            name,
            token_type,
            token,
        }
    }
}

#[derive(Debug, Default)]
pub struct TokenVec(Vec<String>);

impl TokenVec {
    pub fn push(&mut self, token: String) {
        self.0.push(token);
    }
}

impl Display for TokenVec {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(
            f,
            "{}",
            self.0.iter().fold(String::new(), |mut acc, e| {
                if !acc.is_empty() {
                    acc.push_str(", ");
                }
                acc.push_str(e.to_string().as_str());
                acc
            })
        )
    }
}

#[derive(Debug)]
pub struct FileSource {
    pub file_name: Cow<'static, Path>,
    pub input: String,
}

impl FileSource {
    pub fn try_new<T>(file_name: T) -> std::result::Result<Self, std::io::Error>
    where
        T: Into<Cow<'static, Path>>,
    {
        let file_name: Cow<Path> = file_name.into();
        let input = std::fs::read_to_string(&*file_name)?;
        Ok(Self { file_name, input })
    }

    pub fn from_stream(token_stream: &TokenStream<'_>) -> Self {
        let file_name = token_stream.file_name.clone();
        let input = token_stream.input.to_string();
        Self { file_name, input }
    }
}
