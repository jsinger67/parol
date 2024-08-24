use crate::{
    lexer::{location, Token, RX_NEW_LINE},
    Location, TokenNumber,
};
use location::LocationBuilder;
use log::trace;
use scnr::{scanner::Scanner, FindMatches};
use std::{path::PathBuf, sync::Arc};

///
/// The TokenIter type provides iterator functionality for Token<'t> objects.
/// The lifetime parameter `'t` refers to the lifetime of the scanned text.
///
pub struct TokenIter<'t> {
    /// Line number, starting with 1
    line: u32,

    /// Column number, starting with 1
    col: u32,

    /// An iterator over token matches
    pub(crate) find_iter: FindMatches<'t>,

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
            line: 1,
            col: 1,
            find_iter: scanner.find_iter(input),
            scanner,
            input,
            k,
            file_name: file_name.clone(),
            token_number: 0,
            last_location: None,
        }
    }

    /// Sets the initial position of the iterator.
    pub fn with_position(mut self, line: u32, column: u32) -> Self {
        self.line = line;
        self.col = column;
        self
    }

    ///
    /// Counts the occurrences of newlines in the given text.
    /// If at least one newline is counted it also calculates the column position after the last
    /// matched newline.
    /// It is used to update `line` and `col` members.
    ///
    /// Returns a tuple of line count and new column number.
    ///
    pub(crate) fn count_nl(s: &str) -> (u32, u32) {
        let matches = RX_NEW_LINE.find_iter(s).collect::<Vec<_>>();
        let lines = matches.len() as u32;
        let result = if let Some(&right_most_match) = matches.last().as_ref() {
            (lines, (s.len() - right_most_match.end()) as u32 + 1)
        } else {
            // Column number 0 means invalid
            (lines, 0)
        };
        trace!("count_nl: {}, {}", result.0, result.1);
        result
    }

    /// Returns the name of the scanner mode with the given index.
    pub(crate) fn scanner_mode_name(&self, index: usize) -> Option<&str> {
        self.scanner.mode_name(index)
    }

    /// Returns the index of the current scanner mode.
    pub(crate) fn current_mode(&self) -> usize {
        self.scanner.current_mode()
    }

    pub(crate) fn token_from_match(&mut self, matched: scnr::Match) -> Option<Token<'t>> {
        let token_type = matched.token_type();
        // The token's text is taken from the match
        let text = &self.input[matched.range()];
        let length = text.len() as u32;
        // The token position is calculated from the matched text
        let start_line = self.line;
        let start_column = self.col;

        // Set the inner position behind the scanned token
        let (new_lines, column_after_nl) = Self::count_nl(text);
        let pos = matched.end();
        self.line += new_lines;
        self.col = if new_lines > 0 {
            column_after_nl
        } else {
            debug_assert!(column_after_nl == 0);
            self.col + length
        };
        if let Ok(location) = LocationBuilder::default()
            .start_line(start_line)
            .start_column(start_column)
            .end_line(start_line + new_lines)
            .end_column(self.col)
            .length(length)
            .offset(pos)
            .file_name(self.file_name.clone())
            .build()
        {
            self.last_location = Some(location.clone());
            let token = Token::with(text, token_type as u16, location, self.token_number);
            if !token.is_skip_token() || token.is_comment_token() {
                self.token_number += 1;
            }
            trace!("{}, newline count: {}", token, new_lines);
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
        self.find_iter = self.scanner.find_iter(self.input).with_offset(position);
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
            let mut eoi = Token::eoi(self.token_number);
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
