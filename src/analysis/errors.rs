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
        recursions: Vec<RecursionPath>,
    },

    /// Unreachable non-terminals are not allowed.
    #[error("Grammar contains unreachable non-terminals")]
    #[diagnostic(
        help("If not used they can safely be removed"),
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
        help("If not used they can safely be removed"),
        code(parol::analysis::nonproductive_non_terminals)
    )]
    NonProductiveNonTerminals {
        /// Non-terminals
        #[related]
        non_terminals: Vec<RelatedHint>,
    },

    /// Maximum lookahead exceeded.
    #[error("Maximum lookahead of {max_k} exceeded")]
    #[diagnostic(help("Please examine your grammar"), code(parol::analysis::max_k_exceeded))]
    MaxKExceeded {
        /// Maximum lookahead
        max_k: usize,
    },
}

/// A single recursion
#[derive(Error, Diagnostic, Debug)]
#[error("Recursion {number}")]
pub struct RecursionPath {
    /// The number of the recursion path
    pub number: usize,
    /// Recursion path elements
    #[related]
    pub hints: Vec<RelatedHint>,
}

/// Related information
#[derive(Error, Diagnostic, Debug)]
#[error("Hint: {hint}")]
pub struct RelatedHint {
    /// A topic or a category to describe the hint
    pub topic: String,
    /// Information
    pub hint: String,
}
