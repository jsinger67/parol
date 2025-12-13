use std::{
    fmt::{Debug, Display, Error, Formatter},
    ops::Range,
    path::PathBuf,
    sync::Arc,
};

use derive_builder::Builder;

///
/// Common Location type
/// This type is used to store the location of a token in the input text.
/// The location is stored as line and column numbers, and as start and end positions in the input
/// text.
///
/// We don't use std::ops::Range<usize> for the span information because we need to implement Ord
/// and PartialOrd for Location.
///
#[derive(Builder, Debug, Default, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Location {
    /// Position information: line number, starting by 1
    /// A value of 0 indicates an invalid position, for instance for EOF token.
    #[builder(default)]
    pub start_line: u32,

    /// Position information: column number, starting by 1
    /// A value of 0 indicates an invalid position, for instance for EOF token.
    #[builder(default)]
    pub start_column: u32,

    /// Position information: line number, starting by 1
    /// A value of 0 indicates an invalid position, for instance for EOF token.
    #[builder(default)]
    pub end_line: u32,

    /// Position information: column number, starting by 1
    /// A value of 0 indicates an invalid position, for instance for EOF token.
    #[builder(default)]
    pub end_column: u32,

    /// The start of the span of the token in the input text
    /// We use 0 also when dealing with artificial tokens introduced by the parser during error
    /// recovery.
    #[builder(default)]
    pub start: u32,

    /// The end of the span of the token in the input text
    /// The end is exclusive. It is the first character after the span.
    /// We use 0 also when dealing with artificial tokens introduced by the parser during error
    /// recovery.
    #[builder(default)]
    pub end: u32,

    /// The name of the input file
    pub file_name: Arc<PathBuf>,
}

impl Location {
    /// Calculate the length of the location
    #[inline]
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start) as usize
    }

    /// Returns true if the location is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    /// Create a std::ops::Range<usize> from the location
    /// This is useful to extract the text of the token from the input text.
    #[inline]
    pub fn range(&self) -> Range<usize> {
        self.start as usize..self.end as usize
    }

    /// Returns the start of the location
    #[inline]
    pub fn start(&self) -> usize {
        self.start as usize
    }

    /// Returns the end of the location
    #[inline]
    pub fn end(&self) -> usize {
        self.end as usize
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        if self.start_line == self.end_line {
            write!(
                f,
                "{}:{}:{}-{}",
                self.file_name.display(),
                self.start_line,
                self.start_column,
                self.end_column
            )
        } else {
            write!(
                f,
                "{}:{}:{}-{}:{}",
                self.file_name.display(),
                self.start_line,
                self.start_column,
                self.end_line,
                self.end_column
            )
        }
    }
}

impl From<&Location> for Range<usize> {
    fn from(location: &Location) -> Self {
        location.range()
    }
}
