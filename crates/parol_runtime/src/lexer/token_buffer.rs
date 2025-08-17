//! This module provides a buffer for tokens, allowing to collect and manage tokens
//! from a lexer.
//! It handles both skip tokens and non-skip tokens, providing methods to manipulate
//! the token stream, such as adding tokens, consuming them, and iterating over them.
//!
//! Skip tokens are tokens that are not relevant for the parsing process, such as whitespace
//! or comments. They are typically ignored by the parser, but they are built into the
//! so called lossless parse tree, which can be used by users to understand the structure
//! of the input text.
use super::Token;
use crate::{LexerError, TokenNumber};

/// Buffer for tokens
#[derive(Debug, Default)]
pub struct TokenBuffer<'t> {
    tokens: Vec<Token<'t>>,
    last_token_location: u32,
    last_token_number: TokenNumber,
}

impl<'t> TokenBuffer<'t> {
    /// Creates a new instance
    pub fn new() -> TokenBuffer<'t> {
        TokenBuffer {
            tokens: Vec::new(),
            last_token_location: 0,
            last_token_number: 0,
        }
    }

    /// Adds a token to the buffer
    pub fn add(&mut self, token: Token<'t>, input: &'t str) {
        let new_start = token.location.start;
        if self.last_token_location < new_start {
            use crate::lexer::location::Location;
            use crate::lexer::token::INVALID_TOKEN;
            let gap_location = Location {
                start: self.last_token_location,
                end: new_start,
                file_name: token.location.file_name.clone(),
                ..Location::default()
            };
            let invalid_token = Token::with(
                std::convert::Into::<std::borrow::Cow<'t, str>>::into(
                    &input[gap_location.start as usize..gap_location.end as usize],
                ),
                INVALID_TOKEN,
                gap_location,
                self.last_token_number + 1,
            );
            self.tokens.push(invalid_token);
        }
        self.last_token_location = token.location.end;
        self.last_token_number = token.token_number;
        self.tokens.push(token);
    }

    /// Returns the number of tokens in the buffer
    /// It only counts non-skip-tokens
    pub fn len(&self) -> usize {
        self.tokens.iter().filter(|t| !t.is_skip_token()).count()
    }

    /// Returns skip tokens at the beginning of the buffer.
    /// The skip tokens are removed from the buffer.
    pub fn take_skip_tokens(&mut self) -> Vec<Token<'t>> {
        let split_index = self.tokens.iter().take_while(|t| t.is_skip_token()).count();
        self.tokens.drain(..split_index).collect()
    }

    /// Returns the token types of the tokens in the lookahead buffer.
    /// It only considers non-skip-tokens.
    pub fn non_skip_token_types(&self) -> Vec<u16> {
        self.tokens
            .iter()
            .filter(|t| !t.is_skip_token())
            .map(|t| t.token_type)
            .collect()
    }

    /// Returns an iterator over the tokens in the buffer.
    /// It only considers non-skip-tokens.
    pub fn non_skip_tokens(&self) -> impl Iterator<Item = &Token<'t>> {
        self.tokens.iter().filter(|t| !t.is_skip_token())
    }

    /// Returns a reversed iterator over the tokens in the buffer.
    /// It only considers non-skip-tokens.
    pub fn non_skip_tokens_rev(&self) -> impl Iterator<Item = &Token<'t>> {
        self.tokens.iter().rev().filter(|t| !t.is_skip_token())
    }

    /// Returns the non-skip-token at the given index.
    pub fn non_skip_token_at(&self, index: usize) -> Option<&Token<'t>> {
        self.tokens.iter().filter(|t| !t.is_skip_token()).nth(index)
    }

    /// Returns the non-skip-token at the given index as mutable reference.
    pub fn non_skip_token_at_mut(&mut self, index: usize) -> Option<&mut Token<'t>> {
        self.tokens
            .iter_mut()
            .filter(|t| !t.is_skip_token())
            .nth(index)
    }

    /// Inserts a non-skip-token at the given index, where the index is the index of the
    /// non-skip-tokens.
    pub fn insert(&mut self, index: usize, to_insert: Token<'t>) {
        let mut skip_count = 0;
        let mut insert_index = self.tokens.len(); // Default to end if index is out of bounds
        for (i, token) in self.tokens.iter().enumerate() {
            if !token.is_skip_token() {
                if skip_count == index {
                    insert_index = i;
                    break;
                }
                skip_count += 1;
            }
        }
        self.tokens.insert(insert_index, to_insert);
    }

    /// Remove all tokens from the buffer
    pub fn clear(&mut self) {
        self.tokens.clear();
    }

    /// Returns true if the buffer contains only skip tokens
    pub fn is_empty(&self) -> bool {
        self.tokens.iter().all(|t| t.is_skip_token())
    }

    /// Returns true if the buffer is completely empty
    pub fn is_buffer_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    /// Removes the first non-skip token from the buffer.
    /// Fails if the buffer is empty or if there are skip tokens at the beginning of the buffer.
    /// They should have been removed and inserted into the lossless parse tree before calling this
    /// function.
    pub fn consume(&mut self) -> Result<Token<'t>, LexerError> {
        if self.tokens.is_empty() {
            return Err(LexerError::InternalError(
                "Try to consume from an empty buffer".to_string(),
            ));
        }
        if self.tokens[0].is_skip_token() {
            return Err(LexerError::InternalError(format!(
                "Try to consume with skip tokens at the beginning of the buffer: {:?}",
                self.tokens[0]
            )));
        }
        Ok(self.tokens.remove(0))
    }
}

impl std::fmt::Display for TokenBuffer<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for token in &self.tokens {
            write!(f, "{token}, ")?;
        }
        write!(f, "]")
    }
}
