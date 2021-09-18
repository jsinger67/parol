use crate::analysis::compiled_la_dfa::TerminalIndex;
use crate::Terminal;

/// End of input constant
pub const EOI: TerminalIndex = 0;
/// Line comment token
pub const LINE_COMMENT: TerminalIndex = 3;
/// Block comment token
pub const BLOCK_COMMENT: TerminalIndex = 4;

pub fn get_terminal_index(terminals: &[&str], terminal: &Terminal) -> Option<TerminalIndex> {
    match terminal {
        Terminal::Trm(trm) => terminals.iter().position(|t| t == trm),
        Terminal::End => Some(EOI),
        Terminal::Eps => None,
    }
}
