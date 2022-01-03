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

    #[error("{context} - Empty Group not allowed")]
    #[diagnostic(
        help("Empty Groups () are not allowed."),
        code(parol::parser::empty_group)
    )]
    EmptyGroup {
        context: String,
        #[source_code]
        input: NamedSource,
        #[label("Start")]
        start: SourceSpan,
        #[label("End")]
        end: SourceSpan,
    },

    #[error("{context} - Empty Optionals not allowed")]
    #[diagnostic(
        help("Empty Optionals [] are not allowed."),
        code(parol::parser::empty_optional)
    )]
    EmptyOptional {
        context: String,
        #[source_code]
        input: NamedSource,
        #[label("Start")]
        start: SourceSpan,
        #[label("End")]
        end: SourceSpan,
    },

    #[error("{context} - Empty Repetitions not allowed")]
    #[diagnostic(
        help("Empty Repetitions {{}} are not allowed."),
        code(parol::parser::empty_repetition)
    )]
    EmptyRepetition {
        context: String,
        #[source_code]
        input: NamedSource,
        #[label("Start")]
        start: SourceSpan,
        #[label("End")]
        end: SourceSpan,
    },
}
