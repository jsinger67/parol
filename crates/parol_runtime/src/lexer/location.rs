use std::fmt::{Debug, Display, Error, Formatter};
use std::ops::Range;
use std::path::PathBuf;
use std::sync::Arc;

use derive_builder::Builder;

///
/// Common Location type
///
#[derive(Builder, Debug, Default, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Location {
    /// Position information: line number, starting by 1
    /// A value of 0 indicates an invalid position, for instance for EOF token.
    pub start_line: u32,

    /// Position information: column number, starting by 1
    /// A value of 0 indicates an invalid position, for instance for EOF token.
    pub start_column: u32,

    /// Position information: line number, starting by 1
    /// A value of 0 indicates an invalid position, for instance for EOF token.
    pub end_line: u32,

    /// Position information: column number, starting by 1
    /// A value of 0 indicates an invalid position, for instance for EOF token.
    pub end_column: u32,

    /// Length of the matched input terminal
    /// A value of 0 indicates a virtual token, for instance an EOF token.
    /// Be careful: User tokens with length 0 are always invalid!!!
    /// We use 0 also when dealing with artificial tokens introduced by the parser during error
    /// recovery.
    #[builder(default)]
    pub length: u32,

    /// Absolute position in the haystack as byte offset.
    /// We use default when dealing with artificial tokens introduced by the parser during error
    /// recovery.
    #[builder(default)]
    pub offset: usize,

    /// The name of the input file
    pub file_name: Arc<PathBuf>,
}

impl Display for Location {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
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

impl From<&Location> for Range<usize> {
    fn from(location: &Location) -> Self {
        let start = location.offset - location.length as usize;
        Range {
            start,
            end: start + location.length as usize,
        }
    }
}
