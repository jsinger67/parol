use std::borrow::Cow;
use std::fmt::{Debug, Display, Error, Formatter};
use std::path::Path;

use miette::SourceSpan;

///
/// Common Location type
///
#[derive(Debug, Default, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Location {
    /// Position information: line number, starting by 1
    /// A value of 0 indicates an invalid position, for instance for EOF token.
    pub line: usize,

    /// Position information: column number, starting by 1
    /// A value of 0 indicates an invalid position, for instance for EOF token.
    pub column: usize,

    /// Length of the matched input terminal
    /// A value of 0 indicates a virtual token, for instance an EOF token.
    /// Be careful: User tokens with length 0 are always invalid!!!
    pub length: usize,

    /// Start position as byte offset at last scanner switching.
    pub(crate) start_pos: usize,

    /// Relative position from start position as byte offset after matching this
    /// terminal. Needed for scanner switching.
    pub(crate) pos: usize,

    /// The name of the input file
    pub file_name: Cow<'static, Path>,
}

impl Location {
    ///
    /// Creates a token with given values.
    ///
    pub fn with<T>(
        line: usize,
        column: usize,
        length: usize,
        start_pos: usize,
        pos: usize,
        file_name: T,
    ) -> Self
    where
        T: Into<Cow<'static, Path>>,
    {
        Self {
            line,
            column,
            length,
            start_pos,
            pos,
            file_name: file_name.into(),
        }
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(
            f,
            "{}:{}:{}-{}",
            self.file_name.display(),
            self.line,
            self.column,
            self.column + self.length
        )
    }
}

impl From<&Location> for SourceSpan {
    fn from(location: &Location) -> Self {
        (
            location.start_pos + location.pos - location.length,
            location.length,
        )
            .into()
    }
}
