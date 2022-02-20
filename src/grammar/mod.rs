///
/// Module with attributes used at certain items of a grammar.
///
pub mod attributes;
pub use attributes::{Decorate, ProductionAttribute, SymbolAttribute};

/// Module with the context-free grammar types
pub mod cfg;
pub use cfg::Cfg;

/// Module with a grammar position type
pub mod pos;
pub use pos::Pos;

/// Module with types related to grammar productions
pub mod production;
pub use production::{Pr, Rhs};

pub(crate) mod symbol_string;

/// Module with symbol types
pub mod symbol;
pub use symbol::{Symbol, Terminal};
