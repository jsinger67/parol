///
/// Index of a DFA state within the slice of states of a LookaheadDFA
///
pub type StateIndex = usize;

///
/// Index of a production within the slice of productions of a generated parser
///
pub type ProductionIndex = usize;

///
/// Index of a production within the slice of productions of a generated parser
///
pub type CompiledProductionIndex = i32;

/// Invalid production number
/// It usually denotes the absence of a valid production number after applying a transition
pub const INVALID_PROD: CompiledProductionIndex = -1;

///
/// Index of a non-terminal within the slice of lookahead automatons of a
/// generated parser. Also used to index into the slice of non-terminal names
/// in the generated parser.
///
pub type NonTerminalIndex = usize;

///
/// The index of a scanner configuration
///
pub type ScannerIndex = usize;

///
/// Module with types used to handle the parse tree that is build during runs of
/// the generated parsers.
///
#[forbid(missing_docs)]
pub mod parse_tree_type;
pub use parse_tree_type::ParseTreeType;

///
/// Module with types used to predict the next productions to choose during runs
/// of generated parsers.
///
#[forbid(missing_docs)]
pub mod lookahead_dfa;
pub use lookahead_dfa::{LookaheadDFA, Trans};

///
/// Module with types used in the parse stack.
///
#[forbid(missing_docs)]
pub mod parse_type;
pub use parse_type::{ParseStack, ParseType};

///
/// Module with the actual parser types and some supporting types.
///
#[forbid(missing_docs)]
pub mod parser_types;
pub use parser_types::{LLKParser, ParseTree, Production};

///
/// Module with the UserActionsTrait type.
///
#[forbid(missing_docs)]
pub mod user_access;
pub use user_access::UserActionsTrait;

///
/// Module with recovery algorithms
///
pub(crate) mod recovery;
// pub(crate) use recovery::Recovery;
