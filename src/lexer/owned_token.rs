use crate::lexer::TerminalIndex;
use std::fmt::{Display, Error, Formatter};

///
/// This is a token type in which the matched text is owned, i.e. it does not
/// have any reference to the input text.
/// This is necessary to safely put the token into a decoupled parse tree.
///
#[derive(Debug, Clone, Eq, PartialEq)]
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
        write!(
            f,
            r#""{}"[{}]({},{}-{})"#,
            self.symbol,
            self.token_type,
            self.line,
            self.column,
            self.column + self.symbol.len() - 1
        )
    }
}
