use miette::{NamedSource, SourceSpan};

#[derive(Error, Diagnostic, Debug)]
#[diagnostic(help("Error during parse"), code(parol::examples::calc))]
pub enum CalcError {
    #[error("{context} isize parse error")]
    #[diagnostic(
        help("Error parsing number token as valid isize"),
        code(parol::examples::calc::parse_isize)
    )]
    ParseISizeFailed {
        context: String,
        #[source_code]
        input: NamedSource,
        #[label("Wrong isize value")]
        token: SourceSpan,
    },

    #[error("{context} Undeclared variable")]
    #[diagnostic(
        help("Referencing a variable that was not assigned to yet"),
        code(parol::examples::calc::undeclared_variable)
    )]
    UndeclaredVariable {
        context: String,
        #[source_code]
        input: NamedSource,
        #[label("Unknown variable name")]
        token: SourceSpan,
    },
}
