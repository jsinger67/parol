use thiserror::Error;

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------

/// Error type used by the [crate::analysis] module
#[derive(Error, Debug)]
pub enum GrammarAnalysisError {
    /// Left-recursions detected
    #[error("Grammar contains left-recursions")]
    LeftRecursion {
        /// Recursions
        recursions: Vec<RecursiveNonTerminal>,
    },

    /// Right-recursions detected
    #[error("Grammar contains right-recursions")]
    RightRecursion {
        /// Recursions
        recursions: Vec<RecursiveNonTerminal>,
    },

    /// Unreachable non-terminals are not allowed.
    #[error("Grammar contains unreachable non-terminals")]
    UnreachableNonTerminals {
        /// Non-terminals
        non_terminals: Vec<RelatedHint>,
    },

    /// Nonproductive non-terminals are not allowed.
    #[error("Grammar contains nonproductive non-terminals")]
    NonProductiveNonTerminals {
        /// Non-terminals
        non_terminals: Vec<RelatedHint>,
    },

    /// Maximum lookahead exceeded.
    #[error("Maximum lookahead of {max_k} exceeded")]
    MaxKExceeded {
        /// Maximum lookahead
        max_k: usize,
    },
}

/// A single recursive non-terminal
#[derive(Error, Debug)]
#[error("Recursive non-terminal #{number}: '{name}'")]
pub struct RecursiveNonTerminal {
    /// The number of the recursion path
    pub number: usize,
    /// non-terminal
    pub name: String,
}

/// Related information
#[derive(Error, Debug)]
#[error("{topic}: {hint}")]
pub struct RelatedHint {
    /// A topic or a category to describe the hint
    pub topic: String,
    /// Information
    pub hint: String,
}
