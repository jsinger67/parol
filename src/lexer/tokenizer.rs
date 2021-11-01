use super::errors::*;
use crate::lexer::TerminalIndex;
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
/// Regular expression for any whitespace
///
pub const WHITESPACE_TOKEN: &str = r###"\s+"###;

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
    /// Creates a new Tokenizer object from augmented terminals.
    /// The augmented terminals contain some additional terminal definitions
    /// such as whitespace and newline handling as well as comments.
    ///
    pub fn build(augmented_terminals: &[&str]) -> Result<Tokenizer> {
        let mut error_token_type = 0;
        let combined = augmented_terminals
            .iter()
            .enumerate()
            .fold(Vec::new(), |mut acc, (i, t)| {
                if *t != UNMATCHABLE_TOKEN {
                    acc.push(format!("(?P<G{}>{})", i, t));
                }
                if *t == ERROR_TOKEN {
                    error_token_type = i as TerminalIndex;
                }
                acc
            })
            .join("|");
        if error_token_type == 0 {
            Err("Augmented terminals should always include the error token!".into())
        } else {
            let combined = combined.trim_end_matches('|');
            let rx = combined.to_string();
            let rx = Regex::new(&rx).chain_err(|| "Unable to compile generated RegEx!")?;

            Ok(Tokenizer {
                rx,
                error_token_type,
            })
        }
    }
}
