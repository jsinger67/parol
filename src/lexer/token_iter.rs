use crate::lexer::{TerminalIndex, Token, Tokenizer, RX_NEW_LINE};
use log::trace;
use regex::CaptureMatches;

///
/// The TokenIter type provides iterator functionality for Token<'t> objects.
/// The lifetime parameter `'t` refers to the lifetime of the scanned text.
///
pub struct TokenIter<'t> {
    /// Line number
    line: usize,

    /// Column number
    col: usize,

    /// An iterator of capture groups
    capture_iter: CaptureMatches<'static, 't>,

    /// A list of valid group names. They are used to associate the token type
    /// with the matched text.
    group_names: Vec<String>,

    /// The lookahead size
    k: usize,
}

impl<'t> TokenIter<'t> {
    ///
    /// This function creates a token iterator from a tokenizer and an input.
    /// k determines the number of lookahead tokens the stream shall support.
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
            line: 1,
            col: 1,
            capture_iter: rx.rx.captures_iter(input),
            group_names,
            k,
        }
    }

    ///
    pub fn with_position(mut self, line: usize, column: usize) -> Self {
        self.line = line;
        self.col = column;
        self
    }

    ///
    /// Counts the occurrences of newlines in the given text.
    /// It is used to update the internal line number.
    ///
    pub(crate) fn count_nl(s: &str) -> usize {
        RX_NEW_LINE.find_iter(s).count()
    }

    ///
    /// Calculates the column position after the last matched newline.
    /// Is used to update the internal column number.
    ///
    pub(crate) fn calculate_col(s: &str) -> usize {
        let mut matches = RX_NEW_LINE.find_iter(s).collect::<Vec<_>>();
        let right_most_match = matches.pop().unwrap();
        s.len() - right_most_match.end() + 1
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
                // The symbol is taken from the match
                let symbol = ma.as_str();
                let length = symbol.len();
                // The token position is calculated from the matched text
                let line = self.line;
                let column = self.col;

                // Set the inner position behind the scanned token
                let new_lines = Self::count_nl(symbol);
                let pos = ma.end();
                self.line += new_lines;
                self.col = if new_lines > 0 {
                    Self::calculate_col(symbol)
                } else {
                    self.col + length
                };
                let token = Token::with(symbol, token_type, line, column, length, pos);
                trace!("{}, newline count: {}", token, new_lines);
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
