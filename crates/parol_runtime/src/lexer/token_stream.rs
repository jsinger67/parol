use crate::lexer::EOI;
use crate::parser::ScannerIndex;
use crate::{LexerError, LocationBuilder, TerminalIndex, Token, TokenIter};
use log::{debug, trace};
use scnr::{ScannerBuilder, ScannerMode};

use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::ScannerConfig;

///
/// The TokenStream<'t> type is the interface the parser actually uses.
/// It provides the lookahead functionality by maintaining a lookahead buffer.
/// Also it provides the ability to switch scanner states. This is handled by
/// the used scanner implementation.
///
/// It also maintains the line and column numbers and propagates them to the tokens.
///
/// The lifetime parameter `'t` refers to the lifetime of the scanned text.
///
pub struct TokenStream<'t> {
    /// The number of available lookahead tokens
    pub k: usize,

    /// The input text
    pub(crate) input: &'t str,

    /// The name of the input file
    pub file_name: Arc<PathBuf>,

    /// The actual token iterator.
    /// It is replaced by a new one in case of scanner state switch.
    token_iter: TokenIter<'t>,

    /// Lookahead token buffer, maximum size is k
    pub tokens: Vec<Token<'t>>,

    /// Comment token buffer
    pub comments: Vec<Token<'t>>,

    /// Line number of last consumed token. Needed for scanner switching. Is initially 1.
    line: u32,

    /// Columns after last consumed token. Needed for scanner switching. Is initially 1.
    column: u32,

    /// Absolute position from start of input to the end of the last consumed token.
    last_consumed_token_end_pos: usize,

    /// Scanner stack to support push and pop operations for scanner configurations
    scanner_stack: Vec<ScannerIndex>,
}

impl<'t> TokenStream<'t> {
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
        scanners: &'static [ScannerConfig],
        k: usize,
    ) -> Result<Self, LexerError>
    where
        T: AsRef<Path>,
    {
        let file_name = Arc::new(file_name.as_ref().to_owned());
        let modes = scanners
            .iter()
            .map(|s| s.into())
            .collect::<Vec<ScannerMode>>();
        debug!("Scanner modes: {}", serde_json::to_string(&modes).unwrap());
        let scanner = ScannerBuilder::new().add_scanner_modes(&modes).build()?;
        // To enable debug output compliled DFA as dot file:
        // $env:RUST_LOG="scnr::internal::scanner_impl=debug"
        let _ = scanner.log_compiled_dfas_as_dot(&modes);
        let token_iter = TokenIter::new(scanner, input, file_name.clone(), k);

        // issue #54 "Lookahead exceeds token buffer length" with simple grammar:
        // Ensure that k is at least 1
        let k = std::cmp::max(1, k);

        let mut token_stream = Self {
            k,
            input,
            file_name,
            token_iter,
            tokens: Vec::with_capacity(k),
            comments: Vec::new(),
            line: 1,
            column: 1,
            last_consumed_token_end_pos: 0,
            scanner_stack: Vec::new(),
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
                Err(LexerError::LookaheadExceedsTokenBufferLength)
            } else {
                trace!("LA({}): {}", n, self.tokens[n]);
                Ok(self.tokens[n].clone())
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
                Err(LexerError::LookaheadExceedsTokenBufferLength)
            } else {
                trace!("Type(LA({})): {}", n, self.tokens[n]);
                Ok(self.tokens[n].token_type)
            }
        }
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
            trace!("Consuming {}", &self.tokens[0]);
            // We store the position of the lookahead token to support scanner switching.
            self.update_position();
            token = self.tokens.remove(0);
            self.ensure_buffer()?;
        }
        Ok(token)
    }

    // Update the line and column numbers from the token at index 0 in the token buffer.
    fn update_position(&mut self) {
        let token = &self.tokens[0];
        self.line = token.location.start_line;
        self.column = token.location.start_column + token.location.length;
        self.last_consumed_token_end_pos = token.location.offset;
        trace!(
            "Updated line: {}, column: {}, last consumed token end position: {}",
            self.line,
            self.column,
            self.last_consumed_token_end_pos
        );
    }

    ///
    /// Returns and thereby consumes the comments of this [`TokenStream`].
    ///
    pub fn drain_comments(&mut self) -> Vec<Token<'t>> {
        self.comments.drain(0..).collect()
    }

    ///
    /// Test if all input was processed by the parser
    ///
    pub fn all_input_consumed(&self) -> bool {
        self.tokens.is_empty() || self.tokens[0].token_type == super::EOI
    }

    ///
    /// Returns the last valid token from token buffer if there is one
    ///
    pub fn last_token(&self) -> Result<&Token<'_>, LexerError> {
        self.tokens
            .iter()
            .rev()
            .find(|t| t.token_type != super::EOI)
            .ok_or(LexerError::TokenBufferEmptyError)
    }

    ///
    /// Provides scanner state switching
    ///
    /// *Parser based scanner switching*
    ///
    /// Currently we take the stream position where we set the new scanner from
    /// the match of LA(1) token. More precisely all relevant positions after the match
    /// which had been stored in the token before. These positions are captured in the function
    /// `TokenStream::consume`.
    /// This is a documented restriction.
    ///
    /// A *parser based scanner switch* is executed by the parser itself when handling a `%sc`, a
    /// `%push` or a `%pop` directive.
    /// On `%sc` the parser calls `switch_scanner` with the clear flag set to `true`.
    /// On `%push` the parser calls `push_scanner` which clears the token buffer.
    /// On `%pop` the parser calls `pop_scanner` which also clears the token buffer.
    ///
    /// Thus, the parser always clears the token buffer after the switch.
    ///
    /// *Scanner based scanner switching*
    ///
    /// The `read_tokens` function actually executes the *scanner based scanner switch*.
    /// The clear flag is used to clear the token buffer after the switch.
    /// If the scanner switch is initiated by `read_tokens` the flag is set to `false` to keep
    /// the tokens in the buffer. The `read_tokens` stops reading tokens after the scanner switch
    /// is detected.
    ///
    /// *Return value*
    ///
    /// Currently this never return LexerError but it could be changed in the future.
    ///
    pub fn switch_scanner(&mut self, scanner_index: ScannerIndex) -> Result<usize, LexerError> {
        let mut tokens_read = 0usize;
        if self.token_iter.current_mode() == scanner_index {
            trace!(
                "Redundant switch to scanner {} <{}> omitted",
                scanner_index,
                self.scanner_mode_name(scanner_index),
            );
        } else {
            trace!(
                "Switching to scanner {} <{}>.",
                scanner_index,
                self.scanner_mode_name(scanner_index),
            );
            self.switch_to(scanner_index);
            self.clear_token_buffer();
            tokens_read = self.ensure_buffer()?;
        }
        Ok(tokens_read)
    }

    ///
    /// Push the current scanner index and switch to the scanner with given index.
    ///
    /// Currently this never return LexerError but it could be changed in the future.
    ///
    pub fn push_scanner(&mut self, scanner_index: ScannerIndex) -> Result<(), LexerError> {
        if self.token_iter.current_mode() == scanner_index {
            trace!(
                "push_scanner: Redundant switch to scanner {} <{}> omitted",
                scanner_index,
                self.scanner_mode_name(scanner_index),
            );
            self.scanner_stack.push(self.token_iter.current_mode());
        } else {
            trace!(
                "push_scanner: Pushing current scanner {} and switching to scanner {} <{}>.",
                self.token_iter.current_mode(),
                scanner_index,
                self.scanner_mode_name(scanner_index),
            );
            self.scanner_stack.push(self.token_iter.current_mode());
            self.switch_to(scanner_index);
            self.tokens.clear();
            self.ensure_buffer()?;
            trace!(
                "push_scanner: Resulting scanner stack: {:?}",
                self.scanner_stack
            );
        }
        Ok(())
    }

    ///
    /// Push the current scanner index and switch to the scanner with given index.
    ///
    pub fn pop_scanner(&mut self) -> Result<(), LexerError> {
        if let Some(scanner_index) = self.scanner_stack.pop() {
            if self.token_iter.current_mode() == scanner_index {
                trace!(
                    "pop_scanner: Redundant switch to scanner {} <{}> omitted",
                    scanner_index,
                    self.scanner_mode_name(scanner_index),
                );
            } else {
                trace!(
                    "pop_scanner: Switching to popped scanner {} <{}>.",
                    scanner_index,
                    self.scanner_mode_name(scanner_index),
                );
                self.switch_to(scanner_index);
                self.tokens.clear();
                self.ensure_buffer()?;
                trace!(
                    "pop_scanner: Resulting scanner stack: {:?}",
                    self.scanner_stack
                );
            }
            Ok(())
        } else {
            Err(LexerError::ScannerStackEmptyError)
        }
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
            if !token.is_skip_token() {
                tokens_read += 1;
                trace!("Read {}: {}", self.tokens.len(), token);
                self.tokens.push(token);
                if tokens_read >= n {
                    break;
                }
            } else if token.is_comment_token() {
                // Store comment ready for the user
                self.comments.push(token);
            }
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

    ///
    /// This function is used to setup a new TokenIter at the current stream
    /// position (aka scanner state switching).
    ///
    fn switch_to(&mut self, scanner_index: usize) {
        self.token_iter.set_mode(scanner_index);
        trace!(
            "Switched to scanner {} <{}>. Last consumed token's end position: {}",
            scanner_index,
            self.scanner_mode_name(scanner_index),
            self.last_consumed_token_end_pos
        );
        self.token_iter
            .set_position(self.last_consumed_token_end_pos);
    }

    pub(crate) fn token_types(&self) -> Vec<TerminalIndex> {
        self.tokens.iter().map(|t| t.token_type).collect::<Vec<_>>()
    }

    pub(crate) fn diagnostic_message(&self) -> String {
        format!(
            "Lookahead buffer:\n[\n  {}\n]\n",
            self.tokens
                .iter()
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
                self.tokens[index],
                index,
                token_type
            );
            if (self.tokens[index].token_type) == EOI {
                Err(LexerError::RecoveryError("Can't replace EOI".to_owned()))
            } else {
                self.tokens[index].token_type = token_type;
                Ok(())
            }
        } else {
            Err(LexerError::RecoveryError(
                "Can't replace beyond token buffer".to_owned(),
            ))
        }
    }

    pub(crate) fn insert_token_at(
        &mut self,
        index: usize,
        token_type: TerminalIndex,
    ) -> Result<(), LexerError> {
        if self.tokens.len() >= index {
            trace!("inserting token {} at index {}", token_type, index);
            let location = if self.tokens.len() > index {
                self.tokens[index].location.clone()
            } else {
                LocationBuilder::default()
                    .start_line(self.line)
                    .start_column(self.column)
                    .end_line(self.line)
                    .end_column(self.column)
                    .length(0)
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

    /// Returns the name of the scanner mode with the given index.
    #[inline]
    fn scanner_mode_name(&self, index: usize) -> &str {
        self.token_iter
            .scanner_mode_name(index)
            .unwrap_or("unknown")
    }

    /// Clears the token buffer.
    #[inline]
    fn clear_token_buffer(&mut self) {
        self.tokens.clear();
    }
}
