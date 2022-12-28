use crate::parser::ScannerIndex;
use crate::{LexerError, TerminalIndex, Token, TokenIter, Tokenizer};
use anyhow::{bail, Result};
use log::trace;
use std::borrow::Cow;

use std::path::Path;

///
/// The TokenStream<'t> type is the interface the parser actually uses.
/// It provides the lookahead functionality by maintaining a lookahead buffer.
/// Also it provides the ability to switch scanner states. This is handled by
/// choosing a Tokenizer and updating its position in the input text.
///
/// The lifetime parameter `'t` refers to the lifetime of the scanned text.
///
pub struct TokenStream<'t> {
    /// The number of available lookahead tokens
    pub k: usize,

    /// The input text
    pub(crate) input: &'t str,

    /// The name of the input file
    pub file_name: Cow<'static, Path>,

    /// The index of the error token, obtained from the tokenizer
    error_token_type: TerminalIndex,

    /// The actual token iterator.
    /// It is replaced by a new one in case of scanner state switch.
    token_iter: TokenIter<'t>,

    /// A slice with named tokenizers, which operate in combination with the
    /// TokenIter like a scanner.
    tokenizers: &'static [(&'static str, Tokenizer)],

    /// Lookahead token buffer, maximum size is k
    pub tokens: Vec<Token<'t>>,

    /// Start position in the input text as byte offset.
    /// Can be greater than zero, if `self` was created during a
    /// scanner state switch before.
    start_pos: usize,

    /// Relative position from start of input as byte offset at the last point
    /// the scanner was switched, is 0 initially. Needed for scanner switching.
    pos: usize,

    /// Line number of last consumed token. Needed for scanner switching. Is initially 1.
    line: usize,

    /// Columns after last consumed token. Needed for scanner switching. Is initially 1.
    column: usize,

    /// Index of the current scanner state, is 0 initially.
    pub current_scanner_index: usize,

    /// Scanner stack to support push and pop operations for scanner configurations
    scanner_stack: Vec<ScannerIndex>,
}

impl<'t> TokenStream<'t> {
    ///
    /// Creates a new TokenStream object from an augmented terminals list and
    /// an input string.
    /// The k determines the number of lookahead tokens the stream supports.
    ///
    pub fn new<T>(
        input: &'t str,
        file_name: T,
        tokenizers: &'static [(&'static str, Tokenizer)],
        k: usize,
    ) -> Result<Self>
    where
        T: AsRef<Path>,
    {
        let file_name: Cow<Path> = file_name.as_ref().to_owned().into();
        let token_iter = TokenIter::new(&tokenizers[0].1, input, file_name.clone(), k);

        let mut token_stream = Self {
            k,
            input,
            file_name,
            error_token_type: tokenizers[0].1.error_token_type,
            token_iter,
            tokenizers,
            tokens: Vec::with_capacity(k),
            start_pos: 0,
            pos: 0,
            line: 1,
            column: 1,
            current_scanner_index: 0,
            scanner_stack: Vec::new(),
        };
        token_stream.read_tokens(k);
        Ok(token_stream)
    }

    ///
    /// Provides at maximum k tokens lookahead relative to the current read
    /// position.
    /// If successful it returns an owned token from buffer position self.pos + n
    ///
    pub fn lookahead(&mut self, n: usize) -> Result<Token<'t>> {
        if n > self.k {
            bail!("Lookahead exceeds its maximum");
        } else {
            // Fill buffer to lookahead size k relative to pos
            self.ensure_buffer();
            if n >= self.tokens.len() {
                bail!("Lookahead exceeds token buffer length");
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
    pub fn lookahead_token_type(&mut self, n: usize) -> Result<TerminalIndex> {
        if n > self.k {
            bail!("Lookahead exceeds its maximum");
        } else {
            // Fill buffer to lookahead size k relative to pos
            self.ensure_buffer();
            if n >= self.tokens.len() {
                bail!("Lookahead exceeds token buffer length");
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
    pub fn consume(&mut self) -> Result<()> {
        self.ensure_buffer();
        if self.tokens.is_empty() {
            bail!("Consume on empty buffer is impossible");
        } else {
            // Store positions of last latest consumed token for scanner switching.
            // Actually this is token LA(1) with buffer index 0.
            let la1 = &self.tokens[0];
            let (new_lines, column) = TokenIter::count_nl(la1.text());
            self.pos = la1.location.offset;
            self.line = la1.location.start_line + new_lines;
            self.column = if new_lines > 0 {
                column
            } else {
                la1.location.start_column + la1.location.length
            };
            trace!(
                "Consuming {}, Stream position is {}. Line {}, Column {}",
                la1,
                self.pos,
                self.line,
                self.column,
            );
            self.tokens.remove(0);
            self.ensure_buffer();
            Ok(())
        }
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
    pub fn last_token(&self) -> Result<&Token<'_>> {
        self.tokens
            .iter()
            .rev()
            .find(|t| t.token_type != super::EOI)
            .ok_or_else(|| LexerError::TokenBufferEmptyError.into())
    }

    ///
    /// Read only access to the index of the error token
    /// Needed by the parser.
    ///
    pub fn error_token_type(&self) -> TerminalIndex {
        self.error_token_type
    }

    ///
    /// Provides scanner state switching
    ///
    /// Currently we take the stream position where we set the new scanner from
    /// the match of LA(1) token. More precisely all relevant positions after the match
    /// which had been stored in the token before. These positions are captured in the function
    /// `TokenStream::consume`.
    /// This is a documented restriction.
    ///
    pub fn switch_scanner(&mut self, scanner_index: ScannerIndex) -> Result<()> {
        if self.current_scanner_index == scanner_index {
            trace!(
                "Redundant switch to scanner {} <{}> omitted",
                scanner_index,
                self.tokenizers[scanner_index].0,
            );
        } else {
            trace!(
                "Switching to scanner {} <{}>; Current offset is {}",
                scanner_index,
                self.tokenizers[scanner_index].0,
                self.pos,
            );
            self.token_iter = self.switch_to(scanner_index);
            self.current_scanner_index = scanner_index;
            self.tokens.clear();
            self.ensure_buffer();
        }
        Ok(())
    }

    ///
    /// Push the current scanner index and switch to the scanner with given index.
    ///
    pub fn push_scanner(&mut self, scanner_index: ScannerIndex) -> Result<()> {
        if self.current_scanner_index == scanner_index {
            trace!(
                "push_scanner: Redundant switch to scanner {} <{}> omitted",
                scanner_index,
                self.tokenizers[scanner_index].0,
            );
            self.scanner_stack.push(self.current_scanner_index);
            self.current_scanner_index = scanner_index;
        } else {
            trace!(
                "push_scanner: Pushing current scanner {} and switching to scanner {} <{}>; Current offset is {}",
                self.current_scanner_index,
                scanner_index,
                self.tokenizers[scanner_index].0,
                self.pos,
            );
            self.scanner_stack.push(self.current_scanner_index);
            self.token_iter = self.switch_to(scanner_index);
            self.current_scanner_index = scanner_index;
            self.tokens.clear();
            self.ensure_buffer();
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
    pub fn pop_scanner(&mut self) -> Result<()> {
        if let Some(scanner_index) = self.scanner_stack.pop() {
            if self.current_scanner_index == scanner_index {
                trace!(
                    "pop_scanner: Redundant switch to scanner {} <{}> omitted",
                    scanner_index,
                    self.tokenizers[scanner_index].0,
                );
            } else {
                trace!(
                    "pop_scanner: Switching to popped scanner {} <{}>; Current offset is {}",
                    scanner_index,
                    self.tokenizers[scanner_index].0,
                    self.pos,
                );
                self.token_iter = self.switch_to(scanner_index);
                self.current_scanner_index = scanner_index;
                self.tokens.clear();
                self.ensure_buffer();
                trace!(
                    "pop_scanner: Resulting scanner stack: {:?}",
                    self.scanner_stack
                );
            }
            Ok(())
        } else {
            bail!("pop_scanner: Tried to pop from an empty scanner stack!")
        }
    }

    ///
    /// Returns the name of the currently active scanner state.
    /// Used for diagnostics.
    ///
    pub fn current_scanner(&self) -> &str {
        self.tokenizers[self.current_scanner_index].0
    }

    fn read_tokens(&mut self, n: usize) -> usize {
        let mut tokens_read = 0usize;
        for mut token in &mut self.token_iter {
            if !token.is_skip_token() {
                tokens_read += 1;
                trace!("Read {}: {}", self.tokens.len(), token);
                token.location.scanner_switch_pos = self.start_pos;
                self.tokens.push(token);
                if tokens_read >= n {
                    break;
                }
            }
        }
        tokens_read
    }

    ///
    /// The function fills the lookahead buffer (self.tokens) with k tokens.
    /// It returns the number of tokens read.
    ///
    fn ensure_buffer(&mut self) -> usize {
        let fill_len = self.tokens.len();
        if fill_len < self.k {
            // Fill buffer to lookahead size k
            self.read_tokens(self.k - fill_len)
        } else {
            0
        }
    }

    ///
    /// This function is used to setup a new TokenIter at the current stream
    /// position (aka scanner state switching).
    ///
    fn switch_to(&mut self, scanner_index: usize) -> TokenIter<'t> {
        self.start_pos += self.pos;
        self.pos = 0;
        let (_, input) = self.input.split_at(self.start_pos);
        TokenIter::new(
            &self.tokenizers[scanner_index].1,
            input,
            self.file_name.clone(),
            self.k,
        )
        .with_position(self.line, self.column)
    }
}
