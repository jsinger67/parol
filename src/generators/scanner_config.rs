use crate::Cfg;
use std::fmt::{Debug, Display, Error, Formatter};

///
/// Configuration information for a scanner.
/// Contains features like to optionally switch automatic handling off newlines off.
///
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct ScannerConfig {
    ///
    /// The name of the scanner state taken from the grammar description
    ///
    pub scanner_name: String,

    ///
    /// Index of the scanner, aka scanner state
    ///
    pub scanner_state: usize,

    ///
    /// Strings with the characters that starts line comments
    ///
    pub line_comments: Vec<String>,

    ///
    /// (String, String) tuples with the characters that start and end
    /// a block comments, respectively.
    ///
    pub block_comments: Vec<(String, String)>,

    ///
    /// If true the lexer handles (and skips) newlines.
    /// If false the user has to handle newlines on its own.
    ///
    pub auto_newline: bool,

    ///
    /// If true the lexer handles (and skips) whitespace.
    /// If false the user has to handle whitespace on its own.
    ///
    pub auto_ws: bool,
}

impl ScannerConfig {
    pub fn new(scanner_name: String, scanner_state: usize) -> Self {
        Self {
            scanner_name,
            scanner_state,
            line_comments: Vec::new(),
            block_comments: Vec::new(),
            auto_newline: true,
            auto_ws: true,
        }
    }

    pub fn with_line_comments(mut self, line_comments: Vec<String>) -> Self {
        self.line_comments = line_comments;
        self
    }

    pub fn with_block_comments(mut self, block_comments: Vec<(String, String)>) -> Self {
        self.block_comments = block_comments;
        self
    }

    pub fn with_auto_newline(mut self, auto_newline: bool) -> Self {
        self.auto_newline = auto_newline;
        self
    }

    pub fn with_auto_ws(mut self, auto_ws: bool) -> Self {
        self.auto_ws = auto_ws;
        self
    }

    ///
    /// Generates the augmented tokens vector in the format needed by the lexer
    /// generator.
    ///
    pub fn generate_augmented_terminals(&self, cfg: &Cfg) -> Vec<String> {
        let mut terminals = vec![
            "UNMATCHABLE_TOKEN".to_owned(),
            if self.auto_newline {
                "NEW_LINE_TOKEN".to_owned()
            } else {
                "UNMATCHABLE_TOKEN".to_owned()
            },
            if self.auto_ws {
                "WHITESPACE_TOKEN".to_owned()
            } else {
                "UNMATCHABLE_TOKEN".to_owned()
            },
        ];
        if !self.line_comments.is_empty() {
            let line_comments_rx = self
                .line_comments
                .iter()
                .map(|s| format!(r###"({}.*(\r\n|\r|\n|$))"###, s))
                .collect::<Vec<String>>()
                .join("|");
            terminals.push(line_comments_rx);
        } else {
            terminals.push("UNMATCHABLE_TOKEN".to_owned());
        }
        if !self.block_comments.is_empty() {
            let block_comments_rx = self
                .block_comments
                .iter()
                .map(|(s, e)| format!(r###"((?ms){}.*?{})"###, s, e))
                .collect::<Vec<String>>()
                .join("|");
            terminals.push(block_comments_rx);
        } else {
            terminals.push("UNMATCHABLE_TOKEN".to_owned());
        }

        let mut terminals =
            cfg.get_ordered_terminals()
                .iter()
                .fold(terminals, |mut acc, (t, s)| {
                    if *s == self.scanner_state {
                        acc.push(t.to_string());
                    }
                    acc
                });

        terminals.push("ERROR_TOKEN".to_owned());
        terminals
    }
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            scanner_name: "INITIAL".to_string(),
            scanner_state: 0,
            line_comments: Vec::new(),
            block_comments: Vec::new(),
            auto_newline: true,
            auto_ws: true,
        }
    }
}

impl Display for ScannerConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        writeln!(f, "scanner_name: {}", self.scanner_name)?;
        writeln!(f, "scanner_state: {}", self.scanner_state)?;
        writeln!(f, "line_comments: {:?}", self.line_comments)?;
        writeln!(f, "block_comments: {:?}", self.block_comments)?;
        writeln!(f, "auto_newline: {:?}", self.auto_newline)?;
        writeln!(f, "auto_ws: {:?}", self.auto_ws)
    }
}
