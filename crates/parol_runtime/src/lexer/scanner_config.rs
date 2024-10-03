use scnr::ScannerMode;

use crate::{TerminalIndex, Tokenizer};

/// Scanner configuration fed into a TokenStream

#[derive(Debug)]

pub struct ScannerConfig {
    /// Name of the scanner configuration, i.e. the scanner state or mode
    pub name: &'static str,
    /// The customized tokenizer for this scanner configuration
    pub tokenizer: Tokenizer,
    /// The mapping of token types to new scanner configurations
    /// The entries are tuples of the terminal index and the new scanner configuration index and are
    /// sorted by terminal index.
    pub transitions: &'static [(TerminalIndex, usize)],
}

impl ScannerConfig {
    /// Create a new scanner configuration
    pub fn new(
        name: &'static str,
        tokenizer: Tokenizer,
        transitions: &'static [(TerminalIndex, usize)],
    ) -> Self {
        Self {
            name,
            tokenizer,
            transitions,
        }
    }

    /// Check if the scanner configuration has a transition on the given terminal index
    pub fn has_transition(&self, terminal_index: TerminalIndex) -> Option<usize> {
        self.transitions
            .iter()
            .find(|(term, _)| *term == terminal_index)
            .map(|(_, scanner)| *scanner)
    }
}

impl From<&ScannerConfig> for ScannerMode {
    fn from(config: &ScannerConfig) -> Self {
        ScannerMode::new(
            config.name,
            config
                .tokenizer
                .patterns
                .iter()
                .map(|(p, t)| (p.clone(), (*t).into())),
            config.transitions.iter().map(|(t, m)| (*t as usize, *m)),
        )
    }
}
