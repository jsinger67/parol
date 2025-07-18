use crate::{
    TokenNumber,
    lexer::{Token, location},
};
use location::LocationBuilder;
use log::trace;
use scnr2::{FindMatchesWithPosition, Match};
use std::{path::PathBuf, sync::Arc};

///
/// The TokenIter type provides iterator functionality for Token<'t> objects.
/// The lifetime parameter `'t` refers to the lifetime of the scanned text.
///
pub struct TokenIter<'t, F>
where
    F: Fn(char) -> Option<usize> + 'static + Clone,
{
    /// An iterator over token matches
    pub(crate) find_iter: FindMatchesWithPosition<'t, F>,

    /// The input text
    pub(crate) input: &'t str,

    /// The lookahead size
    k: usize,

    /// The name of the input file
    pub file_name: Arc<PathBuf>,

    token_number: TokenNumber,
}

impl<'t, F> TokenIter<'t, F>
where
    F: Fn(char) -> Option<usize> + 'static + Clone,
{
    ///
    /// This function creates a token iterator from a tokenizer and an input.
    /// k determines the number of lookahead tokens the stream shall support.
    ///
    pub fn new(
        find_iter: FindMatchesWithPosition<'t, F>,
        input: &'t str,
        file_name: Arc<PathBuf>,
        k: usize,
    ) -> Self {
        Self {
            find_iter,
            input,
            k,
            file_name: file_name.clone(),
            token_number: 0,
        }
    }

    /// Returns the name of the scanner mode with the given index.
    pub(crate) fn scanner_mode_name(&self, index: usize) -> Option<&'static str> {
        self.find_iter.mode_name(index)
    }

    /// Returns the index of the current scanner mode.
    pub(crate) fn current_mode(&self) -> usize {
        self.find_iter.current_mode()
    }

    #[inline(always)]
    pub(crate) fn token_from_match(&mut self, matched: Match) -> Option<Token<'t>> {
        let positions = matched.positions?;
        let location = LocationBuilder::default()
            .start_line(positions.start_position.line as u32)
            .start_column(positions.start_position.column as u32)
            .end_line(positions.end_position.line as u32)
            .end_column(positions.end_position.column as u32)
            .start(matched.span.start as u32)
            .end(matched.span.end as u32)
            .file_name(Arc::clone(&self.file_name))
            .build()
            .ok()?;

        let text = &self.input[matched.span];
        let token = Token::with(text, matched.token_type as u16, location, self.token_number);

        // Only increment for non-skip or comment tokens
        if !token.is_skip_token() || token.is_comment_token() {
            self.token_number += 1;
        }
        Some(token)
    }
    #[inline]
    pub(crate) fn next_token_number(&mut self) -> TokenNumber {
        self.token_number += 1;
        self.token_number
    }
}

impl<'t, F> Iterator for TokenIter<'t, F>
where
    F: Fn(char) -> Option<usize> + 'static + Clone,
{
    type Item = Token<'t>;
    fn next(&mut self) -> Option<Token<'t>> {
        if let Some(matched) = self.find_iter.next() {
            self.token_from_match(matched)
        } else if self.k > 0 {
            // Return at most k EOI tokens
            self.k -= 1;
            trace!("EOI");
            Some(Token::eoi(self.next_token_number()))
        } else {
            trace!("Normal end of iteration");
            None
        }
    }
}
