use crate::errors::*;
use crate::lexer::{OwnedToken, Token};
use crate::lexer::{TerminalIndex, TokenIter, Tokenizer, EOI};
use crate::parser::ScannerAccess;
use log::trace;

///
/// The lifetime parameter `'t` refers to the lifetime of the scanned text.
///
pub struct TokenStream<'t> {
    /// The number of available lookahead tokens
    pub k: usize,
    // The input text
    input: &'t str,
    /// The name of the input file
    pub file_name: String,
    /// The index of the error token, obtained from the tokenizer
    error_token_type: TerminalIndex,
    /// The actual token iterator
    token_iter: TokenIter<'t>,
    /// A slice with named tokenizers
    tokenizers: &'static [(&'static str, Tokenizer)],
    /// All available tokens
    pub tokens: Vec<Token<'t>>,
}

impl<'t> TokenStream<'t> {
    ///
    /// Creates a new TokenStream object from an augmented terminals list and
    /// an input string.
    /// The k determines the number of lookahead tokens the stream supports.
    ///
    pub fn new(
        input: &'t str,
        file_name: String,
        tokenizers: &'static [(&'static str, Tokenizer)],
        k: usize,
    ) -> Result<TokenStream<'t>> {
        let mut token_stream = TokenStream {
            k,
            input,
            file_name,
            error_token_type: tokenizers[0].1.error_token_type,
            token_iter: TokenIter::new(&tokenizers[0].1, input, k),
            tokenizers,
            tokens: Vec::with_capacity(k),
        };
        token_stream.read_tokens(k);
        Ok(token_stream)
    }

    ///
    /// Provides at maximum k tokens lookahead relative to the current read
    /// position.
    /// If successful it returns an owned token from buffer position self.pos + n
    ///
    pub fn owned_lookahead(&mut self, n: usize) -> Result<OwnedToken> {
        if n > self.k {
            Err("Lookahead exceeds its maximum".into())
        } else {
            // Fill buffer to lookahead size k relative to pos
            self.ensure_buffer();
            if n >= self.tokens.len() {
                Err("Lookahead exceeds token buffer length".into())
            } else {
                trace!("LA({}): {}", n, self.tokens[n]);
                Ok(self.tokens[n].to_owned())
            }
        }
    }

    ///
    /// Provides at maximum k tokens lookahead relative to the current read
    /// position.
    /// If successful it returns the type (index) of the token at buffer
    /// position self.pos + n
    ///
    pub fn lookahead_token_type(&mut self, n: usize) -> Result<TerminalIndex> {
        if n > self.k {
            Err("Lookahead exceeds its maximum".into())
        } else {
            // Fill buffer to lookahead size k relative to pos
            self.ensure_buffer();
            if n >= self.tokens.len() {
                Err("Lookahead exceeds token buffer length".into())
            } else {
                trace!("Type(LA({})): {}", n, self.tokens[n]);
                Ok(self.tokens[n].token_type)
            }
        }
    }

    ///
    /// Advances the current position in the token buffer by the given number
    /// of tokens.
    /// If necessary more input is read via the token_iter.
    ///
    pub fn consume(&mut self) -> Result<()> {
        self.ensure_buffer();
        if self.tokens.is_empty() {
            Err("Consume on empty buffer is impossible".into())
        } else {
            trace!("Consuming {}", self.tokens[0]);
            self.tokens.remove(0);
            Ok(())
        }
    }

    ///
    /// Test if all input was processed by the parser
    ///
    pub fn all_input_consumed(&self) -> bool {
        self.tokens.is_empty() || self.tokens[0].token_type == EOI
    }

    ///
    /// Read only access to the index of the error token
    /// Needed by the parser.
    ///
    pub fn error_token_type(&self) -> TerminalIndex {
        self.error_token_type
    }

    fn read_tokens(&mut self, n: usize) -> usize {
        let mut tokens_read = 0usize;
        for token in &mut self.token_iter {
            if !token.is_skip_token() {
                tokens_read += 1;
                trace!("Read {}: {}", self.tokens.len(), token);
                self.tokens.push(token);
                if tokens_read >= n {
                    break;
                }
            }
        }
        tokens_read
    }

    ///
    /// The function tries to fill the buffer (self.tokens) with a k tokens
    /// lookahead buffer.
    /// It returns the number of tokens read.
    ///
    fn ensure_buffer(&mut self) -> usize {
        let last_buffer_index = self.tokens.len();
        if last_buffer_index < self.k {
            // Fill buffer to lookahead size k relative to pos
            self.read_tokens(self.k - last_buffer_index)
        } else {
            0
        }
    }
}

impl ScannerAccess for TokenStream<'_> {
    fn switch_scanner(&mut self, scanner_name: &str) -> std::result::Result<(), Error> {
        if let Some(scanner_index) = self.tokenizers.iter().position(|(n, _)| *n == scanner_name) {
            self.token_iter = self
                .token_iter
                .switch_to(&self.tokenizers[scanner_index].1, self.input);
            Ok(())
        } else {
            Err(format!("Unknown scanner: {}", scanner_name).into())
        }
    }
}
