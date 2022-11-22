use std::fmt::Debug;
use std::fmt::{Display, Error, Formatter};
use std::hash::Hash;

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
///
/// Position within a Cfg
/// Immutable struct
///
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Pos {
    /// Index of the production
    pr_index: usize,

    /// Index of the symbol within the production
    /// 0: Always the index of the left-hand-side of the production
    /// >0: Index of a symbol on the hight-hand-side of the production
    sy_index: usize,
}

impl Pos {
    /// Creates an immutable Pos instance
    pub fn new(pr_index: usize, sy_index: usize) -> Self {
        Self { pr_index, sy_index }
    }

    /// Returns the production index
    pub fn pr_index(&self) -> usize {
        self.pr_index
    }

    /// Returns the symbol index
    pub fn sy_index(&self) -> usize {
        self.sy_index
    }

    /// Returns the members as a tuple
    pub fn as_tuple(&self) -> (usize, usize) {
        (self.pr_index, self.sy_index)
    }
}

impl From<(usize, usize)> for Pos {
    fn from(p: (usize, usize)) -> Self {
        Self::new(p.0, p.1)
    }
}

impl Display for Pos {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "({},{})", self.pr_index, self.sy_index)
    }
}
