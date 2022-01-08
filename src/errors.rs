use miette::{NamedSource, SourceSpan};

#[derive(Error, Diagnostic, Debug)]
#[diagnostic(help("Error during parse"), code(json_parser::error))]
pub enum JsonError {
    #[error("f64 parse error")]
    #[diagnostic(
        help("Error parsing number token as valid f64"),
        code(json_parser::error::parse_isize)
    )]
    ParseF64Failed {
        #[source_code]
        input: NamedSource,
        #[label("Wrong f64 value")]
        token: SourceSpan,
    },
}
