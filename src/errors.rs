use crate::lexer::token_stream::TokenStream;
use crate::lexer::Token;
use miette::{
    Diagnostic, IntoDiagnostic, MietteError, NamedSource, Result, SourceCode, SourceSpan,
    SpanContents,
};
use std::borrow::Borrow;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Error, Diagnostic, Debug)]
pub enum ParserError {
    #[error(transparent)]
    #[diagnostic(
        help("Error from id_tree crate"),
        code(parol_runtime::parser::id_tree_error)
    )]
    IdTreeError {
        #[from]
        source: id_tree::NodeIdError,
    },

    #[error("{cause}Expecting one of {expected_tokens}")]
    #[diagnostic(
        help("Syntax error in input prevents prediction of next production"),
        code(parol_runtime::parser::syntax_error)
    )]
    PredictionErrorWithExpectations {
        cause: String,
        #[source_code]
        input: NamedSource,
        #[related("Unexpected tokens")]
        unexpected_tokens: Vec<UnexpectedToken>,
        expected_tokens: TokenVec,
    },

    #[error("Unprocessed input is left after parsing has finished")]
    #[diagnostic(
        help("Unprocessed input is left after parsing has finished"),
        code(parol_runtime::parser::unprocessed_input)
    )]
    UnprocessedInput {
        #[source_code]
        input: NamedSource,
        #[label("Last processed token")]
        last_token: SourceSpan,
    },

    #[error("{0}")]
    #[diagnostic(
        help("Unexpected internal state"),
        code(parol_runtime::parser::internal_error)
    )]
    InternalError(String),
}

#[derive(Error, Diagnostic, Debug)]
pub enum LookaheadError {
    #[error("{0}")]
    #[diagnostic(
        help("Error in generated source"),
        code(parol_runtime::lookahead::generation_error)
    )]
    DataError(&'static str),

    #[error("{cause}")]
    #[diagnostic(
        help("Error in input"),
        code(parol_runtime::lookahead::production_prediction_error)
    )]
    PredictionError { cause: String },

    #[error("No valid token read")]
    #[diagnostic(
        help("No valid token read"),
        code(parol_runtime::lookahead::empty_token_buffer)
    )]
    TokenBufferEmptyError,
}

#[derive(Error, Diagnostic, Debug)]
#[error("Unexpected token: {name} ({token_type})")]
#[diagnostic(help("Unexpected token"), code(parol_runtime::unexpected_token))]
pub struct UnexpectedToken {
    name: String,
    token_type: String,
    #[source_code]
    input: NamedSource,
    #[label("Unexpected token")]
    token: SourceSpan,
}

impl UnexpectedToken {
    pub fn new(name: String, token_type: String, input: NamedSource, token: &Token<'_>) -> Self {
        let token = token.into();
        Self {
            name,
            token_type,
            input,
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
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
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

#[derive(Debug, Default)]
pub struct FileSource {
    file_name: Arc<PathBuf>,
    input: String,
}

impl FileSource {
    pub fn try_new(file_name: Arc<PathBuf>) -> Result<Self> {
        let input = std::fs::read_to_string(<Arc<PathBuf> as Borrow<PathBuf>>::borrow(&file_name))
            .into_diagnostic()?;
        Ok(Self { file_name, input })
    }

    pub fn from_stream(token_stream: &TokenStream<'_>) -> Self {
        let file_name = token_stream.file_name.clone();
        let input = token_stream.input.to_string();
        Self { file_name, input }
    }
}

impl SourceCode for FileSource {
    fn read_span<'a>(
        &'a self,
        span: &SourceSpan,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> Result<Box<dyn SpanContents<'a> + 'a>, MietteError> {
        <str as SourceCode>::read_span(&self.input, span, context_lines_before, context_lines_after)
    }
}

impl From<FileSource> for NamedSource {
    fn from(file_source: FileSource) -> Self {
        let file_name = file_source.file_name.clone();
        let file_name = file_name.to_str().unwrap_or("<Bad file name>");
        Self::new(file_name, file_source)
    }
}
