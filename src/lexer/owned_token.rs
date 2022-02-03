use crate::lexer::{FormatToken, TerminalIndex};
use miette::SourceSpan;
use std::fmt::{Display, Error, Formatter};

///
/// This is a token type in which the matched text is owned, i.e. it does not
/// have any reference to the input text.
/// This is necessary to safely put the token into a decoupled parse tree.
///
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct OwnedToken {
    /// The matched string
    pub symbol: String,
    /// The index of the terminal in the augmented terminal list
    pub token_type: TerminalIndex,
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
}

impl Display for OwnedToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        let (c1, c2) = if self.symbol.starts_with('\'') {
            ('<', '>')
        } else {
            ('\'', '\'')
        };
        write!(
            f,
            "{}{}{}[{}]({},{}-{})",
            c1,
            self.symbol,
            c2,
            self.token_type,
            self.line,
            self.column,
            self.column + self.symbol.len()
        )
    }
}

impl FormatToken for OwnedToken {
    fn format<T>(
        &self,
        file_name: &T,
        terminal_names: &'static [&'static str],
    ) -> std::string::String
    where
        T: AsRef<std::path::Path>,
    {
        let name = terminal_names[self.token_type];
        format!(
            "'{}'({}) at {}:{}:{}",
            self.symbol.escape_default(),
            name,
            file_name.as_ref().display(),
            self.line,
            self.column,
        )
    }
}

impl<'t> From<&OwnedToken> for SourceSpan {
    fn from(token: &OwnedToken) -> Self {
        (token.start_pos + token.pos - token.length, token.length).into()
    }
}
