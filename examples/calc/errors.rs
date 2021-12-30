use miette::{NamedSource, SourceSpan};

#[derive(Error, Diagnostic, Debug)]
#[diagnostic(help("Error during parse"), code(parol::examples::calc))]
pub enum CalcError {
    #[error("isize parse error")]
    #[diagnostic(
        help("Error parsing number token as valid isize"),
        code(parol::examples::calc::parse_isize)
    )]
    ParseISizeFailed {
        #[source_code]
        input: NamedSource,
        #[label("Wrong isize value")]
        token: SourceSpan,
    },
}
