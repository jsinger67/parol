use crate::analysis::compiled_la_dfa::TerminalIndex;
use crate::analysis::k_tuple::TerminalMappings;
use crate::{Symbol, Terminal, TerminalKind};
use parol_runtime::lexer::EOI;
use std::fmt::{Debug, Display, Error, Formatter};

use super::terminals_trie::MAX_TERMINAL_COUNT;

///
/// Epsilon token constant
/// Can be contained in FIRST sets
///
pub const EPS: TerminalIndex = MAX_TERMINAL_COUNT;

///
/// Invalid token, used as placeholder and initial value in Default
pub(crate) const INVALID: TerminalIndex = MAX_TERMINAL_COUNT - 1;

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
    pub fn create<F, R>(s: &Symbol, terminal_index_resolver: R) -> Self
    where
        R: AsRef<F>,
        F: Fn(&str, TerminalKind) -> TerminalIndex,
    {
        match s {
            Symbol::T(Terminal::Trm(t, k, ..)) => Self(terminal_index_resolver.as_ref()(t, *k)),
            Symbol::T(Terminal::End) => Self(EOI),
            _ => panic!("Unexpected symbol type: {:?}", s),
        }
    }
}

impl Default for CompiledTerminal {
    fn default() -> Self {
        Self(INVALID)
    }
}

impl Display for CompiledTerminal {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.0)
    }
}

impl TerminalMappings<CompiledTerminal> for CompiledTerminal {
    #[inline]
    fn eps() -> CompiledTerminal {
        Self(EPS)
    }

    #[inline]
    fn end() -> CompiledTerminal {
        Self(EOI)
    }

    #[inline]
    fn is_eps(&self) -> bool {
        self.0 == EPS
    }

    #[inline]
    fn is_end(&self) -> bool {
        self.0 == EOI
    }

    #[inline]
    fn is_inv(&self) -> bool {
        self.0 == INVALID
    }
}
