use crate::lexer::{FormatToken, TerminalIndex};
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
    fn format(
        &self,
        file_name: &str,
        terminal_names: &'static [&'static str],
    ) -> std::string::String {
        let name = terminal_names[self.token_type];
        format!(
            "'{}'({}) at {}:{}:{}",
            self.symbol.escape_default(),
            name,
            file_name,
            self.line,
            self.column,
        )
    }
}
