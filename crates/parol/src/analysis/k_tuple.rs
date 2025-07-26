use parol_runtime::TerminalIndex;

use crate::analysis::compiled_terminal::{EPS, INVALID};
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
    /// Check for invalid (i.e. unassigned) terminal
    fn is_inv(&self) -> bool;
}

/// When storing MAX_K terminals in 128 bits, the maximum number of bits used per terminal is 12.
const MAX_BITS: u8 = (std::mem::size_of::<u128>() * 8) as u8 / MAX_K as u8;

/// A collection of terminals
///
/// The terminals are stored in a 128 bit integer where each terminal is stored in a fixed number of
/// bits. The number of bits is determined by the number of terminals to store.
/// The maximum number of terminals when storing MAX_K terminals in 128 bits is:
/// 128 / MAX_K = 128 / 10 = 12.8 => 12 bits
/// The maximum number of terminals that can be stored is 2^12 = 4096.
/// The maximum value of the bit count is therefore 12 and can safely be stored in four bits.
/// We store a mask to more easily extract the terminals from the 128 bits unsigned integer.
/// The mask to extract single terminals from the 128 bit unsigned integer is calculated as
/// 2^bits - 1 that is equivalent to the expression !(!0u128 << bits) at runtime.
///
/// Since we use only 120 bits to store the terminals, we have 8 bits left. We use the 8 bits to
/// store the index of the next insertion as well as the bit count used to calculate the mask.
/// Therefore we split the highest 8 bits of the 128 bits unsigned integer as follows:
/// - The higher 4 bits are used to store the number of bits used per terminal
/// - The lower 4 bits are used to store the index of the next insertion
#[derive(Clone, Copy, Default, Hash, Eq, PartialEq)]
pub struct Terminals {
    t: u128,
}

impl Terminals {
    /// Creates a new item
    /// ```
    /// use parol::analysis::k_tuple::Terminals;
    /// use parol::analysis::compiled_terminal::CompiledTerminal;
    /// let t = Terminals::new(1);
    /// assert!(t.is_empty());
    /// assert_eq!(0, t.len(), "len");
    /// assert_eq!(0, t.k_len(5), "k_len");
    /// assert_eq!(None, t.get(0));
    /// assert_eq!(None, t.get(9));
    /// ```
    pub fn new(max_terminal_index: usize) -> Self {
        // max_terminal_index + 1: we also need to store EPS
        let bits = (max_terminal_index + 1).ilog2() as u8 + 1;
        if bits > MAX_BITS {
            panic!(
                "The number of bits required to store {} terminals is {} which is greater than the maximum of {}",
                max_terminal_index + 1,
                bits,
                MAX_BITS
            );
        }
        let mut this = Self { t: 0 };
        this.set_bits(bits);
        this
    }

    /// Creates a new item with epsilon semantic
    /// ```
    /// use parol::analysis::k_tuple::{Terminals, TerminalMappings};
    /// use parol::analysis::compiled_terminal::CompiledTerminal;
    /// let t = Terminals::eps(1);
    /// assert!(!t.is_empty());
    /// assert_eq!(1, t.len(), "len");
    /// assert_eq!(1, t.k_len(5), "k_len");
    /// assert_eq!(Some(CompiledTerminal::eps()), t.get(0));
    /// assert_eq!(None, t.get(1));
    /// assert_eq!(None, t.get(9));
    /// ```
    pub fn eps(max_terminal_index: usize) -> Terminals {
        let mut t = Self::new(max_terminal_index);
        t.set(0, CompiledTerminal(EPS));
        t.set_next_index(1);
        t
    }

    /// Creates a new item with end (EOI) semantic
    /// Such a terminal can't be extended, i.e. you can't append more terminals
    /// ```
    /// use parol::analysis::k_tuple::{Terminals, TerminalMappings};
    /// use parol::analysis::compiled_terminal::CompiledTerminal;
    /// let t = Terminals::end(1);
    /// assert!(!t.is_empty());
    /// assert_eq!(1, t.len());
    /// assert_eq!(1, t.k_len(5));
    /// assert_eq!(Some(CompiledTerminal::end()), t.get(0));
    /// assert_eq!(None, t.get(1));
    /// assert_eq!(None, t.get(9));
    /// ```
    pub fn end(max_terminal_index: usize) -> Terminals {
        let mut t = Self::new(max_terminal_index);
        // t.t = 0; // EOI as u128 & t.mask;
        t.set_next_index(1);
        t
    }

    ///
    /// Creates a new object with maximum k length from another object
    ///
    #[must_use]
    pub fn of(k: usize, other: Self) -> Self {
        let bits = other.bits();
        let mask = other.mask();
        let i = other.k_len(k) as u8;
        let mut copy_mask = 0u128;
        (0..i).for_each(|_| {
            copy_mask <<= bits as usize;
            copy_mask |= mask;
        });
        let t = other.t & copy_mask;
        let mut t = Self { t };
        t.set_bits(bits);
        t.set_next_index(i);
        t
    }

    /// Returns the length of the collection
    #[inline]
    pub fn len(&self) -> usize {
        self.next_index() as usize
    }
    /// Checks if the collection is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.next_index() == 0
    }

    /// Returns the index of the next insertion
    /// The highest 8 bits of t are used to store the index of the next insertion in it's lowest 4
    /// bits.
    #[inline]
    pub fn next_index(&self) -> u8 {
        ((self.t & 0x0F00_0000_0000_0000_0000_0000_0000_0000) >> 120) as u8
    }

    /// Sets the index of the next insertion
    #[inline]
    fn set_next_index(&mut self, i: u8) {
        self.t &= 0xF0FF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF;
        self.t |= (i as u128) << 120;
    }

    /// Increments the index of the next insertion
    #[inline]
    pub fn inc_index(&mut self) {
        let i = self.next_index() + 1;
        self.t &= 0xF0FF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF;
        self.t |= (i as u128) << 120;
    }

    /// Returns the bits used per terminal
    /// The highest 8 bits of t are used to store the number of bits used per terminal in it's highest
    /// 4 bits.
    /// ```
    /// use parol::analysis::k_tuple::Terminals;
    /// let t = Terminals::eps(1);
    /// assert_eq!(2, t.bits());
    /// ```
    #[inline]
    pub fn bits(&self) -> u8 {
        ((self.t & 0xF000_0000_0000_0000_0000_0000_0000_0000) >> 124) as u8
    }

    /// Sets the number of bits used per terminal
    #[inline]
    pub fn set_bits(&mut self, bits: u8) {
        self.t &= 0x0FFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF;
        self.t |= (bits as u128) << 124;
        debug_assert_ne!(self.bits(), 0, "Bits must not be 0");
    }

    /// Returns the mask used to extract the terminal at position i
    /// The mask is calculated as 2^bits - 1 that is equivalent to the expression !(!0u128 << bits).
    #[inline]
    pub fn mask(&self) -> u128 {
        !(!0u128 << self.bits())
    }

    #[must_use]
    fn last(&self) -> Option<CompiledTerminal> {
        if self.is_empty() {
            None
        } else {
            self.get(self.next_index() as usize - 1)
        }
    }

    /// Checks if the collection is k-complete, i.e. no terminals can be added
    /// ```
    /// use parol::analysis::k_tuple::Terminals;
    /// let t = Terminals::end(1);
    /// assert!(t.is_k_complete(5));
    /// ```
    pub fn is_k_complete(&self, k: usize) -> bool {
        !self.is_eps() && (self.len() >= k || self.last().is_some_and(|t| t.is_end()))
    }

    /// Returns the k-length, i.e. the number of symbols that contributes to lookahead sizes
    #[must_use]
    pub fn k_len(&self, k: usize) -> usize {
        std::cmp::min(self.len(), k)
    }

    /// Clears the collection
    pub fn clear(&mut self) {
        let bits = self.bits();
        self.t = 0;
        self.set_bits(bits);
        debug_assert_ne!(self.bits(), 0, "Bits must not be 0");
    }

    /// Concatenates two collections with respect to the rules of k-concatenation
    /// ```
    /// use parol::analysis::k_tuple::{Terminals, TerminalMappings};
    /// use parol::analysis::compiled_terminal::CompiledTerminal;
    /// let t1 = Terminals::eps(1);
    /// let t2 = Terminals::end(1);
    /// let t = t1.k_concat(&t2, 5);
    /// assert!(t.is_k_complete(5));
    /// assert_eq!(1, t.len());
    /// assert_eq!(Some(CompiledTerminal::end()), t.get(0));
    /// let t = t2.k_concat(&t1, 5);
    /// assert!(t.is_k_complete(5));
    /// assert_eq!(1, t.len());
    /// assert_eq!(Some(CompiledTerminal::end()), t.get(0));
    /// let mut t1 = Terminals::new(6);
    /// t1.extend([1, 2, 3].iter().cloned());
    /// let mut t2 = Terminals::new(6);
    /// t2.extend([4, 5, 6].iter().cloned());
    /// let t = t1.k_concat(&t2, 5);
    /// assert!(t.is_k_complete(5));
    /// assert_eq!(5, t.len());
    /// assert_eq!(Some(CompiledTerminal(1)), t.get(0));
    /// assert_eq!(Some(CompiledTerminal(2)), t.get(1));
    /// assert_eq!(Some(CompiledTerminal(3)), t.get(2));
    /// assert_eq!(Some(CompiledTerminal(4)), t.get(3));
    /// assert_eq!(Some(CompiledTerminal(5)), t.get(4));
    /// assert_eq!(None, t.get(5));
    /// ```
    pub fn k_concat(mut self, other: &Self, k: usize) -> Self {
        debug_assert!(
            other.bits() == self.bits(),
            "Bits must be the same, self:({self:?}) != other:({other:?})"
        );
        debug_assert_ne!(self.bits(), 0, "Bits must not be 0");

        if other.is_eps() || other.is_empty() {
            // w + ε = w
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
        let other_len = other.k_len(k);
        let to_take = std::cmp::min(k - my_k_len, other_len);
        if to_take == 0 {
            // We can't take any more terminals
            debug_assert!(
                false,
                "to_take == 0, self:({self:?}), other:({other:?})"
            );
            return self;
        };

        let bits = self.bits();

        // Mask out the other value with a length of to_take
        // Shift the other value to the left by the length of my_k_len
        let value =
            (other.t & !(!0u128 << (to_take * bits as usize))) << (my_k_len * bits as usize);
        // Add the other value to self
        self.t |= value;
        self.set_next_index((my_k_len + to_take) as u8);
        self.set_bits(bits);
        self
    }

    /// Adds a new terminal to self if max size is not reached yet and if last is not EOI
    pub fn push(&mut self, t: CompiledTerminal) -> Result<(), String> {
        if self.next_index() >= MAX_K as u8 {
            return Err("Maximum number of terminals reached".to_owned());
        }
        if matches!(self.last(), Some(CompiledTerminal(EOI))) {
            return Ok(());
        }
        debug_assert_ne!(t.0, INVALID, "Invalid terminal");
        self.set(self.next_index().into(), t);
        self.inc_index();
        Ok(())
    }

    /// Checks if self is an Epsilon
    #[inline]
    pub fn is_eps(&self) -> bool {
        if self.next_index() != 1 {
            return false;
        }
        let mask = self.mask();
        (self.t & mask) == mask
    }

    /// Creates an iterator over the terminals
    pub fn iter(&self) -> TermIt {
        TermIt::new(*self)
    }

    /// Returns the terminal at position i
    pub fn get(&self, i: usize) -> Option<CompiledTerminal> {
        if i < self.next_index() as usize {
            let mut terminal_index = (self.t >> (i * self.bits() as usize)) & self.mask();
            if terminal_index == self.mask() {
                // Epsilon is defined as 0xFFFF and stored as a value identical to self.mask, i.e. all
                // bits set to 1. We need to convert it back to 0xFFFF.
                terminal_index = EPS as u128;
            }
            Some(CompiledTerminal(terminal_index as TerminalIndex))
        } else {
            None
        }
    }

    /// Sets the terminal at position i
    pub fn set(&mut self, i: usize, t: CompiledTerminal) {
        let terminal_mask = self.mask();
        debug_assert!(
            t.0 <= terminal_mask as TerminalIndex || t.0 == EPS as TerminalIndex,
            "Terminal index {} out of range",
            t.0
        );
        debug_assert_ne!(t.0, INVALID, "Invalid terminal");
        let bits = self.bits() as usize;
        let v = (t.0 as u128 & terminal_mask) << (i * bits);
        let mask = !(terminal_mask << (i * bits));
        self.t &= mask;
        self.t |= v;
    }
}

impl Ord for Terminals {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.next_index().cmp(&other.next_index()) {
            std::cmp::Ordering::Less => std::cmp::Ordering::Less,
            std::cmp::Ordering::Equal => {
                <&Self as Into<u128>>::into(self).cmp(&<&Self as Into<u128>>::into(other))
            }
            std::cmp::Ordering::Greater => std::cmp::Ordering::Greater,
        }
    }
}

impl PartialOrd for Terminals {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Display for Terminals {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "[{}(i{})]",
            (0..self.next_index())
                .map(|i| format!("{}", self.get(i as usize).unwrap()))
                .collect::<Vec<String>>()
                .join(", "),
            self.next_index(),
        )
    }
}

// Used for comparison in implementation of Ord
impl From<&Terminals> for u128 {
    fn from(t: &Terminals) -> Self {
        // Mask out the unused bits although it should not be necessary
        t.t & !(!0u128 << (t.next_index() * t.bits()) as usize)
    }
}

impl Extend<CompiledTerminal> for Terminals {
    fn extend<I: IntoIterator<Item = CompiledTerminal>>(&mut self, iter: I) {
        for t in iter {
            let _ = self.push(t);
        }
    }
}

impl Extend<TerminalIndex> for Terminals {
    fn extend<I: IntoIterator<Item = TerminalIndex>>(&mut self, iter: I) {
        for t in iter {
            let _ = self.push(CompiledTerminal(t));
        }
    }
}

impl Debug for Terminals {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "0b{:b}, i:{}, bits:0x{:x}, mask:0x{:x}",
            self.t,
            self.next_index(),
            self.bits(),
            self.mask()
        )
    }
}

/// Iterator for Terminals
/// It returns the terminal indices
#[derive(Debug)]
pub struct TermIt {
    /// A copy of the Terminals object.
    /// During iteration, the member t is shifted to the right by bits and the terminal is extracted
    /// by masking the lowest bits.
    t: Terminals,
    /// The current index
    i: usize,
    /// The number of bits used per terminal
    bits: usize,
    /// The mask to extract the terminal
    mask: u128,
    /// The number of terminals in the collection
    len: usize,
}

impl TermIt {
    fn new(t: Terminals) -> Self {
        Self {
            t,
            i: 0,
            bits: t.bits() as usize,
            mask: t.mask(),
            len: t.next_index() as usize,
        }
    }
}

impl Iterator for TermIt {
    type Item = TerminalIndex;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i < self.len {
            let t = self.t.t & self.mask;
            // Prepare for the next iteration
            self.t.t >>= self.bits;
            self.i += 1;

            if t == self.mask {
                // Epsilon is defined as 0xFFFF and stored as a value identical to self.mask, i.e.
                // all bits set to 1. We need to convert it back to 0xFFFF.
                Some(EPS)
            } else {
                Some(t as TerminalIndex)
            }
        } else {
            None
        }
    }
}

/// Terminal string with support for k-completeness
#[derive(Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
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
        let mut inner = match self {
            Self::Incomplete(t) | Self::Complete(t) => t,
        };
        inner.clear();

        Self::Incomplete(inner)
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

    /// Push a new terminal
    pub fn push(&mut self, t: CompiledTerminal, k: usize) -> Result<(), String> {
        match self {
            Self::Incomplete(v) => {
                v.push(t)?;
                if v.is_k_complete(k) {
                    *self = Self::Complete(*v);
                }
            }
            Self::Complete(_) => {}
        }
        Ok(())
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

impl Display for TerminalString {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Self::Incomplete(v) => write!(f, "Incomplete({v})"),
            Self::Complete(v) => write!(f, "Complete  ({v})"),
        }
    }
}

impl Debug for TerminalString {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Self::Incomplete(v) => write!(f, "Incomplete({v:?})"),
            Self::Complete(v) => write!(f, "Complete  ({v:?})"),
        }
    }
}

/// A builder for KTuple
#[derive(Clone, Default)]
pub struct KTupleBuilder<'a> {
    k: Option<usize>,
    max_terminal_index: Option<usize>,
    k_tuple: Option<&'a KTuple>,
    terminal_string: Option<&'a [TerminalIndex]>,
}

impl<'a> KTupleBuilder<'a> {
    /// Creates a new builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the lookahead size
    pub fn k(mut self, k: usize) -> Self {
        self.k = Some(k);
        self
    }

    /// Sets the maximum terminal index
    pub fn max_terminal_index(mut self, max_terminal_index: usize) -> Self {
        self.max_terminal_index = Some(max_terminal_index);
        self
    }

    /// Sets the k-tuple to be used during construction
    pub fn k_tuple(mut self, k_tuple: &'a KTuple) -> Self {
        self.k_tuple = Some(k_tuple);
        self
    }

    /// Sets the terminal string to be used during construction
    pub fn terminal_string(mut self, terminal_string: &'a [TerminalIndex]) -> Self {
        self.terminal_string = Some(terminal_string);
        self
    }

    /// Builds a new KTuple
    pub fn build(self) -> Result<KTuple, String> {
        if self.k.is_none() {
            return Err("k is not set".to_owned());
        }
        if self.max_terminal_index.is_none() {
            return Err("max_terminal_index is not set".to_owned());
        }
        let k = self.k.unwrap();
        let max_terminal_index = self.max_terminal_index.unwrap_or(0);
        if let Some(k_tuple) = self.k_tuple {
            let mut terminals = Terminals::new(max_terminal_index);
            for t in k_tuple.terminals.inner().iter().take(k) {
                terminals.push(CompiledTerminal(t))?;
            }
            let terminals = if terminals.is_k_complete(k) {
                TerminalString::Complete(terminals)
            } else {
                TerminalString::Incomplete(terminals)
            };
            Ok(KTuple {
                terminals,
                k: std::cmp::min(k, MAX_K),
            })
        } else if let Some(terminal_string) = self.terminal_string {
            let mut terminals = Terminals::new(max_terminal_index);
            for t in terminal_string.iter().take(k) {
                terminals.push(CompiledTerminal(*t))?;
            }
            let terminals = if terminals.is_k_complete(k) {
                TerminalString::Complete(terminals)
            } else {
                TerminalString::Incomplete(terminals)
            };
            Ok(KTuple {
                terminals,
                k: std::cmp::min(k, MAX_K),
            })
        } else {
            Err("k_tuple or terminal_string must be set".to_owned())
        }
    }

    ///
    /// Creates a new ε object
    ///
    pub fn eps(self) -> Result<KTuple, String> {
        if self.k.is_none() {
            return Err("k is not set".to_owned());
        }
        if self.max_terminal_index.is_none() {
            return Err("max_terminal_index is not set".to_owned());
        }
        let k = self.k.unwrap();
        let terminals =
            TerminalString::Incomplete(Terminals::eps(self.max_terminal_index.unwrap()));
        Ok(KTuple {
            terminals,
            k: std::cmp::min(k, MAX_K),
        })
    }
    ///
    /// Creates a new End object
    ///
    pub fn end(self) -> Result<KTuple, String> {
        if self.k.is_none() {
            return Err("k is not set".to_owned());
        }
        if self.max_terminal_index.is_none() {
            return Err("max_terminal_index is not set".to_owned());
        }
        let k = self.k.unwrap();
        let terminals = TerminalString::Complete(Terminals::end(self.max_terminal_index.unwrap()));
        Ok(KTuple {
            terminals,
            k: std::cmp::min(k, MAX_K),
        })
    }
}

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
///
/// Terminal symbol string type
///
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct KTuple {
    /// The sequence of terminals
    terminals: TerminalString,
    /// The lookahead size
    k: usize,
}

impl KTuple {
    /// Used for debugging only
    pub fn with_terminal_indices(self, terms: &[TerminalIndex]) -> Self {
        let k = self.k;
        let mut terminals = match self.terminals {
            TerminalString::Incomplete(s) => s,
            TerminalString::Complete(s) => s,
        };

        terms.iter().take(k).enumerate().for_each(|(i, t)| {
            terminals.set(i, CompiledTerminal(*t));
            terminals.inc_index();
        });

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
    pub fn from_slice(others: &[CompiledTerminal], k: usize, max_terminal_index: usize) -> Self {
        let mut terminals = Terminals::new(max_terminal_index);
        terminals.extend(others.iter().take(k).cloned());
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

    /// Adds a new terminal to self while consuming self
    pub fn push(&mut self, t: CompiledTerminal) -> Result<(), String> {
        self.terminals.push(t, self.k)
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
                .iter()
                .map(|t| match t {
                    EOI => "$".to_owned(),
                    NEW_LINE => "NewLine".to_owned(),
                    WHITESPACE => "WhiteSpace".to_owned(),
                    LINE_COMMENT => "LineComment".to_owned(),
                    BLOCK_COMMENT => "BlockComment".to_owned(),
                    EPS => "\u{03B5}".to_owned(),
                    _ => terminals[t as usize].to_string(),
                })
                .collect::<Vec<String>>()
                .join(", ")
        )
    }

    /// Returns the k value
    #[inline]
    pub fn k(&self) -> usize {
        self.k
    }

    /// Returns the terminals
    #[inline]
    pub fn terminals(&self) -> &Terminals {
        self.terminals.inner()
    }
}

impl Debug for KTuple {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "[{:?}(i{})](k{})",
            self.terminals,
            self.terminals.inner().next_index(),
            self.k
        )
    }
}

impl Display for KTuple {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "[{}(i{})](k{})",
            self.terminals,
            self.terminals.inner().next_index(),
            self.k
        )
    }
}

impl Hash for KTuple {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let self_inner = self.terminals.inner();
        self_inner.t.hash(state)
    }
}

impl Extend<CompiledTerminal> for KTuple {
    fn extend<I: IntoIterator<Item = CompiledTerminal>>(&mut self, iter: I) {
        if !self.terminals.is_k_complete() {
            for t in iter.into_iter().take(self.k - self.len()) {
                let _ = self.push(t);
            }
        }
    }
}

impl Extend<TerminalIndex> for KTuple {
    fn extend<I: IntoIterator<Item = TerminalIndex>>(&mut self, iter: I) {
        if !self.terminals.is_k_complete() {
            for t in iter.into_iter().take(self.k - self.len()) {
                let _ = self.push(CompiledTerminal(t));
            }
        }
    }
}

#[cfg(test)]
mod test {
    use parol_runtime::TerminalIndex;

    use super::{TerminalString, Terminals};
    use crate::{
        CompiledTerminal, KTuple, MAX_K,
        analysis::k_tuple::{EOI, KTupleBuilder},
    };

    fn term(terminals: &[TerminalIndex], k: usize, max_terminal_index: usize) -> Terminals {
        debug_assert!(k <= MAX_K);
        let mut t = Terminals::new(max_terminal_index);
        t.extend(terminals.iter().map(|t| CompiledTerminal(*t)));
        t
    }

    #[test]
    fn test_terminals_bits() {
        let terminals = Terminals::new(6);
        assert_eq!(3, terminals.bits());
    }

    #[test]
    fn test_terminals_set_bits() {
        let mut terminals = Terminals::new(6);
        terminals.set_bits(0b1010);
        assert_eq!(0b1010, terminals.bits());
        terminals.set_bits(0b1100);
        assert_eq!(0b1100, terminals.bits());
    }

    #[test]
    fn test_terminals_mask() {
        let terminals = Terminals::new(6);
        assert_eq!(terminals.mask(), 0b111);
    }

    #[test]
    fn test_terminals_next_index() {
        let mut terminals = Terminals::new(6);
        assert_eq!(0, terminals.next_index());
        terminals.set_next_index(3);
        assert_eq!(3, terminals.next_index());
    }

    #[test]
    fn test_terminals_set_next_index() {
        let mut terminals = Terminals::new(6);
        assert_eq!(0, terminals.next_index());
        terminals.set_next_index(3);
        assert_eq!(3, terminals.next_index());
        terminals.set_next_index(5);
        assert_eq!(5, terminals.next_index());
    }

    #[test]
    fn test_terminals_inc_index() {
        let mut terminals = Terminals::new(6);
        assert_eq!(0, terminals.next_index());
        terminals.inc_index();
        assert_eq!(1, terminals.next_index());
        terminals.inc_index();
        assert_eq!(2, terminals.next_index());
    }

    #[test]
    fn check_terminals_k_concat() {
        // let t1 = Terminals::eps(1);
        // let t2 = Terminals::end(1);
        // let t = t1.k_concat(&t2, 5);
        // assert!(t.is_k_complete(5));
        // assert_eq!(1, t.len());
        // assert_eq!(Some(CompiledTerminal(EOI)), t.get(0));
        // let t = t2.k_concat(&t1, 5);
        // assert!(t.is_k_complete(5));
        // assert_eq!(1, t.len());
        // assert_eq!(Some(CompiledTerminal(EOI)), t.get(0));
        let mut t1 = Terminals::new(6);
        t1.extend([1, 2, 3].iter().cloned());
        let mut t2 = Terminals::new(6);
        t2.extend([4, 5, 6].iter().cloned());
        let t = t1.k_concat(&t2, 5);
        assert!(t.is_k_complete(5));
        assert_eq!(5, t.len());
        assert_eq!(Some(CompiledTerminal(1)), t.get(0));
        assert_eq!(Some(CompiledTerminal(2)), t.get(1));
        assert_eq!(Some(CompiledTerminal(3)), t.get(2));
        assert_eq!(Some(CompiledTerminal(4)), t.get(3));
        assert_eq!(Some(CompiledTerminal(5)), t.get(4));
        assert_eq!(None, t.get(5));
    }

    #[test]
    fn check_with_terminal_indices() {
        {
            let k_tuple = KTupleBuilder::new()
                .k(1)
                .max_terminal_index(1)
                .terminal_string(&[1])
                .build()
                .unwrap();
            let t = term(&[1], 1, 1);
            let expected = KTuple {
                terminals: TerminalString::Complete(t),
                k: 1,
            };
            assert_eq!(None, t.get(1));
            assert_eq!(None, t.get(MAX_K - 1));
            assert_eq!(expected, k_tuple, "[1]");
        }
        {
            let k_tuple = KTupleBuilder::new()
                .k(MAX_K)
                .max_terminal_index(10)
                .terminal_string(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10])
                .build()
                .unwrap();
            let t = term(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10], MAX_K, 10);
            let expected = KTuple {
                terminals: TerminalString::Complete(t),
                k: MAX_K,
            };
            assert_eq!(Some(CompiledTerminal(1)), t.get(0));
            assert_eq!(Some(CompiledTerminal(10)), t.get(MAX_K - 1));
            assert_eq!(expected, k_tuple, "[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]");
        }
        {
            let k_tuple = KTupleBuilder::new()
                .k(5)
                .max_terminal_index(10)
                .terminal_string(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10])
                .build()
                .unwrap();
            let t = term(&[1, 2, 3, 4, 5], 5, 10);
            let expected = KTuple {
                terminals: TerminalString::Complete(t),
                k: 5,
            };
            assert_eq!(Some(CompiledTerminal(1)), t.get(0));
            assert_eq!(Some(CompiledTerminal(5)), t.get(4));
            assert_eq!(None, t.get(5));
            assert_eq!(expected, k_tuple, "[1, 2, 3, 4, 5]");
        }
    }

    #[test]
    fn check_from_slice() {
        {
            let k_tuple = KTuple::from_slice(&[CompiledTerminal(1)], 1, 1);
            let t = term(&[1], 1, 1);
            let expected = KTuple {
                terminals: TerminalString::Complete(t),
                k: 1,
            };
            assert_eq!(None, t.get(1));
            assert_eq!(None, t.get(MAX_K - 1));
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
                ],
                MAX_K,
                10,
            );
            let t = term(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10], MAX_K, 10);
            let expected = KTuple {
                terminals: TerminalString::Complete(t),
                k: MAX_K,
            };
            assert_eq!(Some(CompiledTerminal(1)), t.get(0));
            assert_eq!(Some(CompiledTerminal(10)), t.get(MAX_K - 1));
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
                ],
                5,
                10,
            );
            let t = term(&[1, 2, 3, 4, 5], 5, 10);
            let expected = KTuple {
                terminals: TerminalString::Complete(t),
                k: 5,
            };
            assert_eq!(Some(CompiledTerminal(1)), t.get(0));
            assert_eq!(Some(CompiledTerminal(5)), t.get(4));
            assert_eq!(None, t.get(5));
            assert_eq!(expected, k_tuple, "[1, 2, 3, 4, 5]");
        }
    }

    #[test]
    fn check_k_tuple_of() {
        {
            let k = 1;
            let mut t = Terminals::new(1);
            t.extend([1]);
            let k_tuple = KTuple::of(t, k);
            let mut t2 = Terminals::new(1);
            t2.extend([1]);
            let expected = KTuple {
                terminals: TerminalString::Complete(t2),
                k,
            };
            assert_eq!(None, t.get(1));
            assert_eq!(None, t.get(MAX_K - 1));
            assert_eq!(expected, k_tuple, "[1]");
        }
        {
            let k = MAX_K;
            let mut t = Terminals::new(11);
            t.extend([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
            let k_tuple = KTuple::of(t, k);
            assert_eq!(MAX_K, k_tuple.len());
            let mut t2 = Terminals::new(11);
            t2.extend([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
            let expected = KTuple {
                terminals: TerminalString::Complete(t2),
                k,
            };
            assert_eq!(Some(CompiledTerminal(1)), t.get(0));
            assert_eq!(Some(CompiledTerminal(10)), t.get(MAX_K - 1));
            assert_eq!(expected, k_tuple, "[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]");
        }
        {
            let k = 5;
            let mut t = Terminals::new(11);
            t.extend([1, 2, 3, 4, 5]);

            let k_tuple = KTuple::of(t, k);
            let mut t2 = Terminals::new(11);
            t2.extend([1, 2, 3, 4, 5]);
            let expected = KTuple {
                terminals: TerminalString::Complete(t2),
                k,
            };
            assert_eq!(Some(CompiledTerminal(1)), t.get(0));
            assert_eq!(Some(CompiledTerminal(5)), t.get(4));
            assert_eq!(None, t.get(5));
            assert_eq!(expected, k_tuple, "[1, 2, 3, 4, 5]");
        }
    }

    #[test]
    fn check_k_concat() {
        {
            let tuple1 = KTupleBuilder::new()
                .k(1)
                .max_terminal_index(1)
                .eps()
                .unwrap();
            let tuple2 = KTupleBuilder::new()
                .k(1)
                .max_terminal_index(1)
                .eps()
                .unwrap();
            let result = tuple1.k_concat(&tuple2, 1);
            let expected = KTupleBuilder::new()
                .k(1)
                .max_terminal_index(1)
                .eps()
                .unwrap();
            assert_eq!(expected, result, "1: [ε] + [ε] = [ε]");
        }
        {
            let tuple1 = KTupleBuilder::new()
                .k(1)
                .max_terminal_index(1)
                .terminal_string(&[1])
                .build()
                .unwrap();
            let tuple2 = KTupleBuilder::new()
                .k(1)
                .max_terminal_index(1)
                .eps()
                .unwrap();
            let result = tuple1.k_concat(&tuple2, 1);
            let expected = KTupleBuilder::new()
                .k(1)
                .max_terminal_index(1)
                .terminal_string(&[1])
                .build()
                .unwrap();
            assert_eq!(expected, result, "1: [a] + [ε] = [a]");
        }
        {
            let tuple1 = KTupleBuilder::new()
                .k(1)
                .max_terminal_index(1)
                .eps()
                .unwrap();
            let tuple2 = KTupleBuilder::new()
                .k(1)
                .max_terminal_index(1)
                .terminal_string(&[1])
                .build()
                .unwrap();
            let result = tuple1.k_concat(&tuple2, 1);
            let expected = KTupleBuilder::new()
                .k(1)
                .max_terminal_index(1)
                .terminal_string(&[1])
                .build()
                .unwrap();
            assert_eq!(expected, result, "1: [ε] + [a] = [a]");
        }
        {
            let tuple1 = KTupleBuilder::new()
                .k(2)
                .max_terminal_index(2)
                .terminal_string(&[1])
                .build()
                .unwrap();
            let tuple2 = KTupleBuilder::new()
                .k(2)
                .max_terminal_index(2)
                .terminal_string(&[2])
                .build()
                .unwrap();
            let result = tuple1.k_concat(&tuple2, 2);
            let expected = KTupleBuilder::new()
                .k(2)
                .max_terminal_index(1)
                .terminal_string(&[1, 2])
                .build()
                .unwrap();
            assert_eq!(expected, result, "2: [a] + [b] = [ab]");
        }
    }

    #[test]
    fn check_term() {
        {
            let terminals = Terminals::new(4);
            assert_eq!(0, terminals.k_len(0));
            assert_eq!(0, terminals.k_len(1));
            assert_eq!(0, terminals.k_len(2));

            assert!(terminals.is_k_complete(0));
            assert!(!terminals.is_k_complete(1));
            assert!(!terminals.is_k_complete(2));
            assert!(!terminals.is_k_complete(3));
        }
        {
            let terminals = term(&[1], 1, 4);
            assert_eq!(0, terminals.k_len(0));
            assert_eq!(1, terminals.k_len(1));
            assert_eq!(1, terminals.k_len(2));

            assert!(terminals.is_k_complete(0));
            assert!(terminals.is_k_complete(1));
            assert!(!terminals.is_k_complete(2));
            assert!(!terminals.is_k_complete(3));
        }
        {
            let terminals = term(&[1, 2], 2, 4);
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
            let terminals = term(&[1, EOI], 2, 4);
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
                1,
            );
            assert_eq!(0, terminals.k_len(0));
            assert_eq!(1, terminals.k_len(1));
            assert_eq!(2, terminals.k_len(2));
            assert_eq!(2, terminals.k_len(3));

            assert!(terminals.is_k_complete(0));
            assert!(terminals.is_k_complete(1));
            assert!(terminals.is_k_complete(2));
            assert!(terminals.is_k_complete(3));

            let terminals2 = term(&[3], 1, 1);
            let result = terminals.k_concat(&terminals2, 3);
            let expected = term(&[1, EOI, 1], 3, 1);
            assert_eq!(expected, result);
        }
    }

    #[test]
    fn test_iteration_of_terminals() {
        let terminals = term(&[1, 2, 3, 4, 5], 5, 5);
        let mut iter = terminals.iter();
        assert_eq!(Some(1), iter.next());
        assert_eq!(Some(2), iter.next());
        assert_eq!(Some(3), iter.next());
        assert_eq!(Some(4), iter.next());
        assert_eq!(Some(5), iter.next());
        assert_eq!(None, iter.next());
    }
}
