use crate::{
    lexer::{location, Token},
    Location, TokenNumber,
};
use location::LocationBuilder;
use log::trace;
use scnr::{FindMatches, MatchExt, MatchExtIterator, Scanner, ScannerModeSwitcher, WithPositions};
use std::{path::PathBuf, sync::Arc};

///
/// The TokenIter type provides iterator functionality for Token<'t> objects.
/// The lifetime parameter `'t` refers to the lifetime of the scanned text.
///
pub struct TokenIter<'t> {
    /// An iterator over token matches
    pub(crate) find_iter: WithPositions<FindMatches<'t>>,

    /// The tokenizer itself
    pub(crate) scanner: Scanner,

    /// The input text
    pub(crate) input: &'t str,

    /// The lookahead size
    k: usize,

    /// The name of the input file
    pub file_name: Arc<PathBuf>,

    token_number: TokenNumber,

    last_location: Option<Location>,
}

impl<'t> TokenIter<'t> {
    ///
    /// This function creates a token iterator from a tokenizer and an input.
    /// k determines the number of lookahead tokens the stream shall support.
    ///
    pub fn new(scanner: Scanner, input: &'t str, file_name: Arc<PathBuf>, k: usize) -> Self {
        Self {
            find_iter: scanner.find_iter(input).with_positions(),
            scanner,
            input,
            k,
            file_name: file_name.clone(),
            token_number: 0,
            last_location: None,
        }
    }

    /// Returns the name of the scanner mode with the given index.
    pub(crate) fn scanner_mode_name(&self, index: usize) -> Option<&str> {
        self.scanner.mode_name(index)
    }

    /// Returns the index of the current scanner mode.
    pub(crate) fn current_mode(&self) -> usize {
        self.scanner.current_mode()
    }

    pub(crate) fn token_from_match(&mut self, matched: MatchExt) -> Option<Token<'t>> {
        let token_type = matched.token_type();
        if let Ok(location) = LocationBuilder::default()
            .start_line(matched.start_position().line as u32)
            .start_column(matched.start_position().column as u32)
            .end_line(matched.end_position().line as u32)
            .end_column(matched.end_position().column as u32)
            .length(matched.len() as u32)
            .offset(matched.end())
            .file_name(self.file_name.clone())
            .build()
        {
            self.last_location = Some(location.clone());

            // The token's text is taken from the match
            let text = &self.input[matched.range()];
            let token = Token::with(text, token_type as u16, location, self.token_number);

            if !token.is_skip_token() || token.is_comment_token() {
                self.token_number += 1;
            }
            Some(token)
        } else {
            // Error
            trace!("Error: Runtime builder error");
            None
        }
    }

    /// Set the iterator to the given position.
    ///
    /// If the position is equal to the current position, the function does nothing.
    /// If the position is greater than the current position, the function advances the iterator to
    /// the given position.
    /// If the position is less than the current position, the function creates a new iterator and
    /// advances it to the given position.
    pub fn set_position(&mut self, position: usize) {
        self.find_iter = self
            .scanner
            .find_iter(self.input)
            .with_offset(position)
            .with_positions();
    }

    pub(crate) fn set_mode(&mut self, scanner_index: usize) {
        self.scanner.set_mode(scanner_index);
        self.find_iter.set_mode(scanner_index);
    }

    #[inline]
    pub(crate) fn next_token_number(&mut self) -> TokenNumber {
        self.token_number += 1;
        self.token_number
    }
}

impl<'t> Iterator for TokenIter<'t> {
    type Item = Token<'t>;
    fn next(&mut self) -> Option<Token<'t>> {
        if let Some(matched) = self.find_iter.next() {
            self.token_from_match(matched)
        } else if self.k > 0 {
            // Return at most k EOI tokens
            self.k -= 1;
            trace!("EOI");
            let mut eoi = Token::eoi(self.next_token_number());
            if let Some(location) = self.last_location.as_mut() {
                location.end_column += 1;
                eoi = eoi.with_location(location.clone());
            }
            Some(eoi)
        } else {
            trace!("Normal end of iteration");
            None
        }
    }
}
