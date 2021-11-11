use crate::lexer::{TerminalIndex, Token, Tokenizer, RX_NEW_LINE};
use log::trace;
use regex::CaptureMatches;

///
/// The lifetime parameter `'t` refers to the lifetime of the scanned text.
///
pub struct TokenIter<'t> {
    // Start position, can be greater than zero, if self was switched before
    start_pos: usize,
    // Relative position from start position
    pos: usize,
    line: usize,
    col: usize,
    capture_iter: CaptureMatches<'static, 't>,
    group_names: Vec<String>,
    k: usize,
}

impl<'t> TokenIter<'t> {
    ///
    /// This creates a token iterator from a tokenizer and an input source.
    /// The k determines the number of lookahead tokens the stream supports.
    ///
    pub fn new(rx: &'static Tokenizer, input: &'t str, k: usize) -> TokenIter<'t> {
        let group_names: Vec<String> = rx
            .rx
            .capture_names()
            .flatten()
            .filter(|n| n.starts_with('G'))
            .map(String::from)
            .collect();
        TokenIter {
            start_pos: 0,
            pos: 0,
            line: 1,
            col: 1,
            capture_iter: rx.rx.captures_iter(input),
            group_names,
            k,
        }
    }

    ///
    /// This function is used to setup a new TokenIter (aka scanner state
    /// switching) by updating all inner position values on the newly created
    /// TokenIter.
    ///
    pub(crate) fn switch_to(&self, rx: &'static Tokenizer, input: &'t str) -> TokenIter<'t> {
        let start_pos = self.start_pos + self.pos;
        let (_, input) = input.split_at(start_pos);
        let group_names: Vec<String> = rx
            .rx
            .capture_names()
            .flatten()
            .filter(|n| n.starts_with('G'))
            .map(String::from)
            .collect();
        TokenIter {
            start_pos,
            pos: 0,
            line: self.line,
            col: self.col,
            capture_iter: rx.rx.captures_iter(input),
            group_names,
            k: self.k,
        }
    }

    fn count_nl(&self, s: &str) -> usize {
        RX_NEW_LINE.find_iter(s).count()
    }

    fn calculate_col(&self, s: &str) -> usize {
        let mut matches = RX_NEW_LINE.find_iter(s).collect::<Vec<_>>();
        let right_most_match = matches.pop().unwrap();
        s.len() - right_most_match.end() + 1
    }

    #[cfg(test)]
    pub fn named_groups(&self) -> &Vec<String> {
        &self.group_names
    }
}

impl<'t> Iterator for TokenIter<'t> {
    type Item = Token<'t>;
    fn next(&mut self) -> Option<Token<'t>> {
        if let Some(ref captures) = self.capture_iter.next() {
            let group_name_opt = self.group_names.iter().find(|g| captures.name(g).is_some());
            let ca_opt = group_name_opt.map(|g| captures.name(g).unwrap());

            if let Some(ma) = ca_opt {
                // Token type is taken from the group name
                let group_name = group_name_opt.unwrap();
                let token_type = TerminalIndex::from_str_radix(&group_name[1..], 10).unwrap();
                // The symbol ist taken from the match
                let symbol = ma.as_str();
                let length = symbol.len();
                // The token position is calculated from the matched text
                let line = self.line;
                let column = self.col;

                // Set the inner position behind the scanned token
                let new_lines = self.count_nl(symbol);
                self.pos = ma.end();
                self.line += new_lines;
                self.col = if new_lines > 0 {
                    self.calculate_col(symbol)
                } else {
                    self.col + length
                };
                let token = Token::with(symbol, token_type, line, column, length);
                trace!("{}", token);
                Some(token)
            } else {
                // Error
                trace!("Error: End of iteration - no match");
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
