/// Module with the compiled lookahead DFA
pub mod compiled_la_dfa;

/// Module with the compiled terminal
pub mod compiled_terminal;
pub use compiled_terminal::CompiledTerminal;

///
/// Error types used by this module
///
#[forbid(missing_docs)]
pub mod errors;
pub use errors::GrammarAnalysisError;

/// Module with check for left-recursions
pub mod left_recursion;
pub use left_recursion::detect_left_recursive_non_terminals;

/// Module with the KTuple type
pub mod k_tuple;
pub use k_tuple::KTuple;

/// Module with the KTuples type
pub mod k_tuples;
pub use k_tuples::KTuples;

/// Module with FIRST set calculations
pub mod first;
pub use first::{first_k, FirstSet};

/// Module with FOLLOW set calculations
pub mod follow;
pub use follow::{follow_k, FollowSet};

/// Module with conflict calculations
pub mod k_decision;
pub use k_decision::{
    calculate_k, calculate_k_tuples, calculate_lookahead_dfas, decidable, explain_conflicts,
    FirstCache, FollowCache,
};

/// Module with types for production selection
pub mod lookahead_dfa;
pub use lookahead_dfa::LookaheadDFA;

/// Module with productivity calculations
pub mod productivity;
pub use productivity::non_productive_non_terminals;

/// Module with reachability calculations
pub mod reachability;
pub use reachability::{
    all_non_terminals_reachable, nt_producing_productions, nt_reachability,
    reachable_from_non_terminal, reachable_from_production, reachable_non_terminals,
    unreachable_non_terminals,
};
