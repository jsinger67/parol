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

#[derive(Debug)]
pub(crate) struct LookaheadExpression {
    pub(crate) is_positive: bool,
    pub(crate) pattern: String,
}

#[derive(Debug)]
pub(crate) struct Pattern {
    pub regex: String,
    pub terminal: TerminalIndex,
    pub lookahead: Option<LookaheadExpression>,
}

impl Pattern {
    pub fn new(
        regex: String,
        terminal: TerminalIndex,
        lookahead_expression: Option<(bool, &str)>,
    ) -> Self {
        Self {
            regex,
            terminal,
            lookahead: lookahead_expression.map(|(is_positive, pattern)| LookaheadExpression {
                is_positive,
                pattern: pattern.to_string(),
            }),
        }
    }
}

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
    pub(crate) patterns: Vec<Pattern>,

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
    /// As of version 2 the augmented_terminals also include optional lookahead expressions
    /// consisting of a boolean flag and a string. The boolean flag indicates if the lookahead
    /// is positive or negative. The string is the regular expression for the lookahead.
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
        augmented_terminals: &[(&str, Option<(bool, &str)>)],
        scanner_specifics: &[&str],
        scanner_terminal_indices: &[TerminalIndex],
    ) -> Result<Self> {
        debug_assert_eq!(5, scanner_specifics.len());
        let mut patterns = Vec::<Pattern>::with_capacity(augmented_terminals.len());
        scanner_specifics.iter().enumerate().for_each(|(i, t)| {
            if *t != UNMATCHABLE_TOKEN {
                patterns.push(Pattern::new(t.to_string(), i as TerminalIndex, None));
            }
        });
        scanner_terminal_indices
            .iter()
            .map(|term_idx| {
                Pattern::new(
                    augmented_terminals[*term_idx as usize].0.to_string(),
                    *term_idx,
                    augmented_terminals[*term_idx as usize].1,
                )
            })
            .for_each(|p| {
                patterns.push(p);
            });
        let error_token_type = (augmented_terminals.len() - 1) as TerminalIndex;

        debug_assert_eq!(
            ERROR_TOKEN, augmented_terminals[error_token_type as usize].0,
            "Last token should always be the error token!"
        );

        patterns.push(Pattern::new(
            augmented_terminals[error_token_type as usize].0.to_string(),
            error_token_type,
            None,
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
