use crate::KTuple;
//use parol_runtime::log::trace;
use std::fmt::{Debug, Display, Error, Formatter};

use super::terminals_trie::Trie;

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
///
/// A set type consisting of terminal strings (called k-tuples)
///
#[derive(Clone, Default, Eq)]
pub struct KTuples(Trie, usize, bool);

impl KTuples {
    /// Creates a new item
    pub fn new(k: usize) -> Self {
        Self(Trie::new(), k, false)
    }

    /// Creates a new item from a slice of KTuples
    pub fn of(tuples: &[KTuple], k: usize) -> Self {
        let mut tuples = Self(
            tuples.iter().fold(Trie::new(), |mut acc, t| {
                acc.insert(t);
                acc
            }),
            k,
            false,
        );
        tuples.update_completeness();
        tuples
    }

    /// Inserts a KTuple
    pub fn insert(&mut self, tuple: KTuple) {
        debug_assert!(self.1 >= tuple.k);
        self.2 &= tuple.is_k_complete();
        self.0.insert(&tuple);
    }

    /// Appends another KTuples item to self
    pub fn append(&mut self, mut other: Self) -> bool {
        let count = self.0.len();
        self.0.append(&other.0);
        count != self.0.len()
    }

    /// Creates a union with another KTuples and self
    pub fn union(&self, mut other: Self) -> Self {
        let unn = self.0.union(&other.0);
        let mut tuples = Self(unn, self.1, false);
        tuples.update_completeness();
        tuples
    }

    /// Creates a intersection with another KTuples and self
    pub fn intersection(&self, other: &Self) -> Self {
        let mut tuples = Self(self.0.intersection(&other.0), self.1, false);
        tuples.update_completeness();
        tuples
    }

    /// Returns the number of `KTuple`s
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Checks if the collection is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Checks if self and other are disjoint
    pub fn is_disjoint(&self, other: &Self) -> bool {
        self.0.is_disjoint(&other.0)
    }

    /// Creates an epsilon item, i.e. a set with exactly one epsilon k-tuple
    pub fn eps(k: usize) -> Self {
        let set = Trie::eps();
        Self(set, k, false)
    }

    /// Creates an end-of-input item, i.e. a set with exactly one end-of-input k-tuple
    pub fn end(k: usize) -> Self {
        let set = Trie::end();
        Self(set, k, true)
    }

    ///
    /// Creates a new object from a slice of KTuple objects.
    ///
    /// ```
    /// use parol::{KTuple, KTuples, CompiledTerminal};
    /// use parol::analysis::k_tuple::TerminalMappings;
    /// use parol::analysis::compiled_terminal::EPS;
    ///
    /// {
    ///     let tuples1 = KTuples::of(&vec![KTuple::new(1).with_terminal_indices(&[EPS])], 1);
    ///     let tuples2 = KTuples::of(&vec![KTuple::new(1).with_terminal_indices(&[EPS])], 1);
    ///     let result = tuples1.k_concat(&tuples2, 1);
    ///     let expected = KTuples::of(&vec![KTuple::new(1).with_terminal_indices(&[EPS])], 1);
    ///     assert_eq!(expected, result, "[ε] + [ε] = [ε]");
    /// }
    /// {
    ///     let tuples1 = KTuples::of(&vec![KTuple::new(1).with_terminal_indices(&[1])], 1);
    ///     let tuples2 = KTuples::of(&vec![KTuple::new(1).with_terminal_indices(&[EPS])], 1);
    ///     let result = tuples1.k_concat(&tuples2, 1);
    ///     let expected = KTuples::of(&vec![KTuple::new(1).with_terminal_indices(&[1])], 1);
    ///     assert_eq!(expected, result, "[a] + [ε] = [a]");
    /// }
    /// {
    ///     let tuples1 = KTuples::of(&vec![KTuple::new(1).with_terminal_indices(&[EPS])], 1);
    ///     let tuples2 = KTuples::of(&vec![KTuple::new(1).with_terminal_indices(&[1])], 1);
    ///     let result = tuples1.k_concat(&tuples2, 1);
    ///     let expected = KTuples::of(&vec![KTuple::new(1).with_terminal_indices(&[1])], 1);
    ///     assert_eq!(expected, result, "[ε] + [a] = [a]");
    /// }
    /// {
    ///     let tuples1 = KTuples::of(&vec![KTuple::new(1).with_terminal_indices(&[1])], 1);
    ///     let tuples2 = KTuples::of(&vec![KTuple::new(1).with_terminal_indices(&[2])], 1);
    ///     let result = tuples1.k_concat(&tuples2, 1);
    ///     let expected = KTuples::of(&vec![KTuple::new(1).with_terminal_indices(&[1])], 1);
    ///     assert_eq!(expected, result, "1: [a] + [b] = [a]");
    /// }
    /// {
    ///     let tuples1 = KTuples::of(&vec![KTuple::new(2).with_terminal_indices(&[1])], 2);
    ///     let tuples2 = KTuples::of(&vec![KTuple::new(2).with_terminal_indices(&[2])], 2);
    ///     let result = tuples1.k_concat(&tuples2, 2);
    ///     let expected = KTuples::of(&vec![KTuple::new(2).with_terminal_indices(&[1, 2])], 2);
    ///     assert_eq!(expected, result, "2: [a] + [b] = [ab]");
    /// }
    ///
    /// ```
    pub fn k_concat(mut self, other: &Self, k: usize) -> Self {
        // trace!("KTuples::k_concat {} with {} at k={}", self, other, k);
        if !self.2 {
            let (complete, incomplete): (Trie, Trie) =
                self.0.iter().partition(|t| t.is_k_complete(k));
            self.0 = complete;
            self.0.extend(
                incomplete
                    .iter()
                    .flat_map(|t| other.0.iter().map(move |o| t.clone().k_concat(&o, k))),
            );
            self.update_completeness();
        }
        // trace!("KTuples::k_concat => {}", result);
        self
    }

    /// Conversion to string with the help of the terminals slice
    pub fn to_string(&self, terminals: &[String]) -> String {
        format!(
            "{{{}}}(k={})",
            self.sorted()
                .iter()
                .map(|t| t.to_string(terminals))
                .collect::<Vec<String>>()
                .join(", "),
            self.1
        )
    }

    /// Set the lookahead size
    pub fn set_k(mut self, k: usize) -> Self {
        self.1 = k;
        self.update_completeness();
        self
    }

    /// Returns a sorted representation of self
    pub fn sorted(&self) -> Vec<KTuple> {
        let mut sorted_k_tuples: Vec<KTuple> =
            self.0.iter().map(|t| KTuple::of(t, self.1)).collect();
        sorted_k_tuples.sort_by(|a, b| a.partial_cmp(b).unwrap());
        sorted_k_tuples
    }

    fn update_completeness(&mut self) {
        self.2 = self.0.iter().all(|t| t.is_k_complete(self.1));
    }
}

impl Debug for KTuples {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        // Use the implementation of Display
        write!(f, "{}", self)
    }
}

impl Display for KTuples {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "{{{}}}(k={})",
            self.0
                .iter()
                .map(|e| format!("{}", e))
                .collect::<Vec<String>>()
                .join(", "),
            self.1
        )
    }
}

impl PartialEq for KTuples {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

#[cfg(test)]
mod test {
    use crate::{KTuple, KTuples};

    #[test]
    fn k_tuples_eq_positive() {
        let tuples1 = KTuples::of(
            &vec![
                KTuple::new(6).with_terminal_indices(&[1, 2, 3]),
                KTuple::new(6).with_terminal_indices(&[1, 2, 4]),
            ],
            6,
        );
        let tuples2 = KTuples::of(
            &vec![
                KTuple::new(6).with_terminal_indices(&[1, 2, 3]),
                KTuple::new(6).with_terminal_indices(&[1, 2, 4]),
            ],
            6,
        );
        //     t1    t2
        // ---------------
        //     1     1
        //     |     |
        //     2     2
        //     | \   | \
        //     3  4  3  4
        assert_eq!(tuples1, tuples2);
    }

    #[test]
    fn k_tuples_eq_negative() {
        let tuples1 = KTuples::of(
            &vec![
                KTuple::new(6).with_terminal_indices(&[1, 2, 3]),
                KTuple::new(6).with_terminal_indices(&[1, 2, 4]),
            ],
            6,
        );
        let tuples2 = KTuples::of(
            &vec![
                KTuple::new(6).with_terminal_indices(&[5, 6, 7]),
                KTuple::new(6).with_terminal_indices(&[5, 8]),
            ],
            6,
        );
        //     t1    t2
        // ---------------
        //     1     5
        //     |     | \
        //     2     6  8
        //     | \   |
        //     3  4  7
        assert_ne!(tuples1, tuples2);
    }
}
