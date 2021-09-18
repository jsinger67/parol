use std::fmt::Debug;
use std::fmt::{Display, Error, Formatter};
use std::hash::Hash;

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
    /// Panics on invalid index values.
    pub fn new(pr_index: usize, sy_index: usize) -> Self {
        Self { pr_index, sy_index }
    }

    pub fn pr_index(&self) -> usize {
        self.pr_index
    }

    pub fn sy_index(&self) -> usize {
        self.sy_index
    }

    pub fn as_tuple(&self) -> (usize, usize) {
        (self.pr_index, self.sy_index)
    }

    pub fn no_pos() -> Self {
        Self {
            pr_index: usize::MAX,
            sy_index: usize::MAX,
        }
    }
    pub fn is_no_pos(&self) -> bool {
        self.pr_index == usize::MAX && self.sy_index == usize::MAX
    }

    pub fn next_pos(&self) -> Self {
        Self {
            pr_index: self.pr_index,
            sy_index: self.sy_index + 1,
        }
    }

    pub fn next_pos_mut(&mut self) {
        self.sy_index += 1;
    }

    ///
    /// Returns true if self is the immediate successor of the given Pos.
    /// The relation holds only within a single production, i.e. the pr_index must be equal.
    ///
    /// ```
    /// use parol::Pos;
    ///
    /// let this : Pos = (0, 2).into();
    /// let that : Pos = (0, 1).into();
    /// assert!(this.follows(&that));
    /// assert!(!that.follows(&this));
    ///
    /// let this : Pos = (0, 1).into();
    /// let that : Pos = (0, 1).into();
    /// assert!(!this.follows(&that));
    /// assert!(!that.follows(&this));
    ///
    /// let this : Pos = (0, 2).into();
    /// let that : Pos = (1, 1).into();
    /// assert!(!this.preceds(&that));
    /// assert!(!that.preceds(&this));
    ///
    /// let this : Pos = (0, 1).into();
    /// let that : Pos = (0, 2).into();
    /// assert!(!this.follows(&that));
    /// assert!(that.follows(&this));
    /// ```
    ///
    pub fn follows(&self, that: &Self) -> bool {
        self.pr_index == that.pr_index && self.sy_index == that.sy_index + 1
    }

    ///
    /// Returns true if self is the immediate predecessor of the given Pos.
    /// The relation holds only within a single production, i.e. the pr_index must be equal.
    ///
    /// ```
    /// use parol::Pos;
    ///
    /// let this : Pos = (0, 2).into();
    /// let that : Pos = (0, 1).into();
    /// assert!(!this.preceds(&that));
    /// assert!(that.preceds(&this));
    ///
    /// let this : Pos = (0, 1).into();
    /// let that : Pos = (0, 1).into();
    /// assert!(!this.preceds(&that));
    /// assert!(!that.preceds(&this));
    ///
    /// let this : Pos = (0, 1).into();
    /// let that : Pos = (1, 2).into();
    /// assert!(!this.preceds(&that));
    /// assert!(!that.preceds(&this));
    ///
    /// let this : Pos = (0, 1).into();
    /// let that : Pos = (0, 2).into();
    /// assert!(this.preceds(&that));
    /// assert!(!that.preceds(&this));
    /// ```
    ///
    pub fn preceds(&self, that: &Self) -> bool {
        self.pr_index == that.pr_index && self.sy_index + 1 == that.sy_index
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
