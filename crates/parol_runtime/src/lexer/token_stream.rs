use crate::lexer::EOI;
use crate::parser::ScannerIndex;
use crate::{LexerError, LocationBuilder, TerminalIndex, Token, TokenIter, TokenNumber};
use log::trace;
use scnr2::ScannerImpl;

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;

use super::TokenBuffer;

///
/// The TokenStream<'t> type is the interface the parser actually uses.
/// It provides the lookahead functionality by maintaining a lookahead buffer.
///
/// The lifetime parameter `'t` refers to the lifetime of the scanned text.
///
pub struct TokenStream<'t, F>
where
    F: Fn(char) -> Option<usize> + 'static + Clone,
{
    /// The number of available lookahead tokens
    pub k: usize,

    /// The input text
    pub(crate) input: &'t str,

    /// The name of the input file
    pub file_name: Arc<PathBuf>,

    /// The actual token iterator.
    /// It is replaced by a new one in case of scanner state switch.
    token_iter: TokenIter<'t, F>,

    /// Lookahead token buffer, maximum size is k
    pub tokens: TokenBuffer<'t>,

    /// Flag to indicate if the parser is in error recovery mode
    pub(crate) recovering: bool,
}

impl<'t, F> TokenStream<'t, F>
where
    F: Fn(char) -> Option<usize> + 'static + Clone,
{
    ///
    /// Creates a new TokenStream object from an augmented terminals list and
    /// an input string.
    /// The k determines the number of lookahead tokens the stream supports.
    ///
    /// Currently this never return LexerError but it could be changed in the future.
    ///
    pub fn new<T>(
        input: &'t str,
        file_name: T,
        scanner_impl: Rc<RefCell<ScannerImpl>>,
        match_function: &'static F,
        k: usize,
    ) -> Result<Self, LexerError>
    where
        T: AsRef<Path>,
    {
        let file_name = Arc::new(file_name.as_ref().to_owned());
        // To output the compiled automata as dot files uncomment the following two lines
        // const TARGET_FOLDER: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../target");
        // let _ = scanner.generate_compiled_automata_as_dot("Parol", Path::new(TARGET_FOLDER));
        let token_iter = TokenIter::new(
            ScannerImpl::find_matches_with_position(scanner_impl, input, 0, match_function),
            input,
            file_name.clone(),
            k,
        );

        // issue #54 "Lookahead exceeds token buffer length" with simple grammar:
        // Ensure that k is at least 1 and at most MAX_K
        let k = std::cmp::max(1, k);

        let mut token_stream = Self {
            k,
            input,
            file_name,
            token_iter,
            tokens: TokenBuffer::new(),
            recovering: false,
        };
        token_stream.read_tokens(k)?;
        Ok(token_stream)
    }

    ///
    /// Provides at maximum k tokens lookahead relative to the current read
    /// position.
    /// If successful it returns a cloned token from buffer position self.pos + n
    ///
    pub fn lookahead(&mut self, n: usize) -> Result<Token<'t>, LexerError> {
        if n >= self.k {
            Err(LexerError::LookaheadExceedsMaximum)
        } else {
            // Fill buffer to lookahead size k relative to pos
            self.ensure_buffer()?;
            if n >= self.tokens.len() {
                if self.tokens.is_empty() && self.recovering {
                    trace!("lookahead LA({n}): EOI for recovery");
                    Ok(Token::eoi(TokenNumber::MAX))
                } else {
                    trace!("{} in {}", n, self.tokens);
                    Err(LexerError::LookaheadExceedsTokenBufferLength)
                }
            } else {
                trace!("LA({}): {}", n, self.tokens.non_skip_token_at(n).unwrap());
                Ok(self.tokens.non_skip_token_at(n).unwrap().clone())
            }
        }
    }

    ///
    /// Provides at maximum k tokens lookahead relative to the current read
    /// position.
    /// If successful it returns the type (index) of the token at buffer
    /// position n.
    ///
    pub fn lookahead_token_type(&mut self, n: usize) -> Result<TerminalIndex, LexerError> {
        if n >= self.k {
            Err(LexerError::LookaheadExceedsMaximum)
        } else {
            // Fill buffer to lookahead size k relative to pos
            self.ensure_buffer()?;
            if n >= self.tokens.len() {
                if self.tokens.is_empty() && self.recovering {
                    trace!("lookahead_token_type LA({n}): EOI for recovery");
                    Ok(EOI)
                } else {
                    trace!("{} in {}", n, self.tokens);
                    Err(LexerError::LookaheadExceedsTokenBufferLength)
                }
            } else {
                trace!(
                    "Type(LA({})): {}",
                    n,
                    self.tokens.non_skip_token_at(n).unwrap()
                );
                Ok(self.tokens.non_skip_token_at(n).unwrap().token_type)
            }
        }
    }

    /// Returns all skip tokens at the beginning of the token buffer.
    /// The tokens are removed from the buffer and the line and column numbers are updated.
    #[inline]
    pub fn take_skip_tokens(&mut self) -> Vec<Token<'t>> {
        self.tokens.take_skip_tokens()
    }

    ///
    /// Consumes one token.
    /// If necessary more input is read via the token_iter into the tokens buffer.
    ///
    /// The token's positions are captured to support scanner switching.
    ///
    pub fn consume(&mut self) -> Result<Token<'t>, LexerError> {
        self.ensure_buffer()?;
        let token;
        if self.tokens.is_empty() {
            return Err(LexerError::InternalError(
                "Consume on empty buffer is impossible".into(),
            ));
        } else {
            // We consume token LA(1) with buffer index 0.
            trace!("Consuming {}", &self.tokens.non_skip_token_at(0).unwrap());
            token = self.tokens.consume()?;
            self.ensure_buffer()?;
        }
        Ok(token)
    }

    ///
    /// Test if all input was processed by the parser
    ///
    pub fn all_input_consumed(&self) -> bool {
        // The unwrap is safe because the token buffer is not empty here.
        self.tokens.is_buffer_empty()
            || self.tokens.non_skip_token_at(0).unwrap().token_type == super::EOI
    }

    ///
    /// Returns the last valid token from token buffer if there is one
    ///
    pub fn last_token(&self) -> Result<&Token<'_>, LexerError> {
        self.tokens
            .non_skip_tokens_rev()
            .find(|t| t.token_type != super::EOI)
            .ok_or(LexerError::TokenBufferEmptyError)
    }

    ///
    /// Returns the name of the currently active scanner state.
    /// Used for diagnostics.
    ///
    #[inline]
    pub fn current_scanner(&self) -> &str {
        self.token_iter
            .scanner_mode_name(self.token_iter.current_mode())
            .unwrap_or("unknown")
    }

    /// Returns the index of the currently active scanner state.
    pub fn current_scanner_index(&self) -> ScannerIndex {
        self.token_iter.current_mode()
    }

    ///
    /// Reads at most n tokens from the input stream and stores them in the token buffer.
    /// It returns the number of tokens read.
    /// The function is used by ensure_buffer and switch_scanner.
    /// The idea is to fill the lookahead buffer with tokens and to switch scanner states as early
    /// as possible.
    ///
    fn read_tokens(&mut self, n: usize) -> Result<usize, LexerError> {
        let mut tokens_read = 0usize;
        for token in &mut self.token_iter {
            trace!("Read {}: {}", self.tokens.len(), token);
            if !token.is_skip_token() {
                tokens_read += 1;
            }
            self.tokens.add(token);
            if tokens_read >= n {
                break;
            }
        }
        while tokens_read < n {
            trace!("read_tokens: Filling with EOI at end of input");
            self.tokens.add(Token::eoi(TokenNumber::MAX));
            tokens_read += 1;
        }
        Ok(tokens_read)
    }

    ///
    /// The function fills the lookahead buffer (self.tokens) with k tokens.
    /// It returns the number of tokens read.
    ///
    pub(crate) fn ensure_buffer(&mut self) -> Result<usize, LexerError> {
        let fill_len = self.tokens.len();
        if fill_len < self.k {
            // Fill buffer to lookahead size k
            self.read_tokens(self.k - fill_len)
        } else {
            Ok(0)
        }
    }

    /// Returns the token types of the tokens in the lookahead buffer.
    /// It only considers non-skip-tokens.
    pub(crate) fn token_types(&self) -> Vec<TerminalIndex> {
        self.tokens.non_skip_token_types()
    }

    pub(crate) fn diagnostic_message(&self) -> String {
        format!(
            "Lookahead buffer:\n[\n  {}\n]\n",
            self.tokens
                .non_skip_tokens()
                .enumerate()
                .map(|(i, t)| format!("LA[{i}]: ({t})"))
                .collect::<Vec<String>>()
                .join(",\n  ")
        )
    }

    pub(crate) fn replace_token_type_at(
        &mut self,
        index: usize,
        token_type: TerminalIndex,
    ) -> Result<(), LexerError> {
        if self.tokens.len() > index {
            trace!(
                "replacing token {} at index {} by {}",
                self.tokens.non_skip_token_at(index).unwrap(),
                index,
                token_type
            );
            if (self.tokens.non_skip_token_at(index).unwrap().token_type) == EOI {
                Err(LexerError::RecoveryError("Can't replace EOI".to_owned()))
            } else {
                self.tokens.non_skip_token_at_mut(index).unwrap().token_type = token_type;
                Ok(())
            }
        } else {
            Err(LexerError::RecoveryError(
                "Can't replace beyond token buffer".to_owned(),
            ))
        }
    }

    /// Used in recovery mode to insert a token at a specific index in the token buffer.
    pub(crate) fn insert_token_at(
        &mut self,
        index: usize,
        token_type: TerminalIndex,
    ) -> Result<(), LexerError> {
        if self.tokens.len() >= index {
            trace!("inserting token {token_type} at index {index}");
            let location = if self.tokens.len() > index {
                self.tokens
                    .non_skip_token_at(index)
                    .unwrap()
                    .location
                    .clone()
            } else {
                LocationBuilder::default()
                    .file_name(self.file_name.clone())
                    .build()
                    .unwrap()
            };
            self.tokens.insert(
                index,
                Token::default()
                    .with_type(token_type)
                    .with_location(location),
            );
            Ok(())
        } else {
            Err(LexerError::RecoveryError(format!(
                "Can't insert in token buffer at position {index}"
            )))
        }
    }

    // /// Returns the name of the scanner mode with the given index.
    // #[inline]
    // fn scanner_mode_name(&self, index: usize) -> &str {
    //     self.token_iter
    //         .scanner_mode_name(index)
    //         .unwrap_or("unknown")
    // }

    /// Sets the token stream in error recovery mode.
    /// In this mode the parser can try to read more tokens even if the end of input is reached.
    /// The token stream will return EOI tokens if the token buffer is empty.
    pub(crate) fn enter_recovery_mode(&mut self) {
        self.recovering = true;
    }
}
