use crate::analysis::k_tuple::TerminalMappings;
use crate::grammar::cfg::TerminalIndexFn;
use crate::{Symbol, Terminal};
use parol_runtime::TerminalIndex;
use parol_runtime::lexer::EOI;
use std::fmt::{Debug, Display, Error, Formatter};

///
/// Epsilon token constant
/// Can be contained in FIRST sets
///
pub const EPS: TerminalIndex = TerminalIndex::MAX;

///
/// Invalid token, used as placeholder and initial value in Default
pub(crate) const INVALID: TerminalIndex = TerminalIndex::MAX - 1;

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
///
/// Internal data structure to represent a compiled terminal, a TerminalIndex.
///
#[derive(Default, Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct CompiledTerminal(pub TerminalIndex);

impl CompiledTerminal {
    /// Creates a new item from a Symbol
    pub fn create<F, R>(s: &Symbol, terminal_index_resolver: R) -> Self
    where
        R: AsRef<F>,
        F: TerminalIndexFn,
    {
        match s {
            Symbol::T(Terminal::Trm(t, k, _, _, _, _, l)) => {
                Self(terminal_index_resolver.as_ref().terminal_index(t, *k, l))
            }
            Symbol::T(Terminal::End) => Self(EOI),
            _ => panic!("Unexpected symbol type: {s:?}"),
        }
    }
}

impl AsRef<TerminalIndex> for CompiledTerminal {
    fn as_ref(&self) -> &TerminalIndex {
        &self.0
    }
}

impl Display for CompiledTerminal {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.0)
    }
}

impl From<TerminalIndex> for CompiledTerminal {
    fn from(t: TerminalIndex) -> Self {
        Self(t)
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
