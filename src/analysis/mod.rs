///
/// Module with the compiled lookahead DFA
///
pub mod compiled_la_dfa;

pub mod compiled_terminal;
pub use compiled_terminal::CompiledTerminal;

pub mod left_recursion;
pub use left_recursion::detect_left_recursions;

pub mod k_tuple;
pub use k_tuple::KTuple;

pub mod k_tuples;
pub use k_tuples::KTuples;

pub mod first;
pub use first::{first_k, FirstSet};

pub mod follow;
pub use follow::{follow_k, FollowSet};

pub mod k_decision;
pub use k_decision::{
    calculate_k, calculate_k_tuples, calculate_lookahead_dfas, decidable, FirstCache, FollowCache,
};

pub mod lookahead_dfa;
pub use lookahead_dfa::LookaheadDFA;

pub mod nt_grammar_graph;
pub use nt_grammar_graph::{NtEdgeType, NtGrammarGraph, NtNodeType};

pub mod productivity;
pub use productivity::non_productive_non_terminals;

pub mod reachability;
pub use reachability::{
    all_non_terminals_reachable, nt_producing_productions, nt_reachability,
    reachable_from_non_terminal, reachable_from_production, reachable_non_terminals,
    unreachable_non_terminals,
};
