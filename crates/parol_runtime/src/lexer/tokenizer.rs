use crate::lexer::TerminalIndex;
use anyhow::{anyhow, Result};
use log::trace;
use regex::Regex;

///
/// This is an  unmatchable regular expression.
/// It is normally not included in the generated Regex's source but stands for
/// tokens that should be skipped, i.e. if a language doesn't support block
/// comments you could mark the regex on index token::BLOCK_COMMENT as
/// unmatchable.
///
pub const UNMATCHABLE_TOKEN: &str = r###"\w\b\w"###;

///
/// Regular expression for new lines
///
pub const NEW_LINE_TOKEN: &str = r###"\r\n|\r|\n"###;

///
/// Regular expression for any whitespace except newline characters
///
pub const WHITESPACE_TOKEN: &str = r###"[\s--\r\n]+"###;

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
    pub fn build(
        augmented_terminals: &[&str],
        scanner_specifics: &[&str],
        scanner_terminal_indices: &[usize],
    ) -> Result<Self> {
        debug_assert_eq!(5, scanner_specifics.len());
        let internal_terminals =
            scanner_specifics
                .iter()
                .enumerate()
                .fold(Vec::new(), |mut acc, (i, t)| {
                    if *t != UNMATCHABLE_TOKEN {
                        acc.push(format!("(?P<G{}>{})", i, t));
                    }
                    acc
                });
        let mut combined = scanner_terminal_indices
            .iter()
            .map(|term_idx| format!("(?P<G{}>{})", term_idx, augmented_terminals[*term_idx]))
            .fold(internal_terminals, |mut acc, e| {
                acc.push(e);
                acc
            })
            .join("|");
        let error_token_type = augmented_terminals.len() - 1;
        debug_assert_eq!(
            ERROR_TOKEN, augmented_terminals[error_token_type],
            "Last token should always be the error token!"
        );
        combined.push_str(
            format!(
                "|(?P<G{}>{})",
                error_token_type, augmented_terminals[error_token_type]
            )
            .as_str(),
        );

        let rx = combined.to_string();
        trace!("Generated regex for scanner:\n{}", rx);
        let rx = Regex::new(&rx).map_err(|e| anyhow!(e))?;
        Ok(Self {
            rx,
            error_token_type,
        })
    }
}
