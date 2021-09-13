///
/// Index of a DFA state within the slice of states of a LookaheadDFA
///
pub type StateIndex = usize;

///
/// Index of a production within the slice of productions of a generated parser
///
pub type ProductionIndex = usize;

///
/// Index of a non-terminal within the slice of lookahead automatons of a
/// generated parser. Also used to index into the slice of non-terminal names
/// in the generated parser.
///
pub type NonTerminalIndex = usize;

///
/// Module with types used to handle the parse tree (abstract syntax tree) that
/// is build during runs of the generated parsers.
///
#[forbid(missing_docs)]
pub mod ast;
pub use ast::AstType;

///
/// Module with types used in the generated parser's ast stack.
///
#[forbid(missing_docs)]
pub mod ast_stack_entry;
pub use ast_stack_entry::AstStackEntry;

///
/// Module with types used predict the next productions to choose during runs of
/// generated parsers.
///
#[forbid(missing_docs)]
pub mod lookahead_dfa;
pub use lookahead_dfa::{DFAState, DFATransition, LookaheadDFA};

///
/// Module with types used in the parser stack.
///
#[forbid(missing_docs)]
pub mod parse_type;
pub use parse_type::{ParseStack, ParseType};

///
/// Module with the actual parser types and some supporting types.
///
#[forbid(missing_docs)]
pub mod parser_types;
pub use parser_types::{LLKParser, Production};

///
/// Module with the UserActionsTrait type.
///
#[forbid(missing_docs)]
pub mod user_actions;
pub use user_actions::UserActionsTrait;

///
/// error_chains error module that auto-creates basic error types.
///
pub mod errors;
