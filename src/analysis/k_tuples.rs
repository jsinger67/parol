use crate::KTuple;
//use log::trace;
use std::collections::HashSet;
use std::fmt::{Debug, Display, Error, Formatter};

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
///
/// A set type consisting of terminal strings (called k-tuples)
///
#[derive(Clone, Default, Eq)]
pub struct KTuples(pub HashSet<KTuple>, usize, bool);

impl KTuples {
    /// Creates a new item
    pub fn new(k: usize) -> Self {
        Self(HashSet::new(), k, false)
    }

    /// Creates a new item from a slice of KTuples
    pub fn of(tuples: &[KTuple], k: usize) -> Self {
        let mut tuples = Self(tuples.iter().cloned().collect(), k, false);
        tuples.update_completeness();
        tuples
    }

    /// Inserts a KTuple
    pub fn insert(&mut self, tuple: KTuple) {
        self.2 &= tuple.is_k_complete();
        self.0.insert(tuple);
    }

    /// Removes a KTuple
    pub fn remove(&mut self, tuple: &KTuple) {
        self.0.remove(tuple);
    }

    /// Appends another KTuples item to self
    pub fn append(&mut self, other: &mut Self) -> bool {
        let count = self.0.len();
        for t in other.0.drain() {
            self.insert(t);
        }
        count != self.0.len()
    }

    /// Creates a union with another KTuples and self
    pub fn union(&self, other: &Self) -> Self {
        let mut tuples = Self(
            self.0.union(&other.0).cloned().collect::<HashSet<KTuple>>(),
            self.1,
            false,
        );
        tuples.update_completeness();
        tuples
    }

    /// Creates a intersection with another KTuples and self
    pub fn intersection(&self, other: &Self) -> Self {
        let mut tuples = Self(
            self.0
                .intersection(&other.0)
                .cloned()
                .collect::<HashSet<KTuple>>(),
            self.1,
            false,
        );
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

    /// Checks if self is epsilon
    pub fn eps(k: usize) -> Self {
        let mut set = HashSet::new();
        set.insert(KTuple::eps(k));
        Self(set, k, false)
    }

    /// Checks if self is end-of-input representation
    pub fn end(k: usize) -> Self {
        let mut set = HashSet::new();
        set.insert(KTuple::end(k));
        Self(set, k, true)
    }

    ///
    /// Creates a new object from a slice of KTuple objects.
    ///
    /// ```
    /// use parol::{KTuple, KTuples, CompiledTerminal};
    /// use parol::analysis::k_tuple::TerminalMappings;
    ///
    /// {
    ///     let tuples1 = KTuples::of(&vec![KTuple::of(vec![CompiledTerminal::eps()], 1)], 1);
    ///     let tuples2 = KTuples::of(&vec![KTuple::of(vec![CompiledTerminal::eps()], 1)], 1);
    ///     let result = tuples1.k_concat(&tuples2, 1);
    ///     let expected = KTuples::of(&vec![KTuple::of(vec![CompiledTerminal::eps()], 1)], 1);
    ///     assert_eq!(expected, result, "[ε] + [ε] = [ε]");
    /// }
    /// {
    ///     let tuples1 = KTuples::of(&vec![KTuple::of(vec![CompiledTerminal(1)], 1)], 1);
    ///     let tuples2 = KTuples::of(&vec![KTuple::of(vec![CompiledTerminal::eps()], 1)], 1);
    ///     let result = tuples1.k_concat(&tuples2, 1);
    ///     let expected = KTuples::of(&vec![KTuple::of(vec![CompiledTerminal(1)], 1)], 1);
    ///     assert_eq!(expected, result, "[a] + [ε] = [a]");
    /// }
    /// {
    ///     let tuples1 = KTuples::of(&vec![KTuple::of(vec![CompiledTerminal::eps()], 1)], 1);
    ///     let tuples2 = KTuples::of(&vec![KTuple::of(vec![CompiledTerminal(1)], 1)], 1);
    ///     let result = tuples1.k_concat(&tuples2, 1);
    ///     let expected = KTuples::of(&vec![KTuple::of(vec![CompiledTerminal(1)], 1)], 1);
    ///     assert_eq!(expected, result, "[ε] + [a] = [a]");
    /// }
    /// {
    ///     let tuples1 = KTuples::of(&vec![KTuple::of(vec![CompiledTerminal(1)], 1)], 1);
    ///     let tuples2 = KTuples::of(&vec![KTuple::of(vec![CompiledTerminal(2)], 1)], 1);
    ///     let result = tuples1.k_concat(&tuples2, 1);
    ///     let expected = KTuples::of(&vec![KTuple::of(vec![CompiledTerminal(1)], 1)], 1);
    ///     assert_eq!(expected, result, "1: [a] + [b] = [a]");
    /// }
    /// {
    ///     let tuples1 = KTuples::of(&vec![KTuple::of(vec![CompiledTerminal(1)], 2)], 2);
    ///     let tuples2 = KTuples::of(&vec![KTuple::of(vec![CompiledTerminal(2)], 2)], 2);
    ///     let result = tuples1.k_concat(&tuples2, 2);
    ///     let expected = KTuples::of(&vec![KTuple::of(vec![CompiledTerminal(1), CompiledTerminal(2)], 2)], 2);
    ///     assert_eq!(expected, result, "2: [a] + [b] = [ab]");
    /// }
    ///
    /// ```
    pub fn k_concat(mut self, other: &Self, k: usize) -> Self {
        // trace!("KTuples::k_concat {} with {} at k={}", self, other, k);
        if !self.2 {
            let mut to_remove = Vec::<KTuple>::with_capacity(self.0.len()); // Maximum possible size
            let mut to_insert = HashSet::<KTuple>::with_capacity(self.0.len()); // Start size
            for i in &self.0 {
                if !i.is_k_complete() {
                    to_remove.push(i.clone());
                    for j in &other.0 {
                        to_insert.insert(i.clone().k_concat(j, k));
                    }
                }
            }
            for i in &to_remove {
                self.remove(i);
            }
            for i in to_insert {
                self.insert(i);
            }
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
        self.0 = self.0.drain().map(|t| t.set_k(k)).collect();
        if k > self.1 {
            self.2 = false;
        } else {
            self.update_completeness();
        }
        self.1 = k;
        self
    }

    /// Returns a sorted representation of self
    pub fn sorted(&self) -> Vec<KTuple> {
        let mut sorted_k_tuples: Vec<KTuple> = self.0.iter().cloned().collect();
        sorted_k_tuples.sort_by(|a, b| a.partial_cmp(b).unwrap());
        sorted_k_tuples
    }

    fn update_completeness(&mut self) {
        self.2 = self.0.iter().all(|t| t.is_k_complete());
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
