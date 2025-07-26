use parol_runtime::TerminalIndex;

use crate::KTuple;
//use parol_runtime::log::trace;
use std::fmt::{Debug, Display, Error, Formatter};

use rustc_hash::FxHashSet;

use super::k_tuple::KTupleBuilder;

type TuplesSet = FxHashSet<KTuple>;

/// Builder for KTuples
#[derive(Clone, Debug, Default)]
pub struct KTuplesBuilder<'a> {
    k: Option<usize>,
    max_terminal_index: Option<usize>,
    k_tuples: Option<&'a [KTuple]>,
    terminal_strings: Option<&'a [&'a [TerminalIndex]]>,
}

impl<'a> KTuplesBuilder<'a> {
    /// Creates a new builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the lookahead size
    pub fn k(&mut self, k: usize) -> &mut Self {
        self.k = Some(k);
        self
    }

    /// Sets the maximum terminal index
    pub fn max_terminal_index(&mut self, max_terminal_index: usize) -> &mut Self {
        self.max_terminal_index = Some(max_terminal_index);
        self
    }

    /// Sets the k-tuples
    /// This is optional and can be used to initialize the set with a set of k-tuples
    pub fn k_tuples(&mut self, k_tuples: &'a [KTuple]) -> &mut Self {
        self.k_tuples = Some(k_tuples);
        self
    }

    /// Sets the terminal strings
    /// This is optional and can be used to initialize the set with a set of terminal strings
    pub fn terminal_indices(&mut self, terminal_strings: &'a [&'a [TerminalIndex]]) -> &mut Self {
        self.terminal_strings = Some(terminal_strings);
        self
    }

    /// Creates an epsilon item, i.e. a set with exactly one epsilon k-tuple
    pub fn eps(&self) -> Result<KTuples, String> {
        if self.k.is_none() {
            return Err("k is not set".to_string());
        }
        if self.max_terminal_index.is_none() {
            return Err("max_terminal_index is not set".to_string());
        }
        let set = TuplesSet::from_iter([KTupleBuilder::new()
            .k(self.k.unwrap())
            .max_terminal_index(self.max_terminal_index.unwrap())
            .eps()?]);
        Ok(KTuples {
            set,
            k: self.k.unwrap(),
            max_terminal_index: self.max_terminal_index.unwrap(),
            k_complete: false,
        })
    }

    /// Creates an end-of-input item, i.e. a set with exactly one end-of-input k-tuple
    pub fn end(&self) -> Result<KTuples, String> {
        if self.k.is_none() {
            return Err("k is not set".to_string());
        }
        if self.max_terminal_index.is_none() {
            return Err("max_terminal_index is not set".to_string());
        }
        let set = TuplesSet::from_iter([KTupleBuilder::new()
            .k(self.k.unwrap())
            .max_terminal_index(self.max_terminal_index.unwrap())
            .end()?]);
        Ok(KTuples {
            set,
            k: self.k.unwrap(),
            max_terminal_index: self.max_terminal_index.unwrap(),
            k_complete: true,
        })
    }

    /// Builds the KTuples
    pub fn build(&self) -> Result<KTuples, String> {
        if self.k.is_none() {
            return Err("k is not set".to_string());
        }
        if self.max_terminal_index.is_none() {
            return Err("max_terminal_index is not set".to_string());
        }

        let mut tuples = KTuples {
            set: TuplesSet::default(),
            k: self.k.unwrap(),
            max_terminal_index: self.max_terminal_index.unwrap(),
            k_complete: false,
        };

        if let Some(k_tuples) = self.k_tuples {
            for tuple in k_tuples.iter() {
                tuples.insert(*tuple);
            }
            tuples.update_completeness();
        }

        if let Some(terminal_strings) = self.terminal_strings {
            for terminal_string in terminal_strings.iter() {
                tuples.insert(
                    KTupleBuilder::new()
                        .k(self.k.unwrap())
                        .max_terminal_index(self.max_terminal_index.unwrap())
                        .terminal_string(terminal_string)
                        .build()
                        .unwrap(),
                );
            }
            tuples.update_completeness();
        }

        Ok(tuples)
    }
}

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
///
/// A type representing a set of terminal strings (i.e. terminal sequences) (called k-tuples)
///
#[derive(Clone, Eq, PartialEq)]
pub struct KTuples {
    set: TuplesSet,
    k: usize,
    max_terminal_index: usize,
    k_complete: bool,
}

impl KTuples {
    /// Inserts a KTuple
    pub fn insert(&mut self, tuple: KTuple) {
        debug_assert!(self.k >= tuple.k());
        self.k_complete &= tuple.is_k_complete();
        self.set.insert(tuple);
    }

    /// Appends another KTuples item to self
    pub fn append(&mut self, mut other: Self) -> bool {
        let count = self.set.len();
        // self.set.append(&mut other.set);
        self.set.extend(other.set.drain());
        count != self.set.len()
    }

    /// Creates a union with another KTuples and self
    pub fn union(&self, other: &Self) -> (Self, bool) {
        let len = self.set.len();
        let max_terminal_index = self.max_terminal_index;
        let unn = self.set.union(&other.set).cloned().collect::<TuplesSet>();
        let changed = len != unn.len();
        let mut tuples = Self {
            set: unn,
            k: self.k,
            max_terminal_index,
            k_complete: false,
        };
        tuples.update_completeness();
        (tuples, changed)
    }

    /// Creates a intersection with another KTuples and self
    pub fn intersection(&self, other: &Self) -> Self {
        let max_terminal_index = self.max_terminal_index;
        let mut tuples = Self {
            set: self
                .set
                .intersection(&other.set)
                .cloned()
                .collect::<TuplesSet>(),
            k: self.k,
            max_terminal_index,
            k_complete: false,
        };
        tuples.update_completeness();
        tuples
    }

    /// Returns the number of `KTuple`s
    pub fn len(&self) -> usize {
        self.set.len()
    }

    /// Checks if the collection is empty
    pub fn is_empty(&self) -> bool {
        self.set.is_empty()
    }

    /// Checks if self and other are disjoint
    pub fn is_disjoint(&self, other: &Self) -> bool {
        self.set.is_disjoint(&other.set)
    }

    ///
    /// Creates a new object from a slice of KTuple objects.
    ///
    /// ```
    /// use parol::{KTuple, KTuplesBuilder, CompiledTerminal};
    /// use parol::analysis::k_tuple::TerminalMappings;
    /// use parol::analysis::compiled_terminal::EPS;
    ///
    /// {
    ///     let tuples1 = KTuplesBuilder::new().k(1).max_terminal_index(1).terminal_indices(&[&[EPS]]).build().unwrap();
    ///     let tuples2 = KTuplesBuilder::new().k(1).max_terminal_index(1).terminal_indices(&[&[EPS]]).build().unwrap();
    ///     let result = tuples1.k_concat(&tuples2, 1);
    ///     let expected = KTuplesBuilder::new().k(1).max_terminal_index(1).terminal_indices(&[&[EPS]]).build().unwrap();
    ///     assert_eq!(expected, result, "[ε] + [ε] = [ε]");
    /// }
    /// {
    ///     let tuples1 = KTuplesBuilder::new().k(1).max_terminal_index(1).terminal_indices(&[&[1]]).build().unwrap();
    ///     let tuples2 = KTuplesBuilder::new().k(1).max_terminal_index(1).terminal_indices(&[&[EPS]]).build().unwrap();
    ///     let result = tuples1.k_concat(&tuples2, 1);
    ///     let expected = KTuplesBuilder::new().k(1).max_terminal_index(1).terminal_indices(&[&[1]]).build().unwrap();
    ///     assert_eq!(expected, result, "[a] + [ε] = [a]");
    /// }
    /// {
    ///     let tuples1 = KTuplesBuilder::new().k(1).max_terminal_index(1).terminal_indices(&[&[EPS]]).build().unwrap();
    ///     let tuples2 = KTuplesBuilder::new().k(1).max_terminal_index(1).terminal_indices(&[&[1]]).build().unwrap();
    ///     let result = tuples1.k_concat(&tuples2, 1);
    ///     let expected = KTuplesBuilder::new().k(1).max_terminal_index(1).terminal_indices(&[&[1]]).build().unwrap();
    ///     assert_eq!(expected, result, "[ε] + [a] = [a]");
    /// }
    /// {
    ///     let tuples1 = KTuplesBuilder::new().k(1).max_terminal_index(2).terminal_indices(&[&[1]]).build().unwrap();
    ///     let tuples2 = KTuplesBuilder::new().k(1).max_terminal_index(2).terminal_indices(&[&[2]]).build().unwrap();
    ///     let result = tuples1.k_concat(&tuples2, 1);
    ///     let expected = KTuplesBuilder::new().k(1).max_terminal_index(2).terminal_indices(&[&[1]]).build().unwrap();
    ///     assert_eq!(expected, result, "1: [a] + [b] = [a]");
    /// }
    /// {
    ///     let tuples1 = KTuplesBuilder::new().k(2).max_terminal_index(2).terminal_indices(&[&[1]]).build().unwrap();
    ///     let tuples2 = KTuplesBuilder::new().k(2).max_terminal_index(2).terminal_indices(&[&[2]]).build().unwrap();
    ///     let result = tuples1.k_concat(&tuples2, 2);
    ///     let expected = KTuplesBuilder::new().k(2).max_terminal_index(2).terminal_indices(&[&[1, 2]]).build().unwrap();
    ///     assert_eq!(expected, result, "2: [a] + [b] = [ab]");
    /// }
    ///
    /// ```
    pub fn k_concat(mut self, other: &Self, k: usize) -> Self {
        // trace!("KTuples::k_concat {} with {} at k={}", self, other, k);
        if !self.k_complete {
            let (complete, incomplete): (TuplesSet, TuplesSet) =
                self.set.iter().partition(|t| t.is_k_complete());
            self.set = complete;
            self.set.extend(
                incomplete
                    .iter()
                    .flat_map(|t| other.set.iter().map(move |o| t.k_concat(o, k))),
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
            self.k
        )
    }

    /// Set the lookahead size
    pub fn set_k(mut self, k: usize) -> Self {
        self.k = k;
        self.update_completeness();
        self
    }

    /// Returns a sorted representation of self
    pub fn sorted(&self) -> Vec<KTuple> {
        let mut sorted_k_tuples: Vec<KTuple> = self.set.iter().cloned().collect();
        sorted_k_tuples.sort_by(|a, b| a.partial_cmp(b).unwrap());
        sorted_k_tuples
    }

    fn update_completeness(&mut self) {
        self.k_complete = self.set.iter().all(|t| t.is_k_complete());
    }
}

impl Debug for KTuples {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "{{{}}}(k={})",
            self.sorted()
                .iter()
                .map(|e| format!("{e:?}"))
                .collect::<Vec<String>>()
                .join(", "),
            self.k
        )
    }
}

impl Display for KTuples {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "{{{}}}(k={})",
            self.set
                .iter()
                .map(|e| format!("{e}"))
                .collect::<Vec<String>>()
                .join(", "),
            self.k
        )
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;

    use super::*;
    use crate::{MAX_K, analysis::compiled_terminal::EPS};
    use quickcheck::{Arbitrary, Gen};

    // The maximum terminal index used in the k-tuples
    // 0xFFE, 0b11111111110
    // Used to leave room for EPS (0xFFF) which is the same as the bit mask
    const MAX_TERMINAL_INDEX: usize = 4094;

    #[derive(Debug, Clone, Copy)]
    struct SmallTerminalIndex(TerminalIndex);

    impl Arbitrary for SmallTerminalIndex {
        fn arbitrary(_g: &mut Gen) -> SmallTerminalIndex {
            let rand = rand::random::<TerminalIndex>();
            // Generate a random value between 0 and 4093
            SmallTerminalIndex(rand % MAX_TERMINAL_INDEX as TerminalIndex)
        }
    }

    #[derive(Debug, Clone, Copy)]
    struct LookaheadSize(usize);

    impl Arbitrary for LookaheadSize {
        fn arbitrary(_g: &mut Gen) -> LookaheadSize {
            let rand = rand::random::<u32>() as usize;
            // Generate a random value between 1 and MAX_K
            LookaheadSize(rand % MAX_K + 1)
        }
    }

    #[test]
    fn test_k_concat_epsilon() {
        let tuples1 = KTuplesBuilder::new()
            .k(1)
            .max_terminal_index(1)
            .terminal_indices(&[&[EPS]])
            .build()
            .unwrap();
        let tuples2 = KTuplesBuilder::new()
            .k(1)
            .max_terminal_index(1)
            .terminal_indices(&[&[EPS]])
            .build()
            .unwrap();
        let result = tuples1.k_concat(&tuples2, 1);
        let expected = KTuplesBuilder::new()
            .k(1)
            .max_terminal_index(1)
            .terminal_indices(&[&[EPS]])
            .build()
            .unwrap();
        assert_eq!(expected, result, "[ε] + [ε] = [ε]");
    }

    #[test]
    fn test_k_concat_terminal_epsilon() {
        let tuples1 = KTuplesBuilder::new()
            .k(1)
            .max_terminal_index(1)
            .terminal_indices(&[&[1]])
            .build()
            .unwrap();
        let tuples2 = KTuplesBuilder::new()
            .k(1)
            .max_terminal_index(1)
            .terminal_indices(&[&[EPS]])
            .build()
            .unwrap();
        let result = tuples1.k_concat(&tuples2, 1);
        let expected = KTuplesBuilder::new()
            .k(1)
            .max_terminal_index(1)
            .terminal_indices(&[&[1]])
            .build()
            .unwrap();
        assert_eq!(expected, result, "[a] + [ε] = [a]");
    }

    #[test]
    fn test_k_concat_epsilon_terminal() {
        let tuples1 = KTuplesBuilder::new()
            .k(1)
            .max_terminal_index(1)
            .terminal_indices(&[&[EPS]])
            .build()
            .unwrap();
        let tuples2 = KTuplesBuilder::new()
            .k(1)
            .max_terminal_index(1)
            .terminal_indices(&[&[1]])
            .build()
            .unwrap();
        let result = tuples1.k_concat(&tuples2, 1);
        let expected = KTuplesBuilder::new()
            .k(1)
            .max_terminal_index(1)
            .terminal_indices(&[&[1]])
            .build()
            .unwrap();
        assert_eq!(expected, result, "[ε] + [a] = [a]");
    }

    #[test]
    fn test_k_concat_terminal() {
        let tuples1 = KTuplesBuilder::new()
            .k(1)
            .max_terminal_index(1)
            .terminal_indices(&[&[1]])
            .build()
            .unwrap();
        let tuples2 = KTuplesBuilder::new()
            .k(1)
            .max_terminal_index(1)
            .terminal_indices(&[&[2]])
            .build()
            .unwrap();
        let result = tuples1.k_concat(&tuples2, 1);
        let expected = KTuplesBuilder::new()
            .k(1)
            .max_terminal_index(1)
            .terminal_indices(&[&[1]])
            .build()
            .unwrap();
        assert_eq!(expected, result, "1: [a] + [b] = [a]");
    }

    #[test]
    fn test_k_concat_terminal_multiple() {
        let tuples1 = KTuplesBuilder::new()
            .k(2)
            .max_terminal_index(2)
            .terminal_indices(&[&[1]])
            .build()
            .unwrap();
        let tuples2 = KTuplesBuilder::new()
            .k(2)
            .max_terminal_index(2)
            .terminal_indices(&[&[2]])
            .build()
            .unwrap();
        let result = tuples1.k_concat(&tuples2, 2);
        let expected = KTuplesBuilder::new()
            .k(2)
            .max_terminal_index(2)
            .terminal_indices(&[&[1, 2]])
            .build()
            .unwrap();
        assert_eq!(expected, result, "2: [a] + [b] = [ab]");
    }

    #[test]
    fn k_tuples_eq_positive() {
        let tuples1 = KTuplesBuilder::new()
            .k(6)
            .max_terminal_index(4)
            .terminal_indices(&[&[1, 2, 3], &[1, 2, 4]])
            .build();
        let tuples2 = KTuplesBuilder::new()
            .k(6)
            .max_terminal_index(4)
            .terminal_indices(&[&[1, 2, 3], &[1, 2, 4]])
            .build();
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
        let tuples1 = KTuplesBuilder::new()
            .k(6)
            .max_terminal_index(4)
            .terminal_indices(&[&[1, 2, 3], &[1, 2, 4]])
            .build();
        let tuples2 = KTuplesBuilder::new()
            .k(6)
            .max_terminal_index(8)
            .terminal_indices(&[&[5, 6, 7], &[5, 8]])
            .build();
        //     t1    t2
        // ---------------
        //     1     5
        //     |     | \
        //     2     6  8
        //     | \   |
        //     3  4  7
        assert_ne!(tuples1, tuples2);
    }

    // KTuples::insert is commutative regarding Eq
    #[quickcheck]
    fn k_tuples_insert_is_commutative_regarding_eq(
        t1: Vec<SmallTerminalIndex>,
        t2: Vec<SmallTerminalIndex>,
        k: LookaheadSize,
    ) -> bool {
        let t1 = t1.iter().map(|t| t.0).collect::<Vec<TerminalIndex>>();
        let t2 = t2.iter().map(|t| t.0).collect::<Vec<TerminalIndex>>();
        let tuples1 = KTuplesBuilder::new()
            .k(k.0)
            .max_terminal_index(MAX_TERMINAL_INDEX)
            .terminal_indices(&[&t1, &t2])
            .build();
        let tuples2 = KTuplesBuilder::new()
            .k(k.0)
            .max_terminal_index(MAX_TERMINAL_INDEX)
            .terminal_indices(&[&t2, &t1])
            .build();
        tuples1 == tuples2
    }

    // KTuples equality is commutative
    #[quickcheck]
    fn k_tuples_eq_is_commutative(t1: Vec<SmallTerminalIndex>, k: LookaheadSize) -> bool {
        let t1 = t1.iter().map(|t| t.0).collect::<Vec<TerminalIndex>>();
        let tuples1 = KTuplesBuilder::new()
            .k(k.0)
            .max_terminal_index(MAX_TERMINAL_INDEX)
            .terminal_indices(&[&t1])
            .build();
        let tuples2 = KTuplesBuilder::new()
            .k(k.0)
            .max_terminal_index(MAX_TERMINAL_INDEX)
            .terminal_indices(&[&t1])
            .build();
        black_box(tuples1 == tuples2) && black_box(tuples2 == tuples1)
    }
}
