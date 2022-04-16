use miette::{NamedSource, SourceSpan};

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
}
