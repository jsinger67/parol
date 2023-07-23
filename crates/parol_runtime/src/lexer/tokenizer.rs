use crate::lexer::TerminalIndex;
use anyhow::{anyhow, Result};
use log::trace;
use regex_automata::{dfa::regex::Regex, PatternID};

///
/// This is an  unmatchable regular expression.
/// It is normally not included in the generated Regex's source but stands for
/// tokens that should be skipped, i.e. if a language doesn't support block
/// comments you could mark the regex on index token::BLOCK_COMMENT as
/// unmatchable.
///
pub const UNMATCHABLE_TOKEN: &str = r"\w\b\w";

///
/// Regular expression for new lines
///
pub const NEW_LINE_TOKEN: &str = r"\r\n|\r|\n";

///
/// Regular expression for any whitespace except newline characters
///
pub const WHITESPACE_TOKEN: &str = r"[\s--\r\n]+";

///
/// Regular expression that matches any other token. With this you can detect
/// so far unmatched tokens. It is only used for error detection during lexical
/// analysis.
///
pub const ERROR_TOKEN: &str = r###"."###;

///
/// The Tokenizer creates a specially formatted regular expression that can be
/// used for tokenizing an input string.
///
pub struct Tokenizer {
    pub(crate) rx: Regex,

    /// This vector provides the mapping of
    /// scanned PatternID (index in vec) to TerminalIndex (content at index)
    token_types: Vec<TerminalIndex>,

    ///
    /// This is the token index for the special error token.
    /// Its value isn't constant and depends on the given token count.
    /// It is always the last token that is tried to match and usually
    /// indicates an error.
    ///
    pub error_token_type: TerminalIndex,
}

impl Tokenizer {
    ///
    /// Creates a new Tokenizer object from augmented terminals and scanner
    /// specific information.
    ///
    /// # Arguments
    ///
    /// ## augmented_terminals
    /// All valid terminals of the grammar. These include the specific common terminals
    /// `EOI`, `NEW_LINE`, `WHITESPACE`, `LINE_COMMENT`, `BLOCK_COMMENT` with the value
    /// `UNMATCHABLE_TOKEN` to provide consistent index handling for all scanner states.
    ///
    /// ## scanner_specifics
    /// The values of the five scanner specific common terminals `EOI`, `NEW_LINE`, `WHITESPACE`,
    /// `LINE_COMMENT` and `BLOCK_COMMENT`
    ///
    /// ## scanner_terminal_indices
    /// The indices of token types belonging to this scanner state. These indices are pointing into
    /// `augmented_terminals`.
    ///
    /// # Errors
    ///
    /// This function will return an error if the regex patterns can't be compiled.
    pub fn build(
        augmented_terminals: &[&str],
        scanner_specifics: &[&str],
        scanner_terminal_indices: &[TerminalIndex],
    ) -> Result<Self> {
        debug_assert_eq!(5, scanner_specifics.len());
        let mut token_types = Vec::with_capacity(augmented_terminals.len());
        let internal_terminals = scanner_specifics.iter().enumerate().fold(
            Vec::with_capacity(augmented_terminals.len()),
            |mut acc, (i, t)| {
                if *t != UNMATCHABLE_TOKEN {
                    acc.push(*t);
                    token_types.push(i as TerminalIndex);
                }
                acc
            },
        );
        let mut patterns = scanner_terminal_indices
            .iter()
            .map(|term_idx| (*term_idx, augmented_terminals[*term_idx as usize]))
            .fold(internal_terminals, |mut acc, (term_idx, pattern)| {
                acc.push(pattern);
                token_types.push(term_idx);
                acc
            });
        let error_token_type = (augmented_terminals.len() - 1) as TerminalIndex;

        debug_assert_eq!(
            ERROR_TOKEN, augmented_terminals[error_token_type as usize],
            "Last token should always be the error token!"
        );

        patterns.push(augmented_terminals[error_token_type as usize]);
        token_types.push(error_token_type);

        debug_assert_eq!(
            patterns.len(),
            token_types.len(),
            "Error in mapping of PatternID to TerminalIndex"
        );

        trace!("Generated regex for scanner:\n{:?}", patterns);
        let rx = Regex::builder()
            .build_many(&patterns)
            .map_err(|e| anyhow!(e))?;
        Ok(Self {
            rx,
            token_types,
            error_token_type,
        })
    }

    /// Decode the pattern index to a terminal index.
    ///
    /// # Panics
    ///
    /// This panics if `pattern_id >= self.token_types.len()`.
    #[inline]
    pub(crate) fn terminal_index_of_pattern(&self, pattern_id: PatternID) -> TerminalIndex {
        self.token_types[pattern_id]
    }
}
