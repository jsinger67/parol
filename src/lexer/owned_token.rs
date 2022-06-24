use crate::lexer::{FormatToken, TerminalIndex};
use miette::SourceSpan;
use std::fmt::{Display, Error, Formatter};

use super::{Location, Token};

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

    /// Position information
    pub location: Location,
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
            "{}{}{}[{}] at {}",
            c1, self.symbol, c2, self.token_type, self.location,
        )
    }
}

impl FormatToken for OwnedToken {
    fn format(&self, terminal_names: &'static [&'static str]) -> std::string::String {
        let name = terminal_names[self.token_type];
        format!(
            "'{}'({}) at {}",
            self.symbol.escape_default(),
            name,
            self.location,
        )
    }
}

impl<'t> From<&OwnedToken> for SourceSpan {
    fn from(token: &OwnedToken) -> Self {
        (&token.location).into()
    }
}

impl From<&Token<'_>> for OwnedToken {
    fn from(token: &Token<'_>) -> Self {
        token.to_owned()
    }
}
