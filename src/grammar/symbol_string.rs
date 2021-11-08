use crate::{Pr, Symbol, Terminal};
use std::cell::RefCell;
use std::fmt::{Display, Error, Formatter};

///
/// Type of the RHS of a Production type
///
#[derive(Debug, Clone, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct SymbolString(pub Vec<Symbol>);

///
/// Internal state type that controls the search of k-complete substrings.
///
#[derive(Debug, Clone)]
enum State {
    Initial,
    NtFound(usize, usize),
    KComplete(usize, usize),
}

impl SymbolString {
    ///
    /// Construction from a given production
    ///
    pub fn from_production(pr: &Pr) -> Self {
        Self(pr.get_r().clone())
    }

    ///
    /// Construction from a given symbol string
    ///
    pub fn from_slice(symbols: &[Symbol]) -> Self {
        Self(symbols.to_vec())
    }

    ///
    /// Find a k-complete sub-symbol-string within this symbol-string.
    ///
    /// k-complete means here that for a given non-terminal nt there exists a
    /// sequence of terminals with length k <<OR>> a sequence of terminals with
    /// 0 <= length < k followed by an END symbol that is directly following it.
    ///
    pub fn is_k_complete_for_nt_at(
        &self,
        nt: &str,
        k: usize,
        start: usize,
    ) -> Option<(usize, usize)> {
        let state = RefCell::new(State::Initial);
        self.0.iter().enumerate().skip(start).for_each(|(idx, sy)| {
            let old_state = state.borrow().clone();
            let mut new_state = state.borrow_mut();
            match (old_state, sy) {
                (State::Initial, Symbol::N(n)) if n == nt => *new_state = State::NtFound(idx, 0),
                (State::Initial, _) => (),
                (State::NtFound(nt_idx, len), Symbol::T(Terminal::Trm(_, _))) if len + 1 < k => {
                    *new_state = State::NtFound(nt_idx, len + 1)
                }
                (State::NtFound(nt_idx, len), Symbol::T(Terminal::Trm(_, _))) => {
                    *new_state = State::KComplete(nt_idx, len + 1)
                }
                (State::NtFound(nt_idx, len), Symbol::T(Terminal::End)) => {
                    *new_state = State::KComplete(nt_idx, len + 1)
                }
                (State::NtFound(_, _), Symbol::N(n)) if n == nt => {
                    *new_state = State::NtFound(idx, 0)
                }
                (State::NtFound(_, _), Symbol::N(_)) => *new_state = State::Initial,
                (State::NtFound(_, _), Symbol::T(Terminal::Eps)) => {
                    panic!("Not allowed symbol type for symbol string: Epsilon")
                }
                (State::KComplete(_, _), _) => (),
            }
        });
        if let State::KComplete(start_idx, len) = state.into_inner() {
            Some((start_idx, len))
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn first_len(&self) -> usize {
        self.0
            .iter()
            .take_while(|s| {
                matches!(s, Symbol::T(Terminal::Trm(_, _))) || matches!(s, Symbol::T(Terminal::End))
            })
            .count()
    }

    pub fn is_k_derivable(&self, k: usize) -> bool {
        // Necessary
        self.first_len() < k &&
        // Possible
        self.0.iter().any(|s| matches!(s, Symbol::N(_)))
    }

    pub fn static_first_len(symbols: &[Symbol]) -> usize {
        symbols
            .iter()
            .take_while(|s| {
                matches!(s, Symbol::T(Terminal::Trm(_, _))) || matches!(s, Symbol::T(Terminal::End))
            })
            .count()
    }
}

impl Display for SymbolString {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|e| format!("{}", e))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

#[cfg(test)]
mod test {
    use crate::grammar::SymbolString;
    use crate::Symbol;

    fn t(t: &str) -> Symbol {
        Symbol::t(t, 0)
    }

    fn n(n: &str) -> Symbol {
        Symbol::n(n)
    }

    fn e() -> Symbol {
        Symbol::e()
    }

    lazy_static! {
        pub static ref TEST_DATA: Vec<(
            SymbolString,
            (&'static str, usize, usize),
            Option<(usize, usize)>
        )> = vec![
            (SymbolString(vec![]), ("B", 1, 0), None),
            (SymbolString(vec![t("a"), n("A")]), ("B", 1, 0), None),
            (
                SymbolString(vec![t("a"), n("B"), n("A")]),
                ("B", 1, 0),
                None
            ),
            (
                SymbolString(vec![t("a"), n("B"), t("a")]),
                ("B", 1, 0),
                Some((1, 1))
            ),
            (
                SymbolString(vec![t("a"), n("B"), t("a"), t("a")]),
                ("B", 1, 0),
                Some((1, 1))
            ),
            (
                SymbolString(vec![t("a"), n("B"), t("a"), t("a")]),
                ("B", 2, 0),
                Some((1, 2))
            ),
            (
                SymbolString(vec![t("a"), n("B"), t("a"), t("a"), t("a")]),
                ("B", 2, 0),
                Some((1, 2))
            ),
            (
                SymbolString(vec![t("a"), n("B"), t("a"), t("a"), e()]),
                ("B", 2, 0),
                Some((1, 2))
            ),
            (
                SymbolString(vec![t("a"), n("B"), t("a"), e()]),
                ("B", 2, 0),
                Some((1, 2))
            ),
            (
                SymbolString(vec![t("a"), n("B"), e()]),
                ("B", 2, 0),
                Some((1, 1))
            ),
            (
                SymbolString(vec![t("a"), n("B"), e()]),
                ("B", 2, 1),
                Some((1, 1))
            ),
            (
                SymbolString(vec![t("a"), n("B"), e()]),
                ("B", 1, 0),
                Some((1, 1))
            ),
        ];
    }

    #[test]
    fn test_is_k_complete_for_nt_at() {
        TEST_DATA
            .iter()
            .for_each(|(symbol_string, (nt, k, start), expected)| {
                assert_eq!(
                    expected,
                    &symbol_string.is_k_complete_for_nt_at(nt, *k, *start)
                )
            });
    }
}
