use anyhow::Result;

use super::TerminalIndex;

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
/// The Tokenizer abstracts one specific scanner state or scanner mode.
/// It is used during generation of the scanner.
///
#[derive(Debug)]
pub struct Tokenizer {
    /// The regular expressions that are valid token types in this mode, bundled with their token
    /// type numbers.
    /// The priorities of the patterns are determined by their order in the vector. Lower indices
    /// have higher priority if multiple patterns match the input and have the same length.
    pub(crate) patterns: Vec<(String, TerminalIndex)>,

    /// This is the token index for the special error token.
    /// Its value isn't constant and depends on the given token count.
    /// It is always the last token that is tried to match and usually
    /// indicates an error.
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
        let mut patterns = Vec::with_capacity(augmented_terminals.len());
        scanner_specifics.iter().enumerate().for_each(|(i, t)| {
            if *t != UNMATCHABLE_TOKEN {
                patterns.push((t.to_string(), i as TerminalIndex));
            }
        });
        scanner_terminal_indices
            .iter()
            .map(|term_idx| {
                (
                    augmented_terminals[*term_idx as usize].to_string(),
                    *term_idx,
                )
            })
            .for_each(|e| {
                patterns.push(e);
            });
        let error_token_type = (augmented_terminals.len() - 1) as TerminalIndex;

        debug_assert_eq!(
            ERROR_TOKEN, augmented_terminals[error_token_type as usize],
            "Last token should always be the error token!"
        );

        patterns.push((
            augmented_terminals[error_token_type as usize].to_string(),
            error_token_type,
        ));

        debug_assert!(
            patterns.len() >= scanner_terminal_indices.len(),
            "Error in mapping of PatternID to TerminalIndex"
        );

        Ok(Self {
            patterns,
            error_token_type,
        })
    }
}
