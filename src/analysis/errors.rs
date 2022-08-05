use miette::Diagnostic;

/// Error type used by the [crate::analysis] module
#[derive(Error, Diagnostic, Debug)]
pub enum GrammarAnalysisError {
    /// Left-recursions detected
    #[error("Grammar contains left-recursions")]
    #[diagnostic(
        help("Left-recursions detected. Please rework your grammar to remove these recursions"),
        code(parol::analysis::left_recursion)
    )]
    LeftRecursion {
        /// Recursions
        #[related]
        recursions: Vec<Recursion>,
    },

    /// Unreachable non-terminals are not allowed.
    #[error("Grammar contains unreachable non-terminals")]
    #[diagnostic(
        help("If not used they can safely removed"),
        code(parol::analysis::unreachable_non_terminals)
    )]
    UnreachableNonTerminals {
        /// Non-terminals
        #[related]
        non_terminals: Vec<RelatedHint>,
    },

    /// Nonproductive non-terminals are not allowed.
    #[error("Grammar contains nonproductive non-terminals")]
    #[diagnostic(
        help("If not used they can safely removed"),
        code(parol::analysis::nonproductive_non_terminals)
    )]
    NonProductiveNonTerminals {
        /// Non-terminals
        #[related]
        non_terminals: Vec<RelatedHint>,
    },
}

/// A single recursion
#[derive(Error, Diagnostic, Debug)]
#[error("Recursion")]
pub struct Recursion {
    /// Hints
    #[related]
    pub hints: Vec<RelatedHint>,
}

/// Related information
#[derive(Error, Diagnostic, Debug)]
#[error("Hint: {hint}")]
pub struct RelatedHint {
    /// Information
    pub hint: String,
}
