use crate::lexer::{location, Token, Tokenizer, RX_NEW_LINE};
use location::LocationBuilder;
use log::trace;
use regex_automata::dfa::{dense::DFA, regex::FindMatches};
use std::{borrow::Cow, path::Path};

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
    find_iter: FindMatches<'static, 't, DFA<Vec<u32>>>,

    /// The tokenizer itself
    rx: &'static Tokenizer,

    /// The input text
    pub(crate) input: &'t str,

    /// The lookahead size
    k: usize,

    /// The name of the input file
    pub file_name: Cow<'static, Path>,
}

impl<'t> TokenIter<'t> {
    ///
    /// This function creates a token iterator from a tokenizer and an input.
    /// k determines the number of lookahead tokens the stream shall support.
    ///
    pub fn new<T>(rx: &'static Tokenizer, input: &'t str, file_name: T, k: usize) -> Self
    where
        T: Into<Cow<'static, Path>>,
    {
        Self {
            line: 1,
            col: 1,
            find_iter: rx.rx.find_iter(input.as_bytes()),
            rx,
            input,
            k,
            file_name: file_name.into(),
        }
    }

    ///
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
        let matches = RX_NEW_LINE.find_iter(s.as_bytes()).collect::<Vec<_>>();
        let lines = matches.len() as u32;
        if let Some(&right_most_match) = matches.last().as_ref() {
            (lines, (s.len() - right_most_match.end()) as u32 + 1)
        } else {
            // Column number 0 means invalid
            (lines, 0)
        }
    }
}

impl<'t> Iterator for TokenIter<'t> {
    type Item = Token<'t>;
    fn next(&mut self) -> Option<Token<'t>> {
        if let Some(ref multi_match) = self.find_iter.next() {
            let token_type = self.rx.terminal_index_of_pattern(multi_match.pattern());
            // The token's text is taken from the match
            let text = &self.input[multi_match.range()];
            let length = text.len() as u32;
            // The token position is calculated from the matched text
            let start_line = self.line;
            let start_column = self.col;

            // Set the inner position behind the scanned token
            let (new_lines, column_after_nl) = Self::count_nl(text);
            let pos = multi_match.end();
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
                let token = Token::with(text, token_type, location);
                trace!("{}, newline count: {}", token, new_lines);
                Some(token)
            } else {
                // Error
                trace!("Error: Runtime builder error");
                None
            }
        } else if self.k > 0 {
            self.k -= 1;
            trace!("EOI");
            Some(Token::eoi())
        } else {
            trace!("Normal end of iteration");
            None
        }
    }
}
