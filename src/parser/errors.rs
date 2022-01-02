use miette::{Diagnostic, NamedSource, SourceSpan};

#[derive(Error, Diagnostic, Debug)]
#[diagnostic(help("Error during parse parol grammar"), code(parol::parser))]
pub enum ParolParserError {
    #[error("{context} - Unknown scanner {name}")]
    #[diagnostic(
        help("Undeclared scanner found. Pease declare a scanner via %scanner name {{...}}"),
        code(parol::parser::unknown_scanner)
    )]
    UnknownScanner {
        context: String,
        name: String,
        #[source_code]
        input: NamedSource,
        #[label("Undeclared")]
        token: SourceSpan,
    },
}
