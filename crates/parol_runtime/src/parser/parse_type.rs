use crate::lexer::TerminalIndex;
use crate::parser::{NonTerminalIndex, ProductionIndex};
use std::fmt::{Display, Error, Formatter};

///
/// The type of the elements in the parser stack.
///
#[derive(Debug, Clone, Copy)]
pub enum ParseType {
    ///
    /// The index of a non-terminal in the generated NON_TERMINALS array
    ///
    N(NonTerminalIndex),

    ///
    /// The index of a terminal in the generated TERMINALS array
    ///
    T(TerminalIndex),

    ///
    /// End of production marker
    ///
    E(ProductionIndex),
}

impl Display for ParseType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Self::N(n) => write!(f, "N({})", n),
            Self::T(t) => write!(f, "T({})", t),
            Self::E(e) => write!(f, "E({})", e),
        }
    }
}

///
/// The generated parsers are push down automata (PDA) which utilize a stack
/// during parsing. It helps to process the grammar's productions.
///
#[derive(Debug, Default)]
pub struct ParseStack {
    ///
    /// The actual stack.
    ///
    pub stack: Vec<ParseType>,
    terminal_names: &'static [&'static str],
    non_terminal_names: &'static [&'static str],
}

impl ParseStack {
    ///
    /// Creates a new instance with the given parameters.
    ///
    pub fn new(
        terminal_names: &'static [&'static str],
        non_terminal_names: &'static [&'static str],
    ) -> Self {
        Self {
            stack: Vec::new(),
            terminal_names,
            non_terminal_names,
        }
    }

    fn decode_terminal(&self, terminal_index: TerminalIndex) -> &'static str {
        self.terminal_names[terminal_index as usize]
    }

    fn decode_non_terminal(&self, non_terminal_index: usize) -> &'static str {
        self.non_terminal_names[non_terminal_index]
    }

    /// Returns all relevant tokens that are already expected (i.e. resolved) from the parse stack
    /// The strategy is to only take terminal indices as long no non-terminal (with unresolved
    /// content) or any scanner switch instructions is found. End of production markers are ignored.
    pub(crate) fn expected_token_types(&self) -> Vec<TerminalIndex> {
        self.stack
            .iter()
            .rev()
            .take_while(|e| matches!(e, ParseType::E(_)) || matches!(e, ParseType::T(_)))
            .filter_map(|e| match e {
                ParseType::T(t) => Some(*t),
                _ => None,
            })
            .collect::<Vec<_>>()
    }
}

impl Display for ParseStack {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.stack
            .iter()
            .rev()
            .enumerate()
            .try_for_each(|(i, e)| match e {
                ParseType::T(t) => writeln!(f, "{} - T({})", i, self.decode_terminal(*t)),
                ParseType::N(n) => writeln!(f, "{} - N({})", i, self.decode_non_terminal(*n)),
                _ => writeln!(f, "{} - {}", i, e),
            })
    }
}
