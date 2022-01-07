use crate::analysis::compiled_la_dfa::TerminalIndex;
use crate::analysis::k_tuple::TerminalMappings;
use crate::{Symbol, Terminal};
use parol_runtime::lexer::EOI;
use std::fmt::{Debug, Display, Error, Formatter};

///
/// Epsilon token constant
/// Can be contained in FIRST sets
///
pub(crate) const EPS: TerminalIndex = TerminalIndex::MAX;

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
///
/// Internal data structure to represent a compiled terminal, a TerminalIndex.
///
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct CompiledTerminal(pub TerminalIndex);

impl CompiledTerminal {
    /// Creates a new item from a Symbol
    pub fn create<R>(s: &Symbol, terminal_index_resolver: R) -> Self
    where
        R: Fn(&str) -> TerminalIndex,
    {
        match s {
            Symbol::T(Terminal::Trm(t, _)) => Self(terminal_index_resolver(t)),
            Symbol::T(Terminal::End) => Self(EOI),
            _ => panic!("Unexpected symbol type: {:?}", s),
        }
    }
}

impl Display for CompiledTerminal {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.0)
    }
}

impl TerminalMappings<CompiledTerminal> for CompiledTerminal {
    fn eps() -> CompiledTerminal {
        Self(EPS)
    }

    fn end() -> CompiledTerminal {
        Self(EOI)
    }

    fn is_eps(&self) -> bool {
        self.0 == EPS
    }

    fn is_end(&self) -> bool {
        self.0 == EOI
    }
}
