use crate::lexer::{FormatToken, TerminalIndex};
use std::fmt::{Display, Error, Formatter};

///
/// Special token constants the lexer has to deal with regularly.
/// There are some special fix values used for common token types.
/// See constants below.
///

/// End of input constant
pub const EOI: TerminalIndex = 0;
/// New line token
pub const NEW_LINE: TerminalIndex = 1;
/// Whitespace token
pub const WHITESPACE: TerminalIndex = 2;
/// Line comment token
pub const LINE_COMMENT: TerminalIndex = 3;
/// Block comment token
pub const BLOCK_COMMENT: TerminalIndex = 4;
/// Index of the first user token.
pub const FIRST_USER_TOKEN: TerminalIndex = 5;

const EOI_TOKEN: &str = "$";

///
/// The Token<'t> type represents a scanned token.
/// It has a reference to the scanned text in the symbol member.
///
/// The lifetime parameter `'t` refers to the lifetime of the scanned text.
///
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub struct Token<'t> {
    /// The matched string
    pub symbol: &'t str,

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

    /// Relative position from start position as byte offset after matching this
    /// terminal. Needed for scanner switching.
    pub(crate) pos: usize,
}

impl<'t> Token<'t> {
    ///
    /// Creates an End-Of-Input token
    ///
    pub fn eoi() -> Self {
        Self {
            symbol: EOI_TOKEN,
            token_type: EOI,
            line: 0,
            column: 0,
            length: 0,
            pos: 0,
        }
    }

    ///
    /// Creates a token with given values.
    ///
    pub fn with(
        symbol: &'t str,
        token_type: TerminalIndex,
        line: usize,
        column: usize,
        length: usize,
        pos: usize,
    ) -> Self {
        Self {
            symbol,
            token_type,
            line,
            column,
            length,
            pos,
        }
    }

    ///
    /// Indicates wether the token is normally skipped by the TokenStream.
    /// The behavior is independent from the language.
    ///
    pub fn is_skip_token(&self) -> bool {
        self.token_type > EOI && self.token_type < FIRST_USER_TOKEN
    }
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        let (c1, c2) = if self.symbol.starts_with('\'') {
            ('<', '>')
        } else {
            ('\'', '\'')
        };
        write!(
            f,
            "{}{}{}, Ty:{}, Loc:{},{}-{}",
            c1,
            self.symbol,
            c2,
            self.token_type,
            self.line,
            self.column,
            self.column + self.length
        )
    }
}

impl FormatToken for Token<'_> {
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
