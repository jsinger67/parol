use crate::analysis::compiled_la_dfa::TerminalIndex;
use crate::analysis::compiled_terminal::EPS;
use crate::{CompiledTerminal, MAX_K};
use std::fmt::{Debug, Display, Error, Formatter};
use std::hash::{Hash, Hasher};

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
}

/// An ordered collection of terminals
#[derive(Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Terminals(pub Vec<CompiledTerminal>);

impl Terminals {
    /// Creates a new item with initial capacity
    pub fn new() -> Self {
        Self(Vec::with_capacity(MAX_K))
    }

    ///
    /// Creates a new object with maximum k length from another object
    ///
    pub fn of(k: usize, mut other: Self) -> Self {
        let first_len = other.k_len(k);
        let mut terminals = Self::new();
        for elem in other.0.drain(..).take(first_len) {
            terminals.0.push(elem);
        }
        terminals
    }

    ///
    /// Creates a new object from a slice of other objects
    ///
    pub fn from_slice<'s, S, M>(others: &'s [S], k: usize, m: M) -> Self
    where
        S: Clone,
        M: Fn(&'s S) -> CompiledTerminal,
    {
        others.iter().take(k).fold(Self::new(), |mut acc, s| {
            acc.0.push(m(s));
            acc
        })
    }

    /// Returns the length of the collection
    pub fn len(&self) -> usize {
        self.0.len()
    }
    /// Checks if the collection is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Checks if the collection is k-complete, i.e. no terminals can be added
    pub fn is_k_complete(&self, k: usize) -> bool {
        !self.is_eps() && (self.len() >= k || (!self.is_empty() && self.0.last().unwrap().is_end()))
    }

    /// Returns the k-length, i.e. the number of symbols that contributes to lookahead sizes
    pub fn k_len(&self, k: usize) -> usize {
        let mut k_len = 0;
        for t in &self.0 {
            if k_len >= k {
                break;
            }
            k_len += 1;
            if t.is_end() {
                break;
            }
        }
        k_len
    }

    /// Clears the collection
    pub fn clear(&mut self) {
        self.0.clear()
    }

    /// Concatenates two collections with respect to the rules of k-concatenation
    pub fn k_concat(mut self, other: &Self, k: usize) -> Self {
        if other.len() == 1 && other.0[0].is_eps() {
            // w + ε = W
            return self;
        }

        if self.len() == 1 && self.0[0].is_eps() {
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
            self.0.push(other.0[i]);
        }
        self
    }

    /// Checks if self is an Epsilon
    pub fn is_eps(&self) -> bool {
        self.len() == 1 && self.0[0].is_eps()
    }

    /// Checks if self is an end-of-input symbol
    pub fn is_end(&self) -> bool {
        self.len() == 1 && self.0[0].is_end()
    }
}

impl Default for Terminals {
    fn default() -> Self {
        Self::new()
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
                v.0.push(t);
                if v.is_k_complete(k) {
                    Self::Complete(v)
                } else {
                    Self::Incomplete(v)
                }
            }
            Self::Complete(_) => self,
        }
    }

    /// Append a sequence
    pub fn append(self, other: &mut Self, k: usize) -> Self {
        match self {
            Self::Incomplete(mut v) => {
                let my_k_len = v.k_len(k);
                let to_take = other.inner().k_len(k - my_k_len);
                v.0.append(&mut other.inner().0[0..to_take].to_vec());
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
            Self::Complete(_) => self.clone(),
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

    ///
    /// Creates a new object from a slice of other objects
    ///
    pub fn from_slice<'s, S, M>(others: &'s [S], m: M, k: usize) -> Self
    where
        S: Clone,
        M: Fn(&'s S) -> CompiledTerminal,
    {
        let terminals = Terminals::from_slice(others, k, m);
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
    pub fn of(t: Vec<CompiledTerminal>, k: usize) -> Self {
        let terminals = Terminals::of(k, Terminals(t));

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
        let terminals = TerminalString::Incomplete(Terminals(vec![CompiledTerminal::eps()]));
        Self { terminals, k }
    }
    ///
    /// Creates a new End object
    ///
    pub fn end(k: usize) -> Self {
        let terminals = TerminalString::Complete(Terminals(vec![CompiledTerminal::end()]));
        Self { terminals, k }
    }
    ///
    /// Empties the object
    ///
    pub fn clear(self) -> Self {
        Self {
            terminals: self.terminals.clear(),
            k: self.k,
        }
    }
    /// Adds a new terminal to self while consuming self
    pub fn push(self, t: CompiledTerminal) -> Self {
        Self {
            terminals: self.terminals.push(t, self.k),
            k: self.k,
        }
    }

    /// Appends a sequence to self while consuming self
    pub fn append(self, other: &mut Self) -> Self {
        Self {
            terminals: self.terminals.append(&mut other.terminals, self.k),
            k: self.k,
        }
    }

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
        Self {
            terminals: self.terminals.k_concat(&other.terminals, k),
            k: self.k,
        }
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
                .0
                .iter()
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
            "[{}](k{})",
            self.terminals
                .inner()
                .0
                .iter()
                .map(|e| format!("{}", e))
                .collect::<Vec<String>>()
                .join(", "),
            self.k
        )
    }
}

impl Hash for KTuple {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.terminals.inner().hash(state)
    }
}

impl PartialEq for KTuple {
    fn eq(&self, other: &Self) -> bool {
        self.terminals.inner().eq(other.terminals.inner())
    }
}

#[cfg(test)]
mod test {
    use super::{TerminalMappings, Terminals};
    use crate::{CompiledTerminal, KTuple};

    #[test]
    fn check_k_concat() {
        {
            let tuple1 = KTuple::of(vec![CompiledTerminal::eps()], 1);
            let tuple2 = KTuple::of(vec![CompiledTerminal::eps()], 1);
            let result = tuple1.k_concat(&tuple2, 1);
            let expected = KTuple::of(vec![CompiledTerminal::eps()], 1);
            assert_eq!(expected, result, "1: [ε] + [ε] = [ε]");
        }
        {
            let tuple1 = KTuple::of(vec![CompiledTerminal(1)], 1);
            let tuple2 = KTuple::of(vec![CompiledTerminal::eps()], 1);
            let result = tuple1.k_concat(&tuple2, 1);
            let expected = KTuple::of(vec![CompiledTerminal(1)], 1);
            assert_eq!(expected, result, "1: [a] + [ε] = [a]");
        }
        {
            let tuple1 = KTuple::of(vec![CompiledTerminal::eps()], 1);
            let tuple2 = KTuple::of(vec![CompiledTerminal(1)], 1);
            let result = tuple1.k_concat(&tuple2, 1);
            let expected = KTuple::of(vec![CompiledTerminal(1)], 1);
            assert_eq!(expected, result, "1: [ε] + [a] = [a]");
        }
        {
            let tuple1 = KTuple::of(vec![CompiledTerminal(1)], 2);
            let tuple2 = KTuple::of(vec![CompiledTerminal(2)], 2);
            let result = tuple1.k_concat(&tuple2, 2);
            let expected = KTuple::of(vec![CompiledTerminal(1), CompiledTerminal(2)], 2);
            assert_eq!(expected, result, "2: [a] + [b] = [ab]");
        }
    }

    #[test]
    fn check_terminals() {
        {
            let terminals = Terminals(vec![]);
            assert_eq!(0, terminals.k_len(0));
            assert_eq!(0, terminals.k_len(1));
            assert_eq!(0, terminals.k_len(2));

            assert!(terminals.is_k_complete(0));
            assert!(!terminals.is_k_complete(1));
            assert!(!terminals.is_k_complete(2));
            assert!(!terminals.is_k_complete(3));
        }
        {
            let terminals = Terminals(vec![CompiledTerminal(1)]);
            assert_eq!(0, terminals.k_len(0));
            assert_eq!(1, terminals.k_len(1));
            assert_eq!(1, terminals.k_len(2));

            assert!(terminals.is_k_complete(0));
            assert!(terminals.is_k_complete(1));
            assert!(!terminals.is_k_complete(2));
            assert!(!terminals.is_k_complete(3));
        }
        {
            let terminals = Terminals(vec![CompiledTerminal(1), CompiledTerminal(2)]);
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
            let terminals = Terminals(vec![CompiledTerminal(1), CompiledTerminal::end()]);
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
            let terminals = Terminals(vec![
                CompiledTerminal(1),
                CompiledTerminal::end(),
                CompiledTerminal(1), // This constellation is actually illegal!
            ]);
            assert_eq!(0, terminals.k_len(0));
            assert_eq!(1, terminals.k_len(1));
            assert_eq!(2, terminals.k_len(2));
            assert_eq!(2, terminals.k_len(3));

            assert!(terminals.is_k_complete(0));
            assert!(terminals.is_k_complete(1));
            assert!(terminals.is_k_complete(2));
            assert!(terminals.is_k_complete(3));

            let terminals2 = Terminals(vec![CompiledTerminal(3)]);
            let result = terminals.k_concat(&terminals2, 3);
            assert_eq!(
                vec![
                    CompiledTerminal(1),
                    CompiledTerminal::end(),
                    CompiledTerminal(1)
                ],
                result.0
            );
        }
    }
}
