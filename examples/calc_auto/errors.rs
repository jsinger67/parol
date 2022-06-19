use miette::{NamedSource, SourceSpan};

#[derive(Error, Diagnostic, Debug)]
pub enum CalcError {
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
