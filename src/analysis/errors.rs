use miette::Diagnostic;

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------

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
        recursions: Vec<RecursiveNonTerminal>,
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
    #[diagnostic(
        help("Please examine your grammar"),
        code(parol::analysis::max_k_exceeded)
    )]
    MaxKExceeded {
        /// Maximum lookahead
        max_k: usize,
    },
}

/// A single recursive non-terminal
#[derive(Error, Diagnostic, Debug)]
#[error("Recursive non-terminal #{number}: '{name}'")]
pub struct RecursiveNonTerminal {
    /// The number of the recursion path
    pub number: usize,
    /// non-terminal
    pub name: String,
}

/// Related information
#[derive(Error, Diagnostic, Debug)]
#[error("{topic}: {hint}")]
pub struct RelatedHint {
    /// A topic or a category to describe the hint
    pub topic: String,
    /// Information
    pub hint: String,
}
