use crate::analysis::compiled_la_dfa::TerminalIndex;
use crate::analysis::compiled_terminal::EPS;
use crate::{CompiledTerminal, MAX_K};
use std::fmt::{Debug, Display, Error, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Index;

const EOI: TerminalIndex = 0;
const NEW_LINE: TerminalIndex = 1;
const WHITESPACE: TerminalIndex = 2;
const LINE_COMMENT: TerminalIndex = 3;
const BLOCK_COMMENT: TerminalIndex = 4;

/// Common functions needed for terminal handling
pub trait TerminalMappings<T> {
    /// Create an epsilon representation
    fn eps() -> T;
    /// Create an end-of-input representation
    fn end() -> T;
    /// Check for epsilon
    fn is_eps(&self) -> bool;
    /// Check for end-of-input
    fn is_end(&self) -> bool;
    /// Check for invalid (i.e. unassigned) terminal
    fn is_inv(&self) -> bool;
}

/// An ordered collection of terminals
#[derive(Clone, Copy, Default, Hash, Eq, PartialEq)]
pub struct Terminals {
    // The terminals
    pub(crate) t: [CompiledTerminal; MAX_K],
    // The index of next insertion
    pub(crate) i: usize,
}

impl Terminals {
    /// Creates a new item
    /// ```
    /// use parol::analysis::k_tuple::Terminals;
    /// use parol::analysis::compiled_terminal::CompiledTerminal;
    /// let t = Terminals::new();
    /// assert!(t.is_empty());
    /// assert_eq!(0, t.len(), "len");
    /// assert_eq!(0, t.k_len(5), "k_len");
    /// assert_eq!(CompiledTerminal::default(), t[0]);
    /// assert_eq!(CompiledTerminal::default(), t[9]);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new item with epsilon semantic
    /// ```
    /// use parol::analysis::k_tuple::{Terminals, TerminalMappings};
    /// use parol::analysis::compiled_terminal::CompiledTerminal;
    /// let t = Terminals::eps();
    /// assert!(!t.is_empty());
    /// assert_eq!(1, t.len(), "len");
    /// assert_eq!(1, t.k_len(5), "k_len");
    /// assert_eq!(CompiledTerminal::eps(), t[0]);
    /// assert_eq!(CompiledTerminal::default(), t[1]);
    /// assert_eq!(CompiledTerminal::default(), t[9]);
    /// ```
    pub fn eps() -> Terminals {
        let mut t = <[CompiledTerminal; MAX_K]>::default();
        t[0] = CompiledTerminal::eps();
        Self { t, i: 1 }
    }

    /// Creates a new item with end (EOI) semantic
    /// Such a terminal can't be extended, i.e. you can't append more terminals
    /// ```
    /// use parol::analysis::k_tuple::{Terminals, TerminalMappings};
    /// use parol::analysis::compiled_terminal::CompiledTerminal;
    /// let t = Terminals::end();
    /// assert!(!t.is_empty());
    /// assert_eq!(1, t.len());
    /// assert_eq!(1, t.k_len(5));
    /// assert_eq!(CompiledTerminal::end(), t[0]);
    /// assert_eq!(CompiledTerminal::default(), t[1]);
    /// assert_eq!(CompiledTerminal::default(), t[9]);
    /// ```
    pub fn end() -> Terminals {
        let mut t = <[CompiledTerminal; MAX_K]>::default();
        t[0] = CompiledTerminal::end();
        Self { t, i: 1 }
    }

    ///
    /// Creates a new object with maximum k length from another object
    ///
    #[must_use]
    pub fn of(k: usize, other: Self) -> Self {
        let first_len = other.k_len(k);
        let mut t = <[CompiledTerminal; MAX_K]>::default();
        (0..first_len).for_each(|i| t[i] = other.t[i]);
        Self { t, i: first_len }
    }

    ///
    /// Creates a new object from a slice of other objects while applying a mapper function
    ///
    #[must_use]
    pub fn from_slice_with<'s, S, M>(others: &'s [S], k: usize, m: M) -> Self
    where
        M: Fn(&'s S) -> CompiledTerminal,
    {
        let mut t = <[CompiledTerminal; MAX_K]>::default();
        let mut i = 0;
        others
            .iter()
            .take(std::cmp::min(k, MAX_K))
            .enumerate()
            .for_each(|(n, o)| {
                t[n] = m(o);
                i += 1;
            });
        Self { t, i }
    }

    ///
    /// Creates a new object from a slice of other objects
    ///
    #[must_use]
    pub fn from_slice(others: &[CompiledTerminal], k: usize) -> Self {
        let mut t = <[CompiledTerminal; MAX_K]>::default();
        let mut i = 0;
        others
            .iter()
            .take(std::cmp::min(k, MAX_K))
            .enumerate()
            .for_each(|(n, o)| {
                t[n] = *o;
                i += 1;
            });
        Self { t, i }
    }

    /// Returns the length of the collection
    #[inline]
    pub fn len(&self) -> usize {
        self.i
    }
    /// Checks if the collection is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.i == 0
    }

    #[must_use]
    fn last(&self) -> Option<&CompiledTerminal> {
        if self.is_empty() {
            None
        } else {
            Some(&self.t[self.i - 1])
        }
    }

    /// Checks if the collection is k-complete, i.e. no terminals can be added
    /// ```
    /// use parol::analysis::k_tuple::Terminals;
    /// let t = Terminals::end();
    /// assert!(t.is_k_complete(5));
    /// ```
    pub fn is_k_complete(&self, k: usize) -> bool {
        !self.is_eps() && (self.len() >= k || self.last().map_or(false, |t| t.is_end()))
    }

    /// Returns the k-length, i.e. the number of symbols that contributes to lookahead sizes
    #[must_use]
    pub fn k_len(&self, k: usize) -> usize {
        let mut k_len = 0;
        for i in 0..self.i {
            if k_len >= k {
                break;
            }
            // if self.t[i].is_eps() {
            //     break;
            // }
            k_len += 1;
            if self.t[i].is_end() {
                break;
            }
        }
        k_len
    }

    /// Clears the collection
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// Concatenates two collections with respect to the rules of k-concatenation
    pub fn k_concat(mut self, other: &Self, k: usize) -> Self {
        if other.is_eps() {
            // w + ε = W
            return self;
        }

        if self.is_eps() {
            // ε + w = w
            // Remove possible epsilon terminal
            self.clear();
        }

        if self.is_k_complete(k) {
            // k: w would be the same as k: (w + x)
            return self;
        }

        let my_k_len = self.k_len(k);
        let to_take = other.k_len(k - my_k_len);
        for i in 0..to_take {
            self.t[self.i] = other.t[i];
            self.i += 1;
        }
        self
    }

    /// Adds a new terminal to self if max size is not reached yet and if last is not EOI
    pub fn push(&mut self, t: CompiledTerminal) {
        if self.i < MAX_K && !matches!(self.last(), Some(CompiledTerminal(EOI))) {
            self.t[self.i] = t;
            self.i += 1;
        }
    }

    /// Checks if self is an Epsilon
    #[inline]
    pub fn is_eps(&self) -> bool {
        self.i == 1 && self.t[0].is_eps()
    }

    /// Checks if self is an end-of-input symbol
    #[inline]
    pub fn is_end(&self) -> bool {
        self.i == 1 && self.t[0].is_end()
    }
}

impl Ord for Terminals {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.i.cmp(&other.i) {
            std::cmp::Ordering::Less => std::cmp::Ordering::Less,
            std::cmp::Ordering::Equal => self.t[0..self.i].cmp(&other.t[0..other.i]),
            std::cmp::Ordering::Greater => std::cmp::Ordering::Greater,
        }
    }
}

impl PartialOrd for Terminals {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.i.partial_cmp(&other.i) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.t[0..self.i].partial_cmp(&other.t[0..other.i])
    }
}

impl Index<usize> for Terminals {
    type Output = CompiledTerminal;

    fn index(&self, index: usize) -> &Self::Output {
        &self.t[index]
    }
}

impl Display for Terminals {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "[{}(i{})]",
            self.t
                .iter()
                .take(self.i)
                .map(|e| format!("{}", e))
                .collect::<Vec<String>>()
                .join(", "),
            self.i,
        )
    }
}

impl Debug for Terminals {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // Forward to Display implementation
        write!(f, "{}", self)
    }
}

/// Terminal string with support for k-completeness
#[derive(Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum TerminalString {
    /// Incomplete sequence
    Incomplete(Terminals),
    /// k-complete sequence
    Complete(Terminals),
}

impl TerminalString {
    /// Returns the length of the sequence
    pub fn len(&self) -> usize {
        self.inner().len()
    }
    /// Checks if the sequence is empty
    pub fn is_empty(&self) -> bool {
        self.inner().is_empty()
    }

    /// Checks if the sequence is k-complete
    pub fn is_k_complete(&self) -> bool {
        match self {
            Self::Incomplete(_) => false,
            Self::Complete(_) => true,
        }
    }

    /// Checks if the inner sequence is k-complete
    pub fn is_complete(&self, k: usize) -> bool {
        self.inner().is_k_complete(k)
    }

    /// Change the state to k-complete
    pub fn make_complete(self) -> Self {
        if let Self::Incomplete(e) = self {
            Self::Complete(e)
        } else {
            self
        }
    }

    /// Revoke the k-complete state
    pub fn make_incomplete(self) -> Self {
        if let Self::Complete(e) = self {
            Self::Incomplete(e)
        } else {
            self
        }
    }

    /// Clear the sequences
    pub fn clear(self) -> Self {
        Self::Incomplete(Terminals::new())
    }

    /// Return the inner sequences
    pub fn inner(&self) -> &Terminals {
        match self {
            Self::Incomplete(v) => v,
            Self::Complete(v) => v,
        }
    }

    /// Checks if self is an Epsilon
    pub fn is_eps(&self) -> bool {
        match self {
            Self::Incomplete(v) => v.is_eps(),
            Self::Complete(_) => false,
        }
    }

    /// Checks if self is an end-of-input symbol
    pub fn is_end(&self) -> bool {
        match self {
            Self::Incomplete(_) => false,
            Self::Complete(v) => v.is_end(),
        }
    }

    /// Push a new terminal
    pub fn push(self, t: CompiledTerminal, k: usize) -> Self {
        match self {
            Self::Incomplete(mut v) => {
                v.push(t);
                if v.is_k_complete(k) {
                    Self::Complete(v)
                } else {
                    Self::Incomplete(v)
                }
            }
            Self::Complete(_) => self,
        }
    }

    /// Concat self with another sequence while consuming self
    pub fn k_concat(self, other: &Self, k: usize) -> Self {
        match self {
            Self::Incomplete(v) => {
                let terminals = v.k_concat(other.inner(), k);
                if terminals.is_k_complete(k) {
                    TerminalString::Complete(terminals)
                } else {
                    TerminalString::Incomplete(terminals)
                }
            }
            Self::Complete(_) => self,
        }
    }
}

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
///
/// Terminal symbol string type
///
#[derive(Clone, Eq, Ord, PartialOrd)]
pub struct KTuple {
    /// The sequence of terminals
    pub terminals: TerminalString,
    /// The lookahead size
    pub k: usize,
}

impl KTuple {
    ///
    /// Creates a new and empty object.
    ///
    pub fn new(k: usize) -> Self {
        let terminals = TerminalString::Incomplete(Terminals::new());
        Self { terminals, k }
    }

    /// Used for debugging only
    pub fn with_terminal_indices(self, terms: &[TerminalIndex]) -> Self {
        let k = self.k;
        let mut terminals = match self.terminals {
            TerminalString::Incomplete(s) => s,
            TerminalString::Complete(s) => s,
        };

        let mut i = 0;
        terminals
            .t
            .iter_mut()
            .zip(terms.iter())
            .take(k)
            .for_each(|(l, r)| {
                *l = CompiledTerminal(*r);
                i += 1;
            });
        terminals.i = i;

        let terminals = if terminals.is_k_complete(k) {
            TerminalString::Complete(terminals)
        } else {
            TerminalString::Incomplete(terminals)
        };

        Self { terminals, k }
    }

    ///
    /// Creates a new object from a slice of other objects while applying a mapper function
    ///
    pub fn from_slice_with<'s, S, M>(others: &'s [S], m: M, k: usize) -> Self
    where
        M: Fn(&'s S) -> CompiledTerminal,
    {
        let terminals = Terminals::from_slice_with(others, k, m);
        let terminals = if terminals.is_k_complete(k) {
            TerminalString::Complete(terminals)
        } else {
            TerminalString::Incomplete(terminals)
        };
        Self { terminals, k }
    }

    ///
    /// Creates a new object from a slice of CompiledTerminals
    ///
    pub fn from_slice(others: &[CompiledTerminal], k: usize) -> Self {
        let terminals = Terminals::from_slice(others, k);
        let terminals = if terminals.is_k_complete(k) {
            TerminalString::Complete(terminals)
        } else {
            TerminalString::Incomplete(terminals)
        };
        Self { terminals, k }
    }

    ///
    /// Creates a new object from a vector of terminal symbols
    ///
    pub fn of(t: Terminals, k: usize) -> Self {
        let terminals = Terminals::of(k, t);

        let terminals = if terminals.is_k_complete(k) {
            TerminalString::Complete(terminals)
        } else {
            TerminalString::Incomplete(terminals)
        };
        Self { terminals, k }
    }

    ///
    /// Creates a new ε object
    ///
    pub fn eps(k: usize) -> Self {
        let terminals = TerminalString::Incomplete(Terminals::eps());
        Self { terminals, k }
    }
    ///
    /// Creates a new End object
    ///
    pub fn end(k: usize) -> Self {
        let terminals = TerminalString::Complete(Terminals::end());
        Self { terminals, k }
    }

    /// Adds a new terminal to self while consuming self
    pub fn push(self, t: CompiledTerminal) -> Self {
        Self {
            terminals: self.terminals.push(t, self.k),
            k: self.k,
        }
    }

    /// Appends a sequence to self while consuming self
    // pub fn append(self, other: &mut Self) -> Self {
    //     Self {
    //         terminals: self.terminals.append(&mut other.terminals, self.k),
    //         k: self.k,
    //     }
    // }

    /// Checks if self is an Epsilon
    pub fn is_eps(&self) -> bool {
        self.terminals.is_eps()
    }
    /// Returns the length of the sequence
    pub fn len(&self) -> usize {
        self.terminals.len()
    }
    /// Checks if the sequence is empty
    pub fn is_empty(&self) -> bool {
        self.terminals.is_empty()
    }
    /// Returns the k-length of the sequence
    pub fn k_len(&self, k: usize) -> usize {
        self.terminals.inner().k_len(k)
    }
    /// Checks if the sequence is k-complete
    pub fn is_k_complete(&self) -> bool {
        self.terminals.is_k_complete()
    }

    /// Concat self with another sequence while consuming self
    pub fn k_concat(self, other: &Self, k: usize) -> Self {
        let terminals = self.terminals.k_concat(&other.terminals, k);
        let k = terminals.inner().k_len(k);
        Self { terminals, k }
    }

    /// Sets the lookahead size
    pub fn set_k(mut self, k: usize) -> Self {
        if self.terminals.is_complete(k) {
            self.terminals = self.terminals.make_complete();
        } else {
            self.terminals = self.terminals.make_incomplete();
        }
        self.k = k;
        self
    }

    /// Conversion to string with the help of the terminals slice
    pub fn to_string(&self, terminals: &[String]) -> String {
        format!(
            "[{}]",
            self.terminals
                .inner()
                .t
                .iter()
                .take(self.terminals.inner().i)
                .map(|t| match t.0 {
                    EOI => "$".to_owned(),
                    NEW_LINE => "NewLine".to_owned(),
                    WHITESPACE => "WhiteSpace".to_owned(),
                    LINE_COMMENT => "LineComment".to_owned(),
                    BLOCK_COMMENT => "BlockComment".to_owned(),
                    EPS => "\u{03B5}".to_owned(),
                    _ => terminals[t.0].to_string(),
                })
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl Debug for KTuple {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        // Forward to the implementation of Display
        write!(f, "{}", self)
    }
}

impl Display for KTuple {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "[{}(i{})](k{})",
            self.terminals.inner(),
            self.terminals.inner().i,
            self.k
        )
    }
}

impl Hash for KTuple {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let self_inner = self.terminals.inner();
        self_inner.t[0..self_inner.i].hash(state)
    }
}

impl PartialEq for KTuple {
    fn eq(&self, other: &Self) -> bool {
        let self_inner = self.terminals.inner();
        let other_inner = other.terminals.inner();
        if self_inner.i == other_inner.i {
            self_inner.t[0..self_inner.i].eq(&other_inner.t[0..other_inner.i])
        } else {
            false
        }
    }
}

#[cfg(test)]
mod test {
    use parol_runtime::TerminalIndex;

    use super::{TerminalString, Terminals};
    use crate::{analysis::k_tuple::EOI, CompiledTerminal, KTuple, MAX_K};

    fn term(terminals: &[TerminalIndex], k: usize) -> Terminals {
        let mut t = <[CompiledTerminal; MAX_K]>::default();
        debug_assert!(k <= MAX_K);
        terminals
            .iter()
            .enumerate()
            .for_each(|(i, x)| t[i] = CompiledTerminal(*x));
        Terminals {
            t,
            i: terminals.len(),
        }
    }

    #[test]
    fn check_with_terminal_indices() {
        {
            let k_tuple = KTuple::new(1).with_terminal_indices(&[1]);
            let mut t = <[CompiledTerminal; MAX_K]>::default();
            let i = 1;
            let k = 1;
            t[0] = CompiledTerminal(1);
            let expected = KTuple {
                terminals: TerminalString::Incomplete(Terminals { t, i }),
                k,
            };
            assert_eq!(CompiledTerminal::default(), t[1]);
            assert_eq!(CompiledTerminal::default(), t[MAX_K - 1]);
            assert_eq!(expected, k_tuple, "[1]");
        }
        {
            let k_tuple =
                KTuple::new(MAX_K).with_terminal_indices(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
            let mut t = <[CompiledTerminal; MAX_K]>::default();
            let i = MAX_K;
            let k = MAX_K;
            (0..MAX_K).for_each(|i| {
                t[i] = CompiledTerminal(i + 1);
            });
            let expected = KTuple {
                terminals: TerminalString::Incomplete(Terminals { t, i }),
                k,
            };
            assert_eq!(CompiledTerminal(1), t[0]);
            assert_eq!(CompiledTerminal(10), t[MAX_K - 1]);
            assert_eq!(expected, k_tuple, "[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]");
        }
        {
            let k_tuple =
                KTuple::new(5).with_terminal_indices(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
            let mut t = <[CompiledTerminal; MAX_K]>::default();
            let i = 5;
            let k = 5;
            (0..5).for_each(|i| {
                t[i] = CompiledTerminal(i + 1);
            });
            let expected = KTuple {
                terminals: TerminalString::Incomplete(Terminals { t, i }),
                k,
            };
            assert_eq!(CompiledTerminal(1), t[0]);
            assert_eq!(CompiledTerminal(5), t[4]);
            assert_eq!(CompiledTerminal::default(), t[5]);
            assert_eq!(expected, k_tuple, "[1, 2, 3, 4, 5]");
        }
    }

    #[test]
    fn check_from_slice() {
        {
            let k_tuple = KTuple::from_slice(&[CompiledTerminal(1)], 1);
            let mut t = <[CompiledTerminal; MAX_K]>::default();
            let i = 1;
            let k = 1;
            t[0] = CompiledTerminal(1);
            let expected = KTuple {
                terminals: TerminalString::Incomplete(Terminals { t, i }),
                k,
            };
            assert_eq!(CompiledTerminal::default(), t[1]);
            assert_eq!(CompiledTerminal::default(), t[MAX_K - 1]);
            assert_eq!(expected, k_tuple, "[1]");
        }
        {
            let k_tuple = KTuple::from_slice(
                &[
                    CompiledTerminal(1),
                    CompiledTerminal(2),
                    CompiledTerminal(3),
                    CompiledTerminal(4),
                    CompiledTerminal(5),
                    CompiledTerminal(6),
                    CompiledTerminal(7),
                    CompiledTerminal(8),
                    CompiledTerminal(9),
                    CompiledTerminal(10),
                    CompiledTerminal(11),
                ],
                MAX_K,
            );
            let mut t = <[CompiledTerminal; MAX_K]>::default();
            let i = MAX_K;
            let k = MAX_K;
            (0..MAX_K).for_each(|i| {
                t[i] = CompiledTerminal(i + 1);
            });
            let expected = KTuple {
                terminals: TerminalString::Incomplete(Terminals { t, i }),
                k,
            };
            assert_eq!(CompiledTerminal(1), t[0]);
            assert_eq!(CompiledTerminal(10), t[MAX_K - 1]);
            assert_eq!(expected, k_tuple);
        }
        {
            let k_tuple = KTuple::from_slice(
                &[
                    CompiledTerminal(1),
                    CompiledTerminal(2),
                    CompiledTerminal(3),
                    CompiledTerminal(4),
                    CompiledTerminal(5),
                    CompiledTerminal(6),
                    CompiledTerminal(7),
                    CompiledTerminal(8),
                    CompiledTerminal(9),
                    CompiledTerminal(10),
                    CompiledTerminal(11),
                ],
                5,
            );
            let mut t = <[CompiledTerminal; MAX_K]>::default();
            let i = 5;
            let k = 5;
            (0..5).for_each(|i| {
                t[i] = CompiledTerminal(i + 1);
            });
            let expected = KTuple {
                terminals: TerminalString::Incomplete(Terminals { t, i }),
                k,
            };
            assert_eq!(CompiledTerminal(1), t[0]);
            assert_eq!(CompiledTerminal(5), t[4]);
            assert_eq!(CompiledTerminal::default(), t[5]);
            assert_eq!(expected, k_tuple, "[1, 2, 3, 4, 5]");
        }
    }

    #[test]
    fn check_from_slice_with() {
        {
            let k_tuple = KTuple::from_slice_with(&[1], |t| CompiledTerminal(*t), 1);
            let mut t = <[CompiledTerminal; MAX_K]>::default();
            let i = 1;
            let k = 1;
            t[0] = CompiledTerminal(1);
            let expected = KTuple {
                terminals: TerminalString::Incomplete(Terminals { t, i }),
                k,
            };
            assert_eq!(CompiledTerminal::default(), t[1]);
            assert_eq!(CompiledTerminal::default(), t[MAX_K - 1]);
            assert_eq!(expected, k_tuple, "[1]");
        }
        {
            let k_tuple = KTuple::from_slice_with(
                &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
                |t| CompiledTerminal(*t),
                MAX_K,
            );
            let mut t = <[CompiledTerminal; MAX_K]>::default();
            let i = MAX_K;
            let k = MAX_K;
            (0..MAX_K).for_each(|i| {
                t[i] = CompiledTerminal(i + 1);
            });
            let expected = KTuple {
                terminals: TerminalString::Incomplete(Terminals { t, i }),
                k,
            };
            assert_eq!(CompiledTerminal(1), t[0]);
            assert_eq!(CompiledTerminal(10), t[MAX_K - 1]);
            assert_eq!(expected, k_tuple);
        }
        {
            let k_tuple = KTuple::from_slice_with(
                &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
                |t| CompiledTerminal(*t),
                5,
            );
            let mut t = <[CompiledTerminal; MAX_K]>::default();
            let i = 5;
            let k = 5;
            (0..5).for_each(|i| {
                t[i] = CompiledTerminal(i + 1);
            });
            let expected = KTuple {
                terminals: TerminalString::Incomplete(Terminals { t, i }),
                k,
            };
            assert_eq!(CompiledTerminal(1), t[0]);
            assert_eq!(CompiledTerminal(5), t[4]);
            assert_eq!(CompiledTerminal::default(), t[5]);
            assert_eq!(expected, k_tuple, "[1, 2, 3, 4, 5]");
        }
    }

    #[test]
    fn check_k_tuple_of() {
        {
            let k = 1;
            let t = Terminals::from_slice_with(&[1], k, |t| CompiledTerminal(*t));
            let k_tuple = KTuple::of(t.clone(), k);
            let i = k;
            let expected = KTuple {
                terminals: TerminalString::Complete(Terminals { t: t.t.clone(), i }),
                k,
            };
            assert_eq!(CompiledTerminal::default(), t.t[1]);
            assert_eq!(CompiledTerminal::default(), t.t[MAX_K - 1]);
            assert_eq!(expected, k_tuple, "[1]");
        }
        {
            let k = MAX_K;
            let t = Terminals::from_slice_with(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10], k, |t| {
                CompiledTerminal(*t)
            });
            let k_tuple = KTuple::of(t.clone(), k);
            assert_eq!(MAX_K, k_tuple.len());
            let i = k;
            let expected = KTuple {
                terminals: TerminalString::Complete(Terminals { t: t.t.clone(), i }),
                k,
            };
            assert_eq!(CompiledTerminal(1), t.t[0]);
            assert_eq!(CompiledTerminal(10), t.t[MAX_K - 1]);
            assert_eq!(expected, k_tuple, "[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]");
        }
        {
            let k = 5;
            let t = Terminals::from_slice_with(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11], k, |t| {
                CompiledTerminal(*t)
            });
            let k_tuple = KTuple::of(t.clone(), k);
            let i = k;
            let expected = KTuple {
                terminals: TerminalString::Complete(Terminals { t: t.t.clone(), i }),
                k,
            };
            assert_eq!(CompiledTerminal(1), t.t[0]);
            assert_eq!(CompiledTerminal(5), t.t[4]);
            assert_eq!(CompiledTerminal::default(), t.t[5]);
            assert_eq!(expected, k_tuple, "[1, 2, 3, 4, 5]");
        }
    }

    #[test]
    fn check_k_concat() {
        {
            let tuple1 = KTuple::eps(1);
            let tuple2 = KTuple::eps(1);
            let result = tuple1.k_concat(&tuple2, 1);
            let expected = KTuple::eps(1);
            assert_eq!(expected, result, "1: [ε] + [ε] = [ε]");
        }
        {
            let tuple1 = KTuple::new(1).with_terminal_indices(&[1]);
            let tuple2 = KTuple::eps(1);
            let result = tuple1.k_concat(&tuple2, 1);
            let expected = KTuple::new(1).with_terminal_indices(&[1]);
            assert_eq!(expected, result, "1: [a] + [ε] = [a]");
        }
        {
            let tuple1 = KTuple::eps(1);
            let tuple2 = KTuple::new(1).with_terminal_indices(&[1]);
            let result = tuple1.k_concat(&tuple2, 1);
            let expected = KTuple::new(1).with_terminal_indices(&[1]);
            assert_eq!(expected, result, "1: [ε] + [a] = [a]");
        }
        {
            let tuple1 = KTuple::new(2).with_terminal_indices(&[1]);
            let tuple2 = KTuple::new(2).with_terminal_indices(&[2]);
            let result = tuple1.k_concat(&tuple2, 2);
            let expected = KTuple::new(2).with_terminal_indices(&[1, 2]);
            assert_eq!(expected, result, "2: [a] + [b] = [ab]");
        }
    }

    #[test]
    fn check_term() {
        {
            let terminals = Terminals::new();
            assert_eq!(0, terminals.k_len(0));
            assert_eq!(0, terminals.k_len(1));
            assert_eq!(0, terminals.k_len(2));

            assert!(terminals.is_k_complete(0));
            assert!(!terminals.is_k_complete(1));
            assert!(!terminals.is_k_complete(2));
            assert!(!terminals.is_k_complete(3));
        }
        {
            let terminals = term(&[1], 1);
            assert_eq!(0, terminals.k_len(0));
            assert_eq!(1, terminals.k_len(1));
            assert_eq!(1, terminals.k_len(2));

            assert!(terminals.is_k_complete(0));
            assert!(terminals.is_k_complete(1));
            assert!(!terminals.is_k_complete(2));
            assert!(!terminals.is_k_complete(3));
        }
        {
            let terminals = term(&[1, 2], 2);
            assert_eq!(0, terminals.k_len(0));
            assert_eq!(1, terminals.k_len(1));
            assert_eq!(2, terminals.k_len(2));
            assert_eq!(2, terminals.k_len(3));

            assert!(terminals.is_k_complete(0));
            assert!(terminals.is_k_complete(1));
            assert!(terminals.is_k_complete(2));
            assert!(!terminals.is_k_complete(3));
        }
        {
            let terminals = term(&[1, EOI], 2);
            assert_eq!(0, terminals.k_len(0));
            assert_eq!(1, terminals.k_len(1));
            assert_eq!(2, terminals.k_len(2));
            assert_eq!(2, terminals.k_len(3));

            assert!(terminals.is_k_complete(0));
            assert!(terminals.is_k_complete(1));
            assert!(terminals.is_k_complete(2));
            assert!(terminals.is_k_complete(3));
        }
        {
            let terminals = term(
                &[
                    1, EOI, 1, // This constellation is actually illegal!
                ],
                3,
            );
            assert_eq!(0, terminals.k_len(0));
            assert_eq!(1, terminals.k_len(1));
            assert_eq!(2, terminals.k_len(2));
            assert_eq!(2, terminals.k_len(3));

            assert!(terminals.is_k_complete(0));
            assert!(terminals.is_k_complete(1));
            assert!(terminals.is_k_complete(2));
            assert!(terminals.is_k_complete(3));

            let terminals2 = term(&[3], 1);
            let result = terminals.k_concat(&terminals2, 3);
            assert_eq!(term(&[1, EOI, 1], 3), result);
        }
    }
}
