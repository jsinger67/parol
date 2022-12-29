use std::borrow::Cow;
use std::fmt::{Debug, Display, Error, Formatter};
use std::ops::Range;
use std::path::Path;

use derive_builder::Builder;

///
/// Common Location type
///
#[derive(Builder, Debug, Default, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Location {
    /// Position information: line number, starting by 1
    /// A value of 0 indicates an invalid position, for instance for EOF token.
    pub start_line: usize,

    /// Position information: column number, starting by 1
    /// A value of 0 indicates an invalid position, for instance for EOF token.
    pub start_column: usize,

    /// Position information: line number, starting by 1
    /// A value of 0 indicates an invalid position, for instance for EOF token.
    pub end_line: usize,

    /// Position information: column number, starting by 1
    /// A value of 0 indicates an invalid position, for instance for EOF token.
    pub end_column: usize,

    /// Length of the matched input terminal
    /// A value of 0 indicates a virtual token, for instance an EOF token.
    /// Be careful: User tokens with length 0 are always invalid!!!
    pub length: usize,

    /// Start position in the input stream as byte offset at last scanner switching.
    #[builder(default)]
    pub scanner_switch_pos: usize,

    /// Relative position from scanner_switch_pos as byte offset.
    pub offset: usize,

    /// The name of the input file
    pub file_name: Cow<'static, Path>,
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
        let start = location.scanner_switch_pos + location.offset - location.length;
        Range {
            start,
            end: start + location.length,
        }
    }
}
