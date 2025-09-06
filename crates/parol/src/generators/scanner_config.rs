use crate::{GrammarConfig, parser::parol_grammar::ScannerStateSwitch};
use anyhow::{Result, bail};
use parol_runtime::{
    TerminalIndex,
    lexer::{
        BLOCK_COMMENT, ERROR_TOKEN, FIRST_USER_TOKEN, LINE_COMMENT, NEW_LINE, NEW_LINE_TOKEN,
        WHITESPACE, WHITESPACE_TOKEN,
    },
};
use std::fmt::{Debug, Display, Error, Formatter};

// Regular expression + terminal index + optional lookahead expression + generated token name
type TerminalMapping = (String, TerminalIndex, Option<(bool, String)>, String);
// Scanner transition is a tuple of terminal index and the name of the next scanner mode
type ScannerTransition = (TerminalIndex, ScannerStateSwitch);
// The build information is a tuple of terminal mappings and scanner transitions
type BuildInformation = (Vec<TerminalMapping>, Vec<ScannerTransition>);

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
///
/// Configuration information for a scanner.
/// Contains features like to optionally switch automatic handling off and newlines off.
///
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
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

    /// If true, unmatched input is allowed without error.
    pub allow_unmatched: bool,

    /// Scanner state transitions
    /// Maps from token to scanner state, where the token is identified by its TerminalIndex
    /// The scanner state is identified by its index.
    pub transitions: Vec<(TerminalIndex, ScannerStateSwitch)>,
}

impl ScannerConfig {
    /// Creates a new item
    pub fn new(scanner_name: String, scanner_state: usize) -> Self {
        Self {
            scanner_name,
            scanner_state,
            line_comments: Vec::new(),
            block_comments: Vec::new(),
            auto_newline: true,
            auto_ws: true,
            allow_unmatched: false,
            transitions: Vec::new(),
        }
    }

    /// Adds line comments to self
    pub fn with_line_comments(mut self, line_comments: Vec<String>) -> Self {
        self.line_comments = line_comments;
        self
    }

    /// Adds block comments to self
    pub fn with_block_comments(mut self, block_comments: Vec<(String, String)>) -> Self {
        self.block_comments = block_comments;
        self
    }

    /// Sets auto newline behavior
    pub fn with_auto_newline(mut self, auto_newline: bool) -> Self {
        self.auto_newline = auto_newline;
        self
    }

    /// Sets auto whitespace behavior
    pub fn with_auto_ws(mut self, auto_ws: bool) -> Self {
        self.auto_ws = auto_ws;
        self
    }

    /// Sets allow unmatched behavior
    pub fn with_allow_unmatched(mut self, allow_unmatched: bool) -> Self {
        self.allow_unmatched = allow_unmatched;
        self
    }

    /// Generates the data needed by the lexer generator.
    /// The tuple contains the mapping of terminal strings to their indices plus an optional
    /// lookahead pattern and the transitions, i.e. a mapping of terminal indices to scanner names.
    ///
    pub fn generate_build_information(
        &self,
        grammar_config: &GrammarConfig,
        terminal_names: &[String],
    ) -> Result<BuildInformation> {
        let cfg = &grammar_config.cfg;
        let mut terminal_mappings = Vec::new();
        if self.auto_newline {
            terminal_mappings.push((
                NEW_LINE_TOKEN.to_owned(),
                NEW_LINE,
                None,
                terminal_names[NEW_LINE as usize].clone(),
            ));
        }
        if self.auto_ws {
            terminal_mappings.push((
                WHITESPACE_TOKEN.to_owned(),
                WHITESPACE,
                None,
                terminal_names[WHITESPACE as usize].clone(),
            ));
        }
        if !self.line_comments.is_empty() {
            let line_comments_rx = self
                .line_comments
                .iter()
                .map(|s| format!(r###"{s}.*(\r\n|\r|\n)?"###))
                .collect::<Vec<String>>()
                .join("|");
            terminal_mappings.push((
                line_comments_rx,
                LINE_COMMENT,
                None,
                terminal_names[LINE_COMMENT as usize].clone(),
            ));
        }
        if !self.block_comments.is_empty() {
            let block_comments_rx = self
                .block_comments
                .iter()
                .map(|(s, e)| Self::format_block_comment(s, e))
                .collect::<Result<Vec<String>>>()?
                .join("|");
            terminal_mappings.push((
                block_comments_rx,
                BLOCK_COMMENT,
                None,
                terminal_names[BLOCK_COMMENT as usize].clone(),
            ));
        }

        let mut terminal_mappings = cfg.get_ordered_terminals().iter().enumerate().fold(
            terminal_mappings,
            |mut acc, (i, (t, k, l, s))| {
                if s.contains(&self.scanner_state) {
                    acc.push((
                        k.expand(t),
                        i as TerminalIndex + FIRST_USER_TOKEN,
                        l.as_ref()
                            .map(|l| (l.is_positive, l.kind.expand(&l.pattern))),
                        terminal_names[i + FIRST_USER_TOKEN as usize].clone(),
                    ));
                }
                acc
            },
        );
        // Add the error token as last terminal of the mode, unless allow_unmatched is set
        if !self.allow_unmatched {
            let error_index = terminal_names.len() - 1;
            terminal_mappings.push((
                ERROR_TOKEN.to_owned(),
                error_index as TerminalIndex,
                None,
                terminal_names[error_index].clone(),
            ));
        }

        Ok((terminal_mappings, self.transitions.clone()))
    }

    /// Formats a block comment
    /// The block comment is formatted as a regular expression.
    /// We need to specify the repeated expression for the comment content in such a way that
    /// the end of the comment is not matched.
    /// For this we need to allow only sequences that do not start with a substring of the end
    /// of the comment. Since the end comment can be any string, we need to build an alternation
    /// of all possible substrings of the end comment.
    /// If the comment end is "*/" the regular expression is:
    /// `r"/\*([^*]|\*[^/])*\*/"`
    fn format_block_comment(s: &str, e: &str) -> Result<String> {
        // Special case for /* ... */ block comments
        if s == r"/\*" && e == r"\*/" {
            // Use improved regex to match /***/ and similar cases
            return Ok(r"/\*/?([^/]|[^*]/)*\*/".to_string());
        }
        let len_with_escaped_chars = |s: &str| {
            let mut prev = None;
            s.chars()
                .map(|c| {
                    if c == '\\' && !matches!(prev, Some('\\')) {
                        prev = Some(c);
                        0
                    } else {
                        prev = Some(c);
                        1
                    }
                })
                .sum::<usize>()
        };
        Ok(match len_with_escaped_chars(e) {
            0 => bail!("Block comment end is empty."),
            1 => {
                let c0 = if e.chars().nth(0).unwrap() == '\\' {
                    if Self::must_escape_in_bracketed_expression(e.chars().nth(1).unwrap()) {
                        e.to_string()
                    } else {
                        e.chars().nth(1).unwrap().escape_default().to_string()
                    }
                } else {
                    e.to_string()
                };
                format!(r"{s}[^{c0}]*{e}")
            }
            2 => {
                let (c0, c1) = if e.chars().nth(0).unwrap() == '\\' {
                    (&e[0..2], &e[2..])
                } else {
                    (&e[0..1], &e[1..])
                };
                // We need to determine if the character is escaped or not, and if it is escaped
                // whether it is a regex meta character or not.
                // If it is a regex meta character we don't need to escape it in a bracket expression.
                let c0c = if c0.len() > 1 {
                    debug_assert_eq!(c0.chars().nth(0).unwrap(), '\\');
                    // Determine if the character after the escape is a regex meta character
                    if Self::must_escape_in_bracketed_expression(c0.chars().nth(1).unwrap()) {
                        c0.to_string()
                    } else {
                        c0.chars().nth(1).unwrap().escape_default().to_string()
                    }
                } else {
                    debug_assert_eq!(c0.len(), 1);
                    c0.to_string()
                };
                let c1c = if c1.len() > 1 {
                    debug_assert_eq!(c1.chars().nth(0).unwrap(), '\\');
                    // Determine if the character after the escape is a regex meta character
                    if Self::must_escape_in_bracketed_expression(c1.chars().nth(1).unwrap()) {
                        c1.to_string()
                    } else {
                        c1.chars().nth(1).unwrap().escape_default().to_string()
                    }
                } else {
                    debug_assert_eq!(c1.len(), 1);
                    c1.to_string()
                };
                format!(r"{s}([^{c0c}]|{c0}[^{c1c}])*{e}")
            }
            _ => bail!(
                r"Block comment end '{}' is too long. Maximum length is 2.
                Consider using manual comment handling, maybe with different scanner modes.",
                e
            ),
        })
    }

    fn must_escape_in_bracketed_expression(c: char) -> bool {
        matches!(c, '-' | ']' | '^' | '\\')
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
            allow_unmatched: false,
            transitions: Vec::new(),
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
        writeln!(f, "auto_ws: {:?}", self.auto_ws)?;
        self.transitions
            .iter()
            .try_for_each(|(k, v)| write!(f, "on {k} enter {v};"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use scnr2::scanner;

    fn format_matches(expected: &[scnr2::Match], input: &str) -> String {
        format!(
            "[{}]",
            expected
                .iter()
                .map(|m| format!(
                    "(\"{}\", {}, {})",
                    &input[m.span.start..m.span.end],
                    m.span.start,
                    m.span.end
                ))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    /// To help type inference in the macro
    fn format_expected_matches(expected: &[(&str, usize, usize)]) -> String {
        format!("{expected:?}")
    }

    macro_rules! scan_test {
        ($test_name:ident, $module:ident, $scanner:ident, $pattern:expr, $input:expr, $expected:expr, $test_num:expr) => {
            scanner! {
                $scanner {
                    mode M {
                        token $pattern => 0;
                    }
                }
            }
            #[test]
            fn $test_name() {
                use $module::$scanner as S;
                let scanner = S::new();
                let matches = scanner.find_matches($input, 0).collect::<Vec<_>>();
                const EXPECTED_MATCHES: &[(&str, usize, usize)] = $expected;
                assert_eq!(
                    matches.len(),
                    EXPECTED_MATCHES.len(),
                    "{}: Unexpected match count exp: {:?}, act: {:?}",
                    $test_num,
                    format_expected_matches(&EXPECTED_MATCHES),
                    format_matches(&matches, $input)
                );
                for (i, ma) in EXPECTED_MATCHES.iter().enumerate() {
                    assert_eq!(
                        matches[i].span.start, ma.1,
                        concat!($test_num, ": Match start does not match")
                    );
                    assert_eq!(
                        matches[i].span.end, ma.2,
                        concat!($test_num, ": Match end does not match")
                    );
                    assert_eq!(
                        &($input)[ma.1..ma.2],
                        ma.0,
                        concat!($test_num, ": Matched substring does not match expected")
                    );
                }
            }
        };
    }

    #[test]
    fn test_format_block_comment() {
        let s = r"/\*";
        let e = r"\*/";
        let r = ScannerConfig::format_block_comment(s, e);
        assert_eq!(r.unwrap(), r"/\*/?([^/]|[^*]/)*\*/");

        let s = r"\{\{";
        let e = r"\}\}";
        let r = ScannerConfig::format_block_comment(s, e);
        assert_eq!(r.unwrap(), r"\{\{([^}]|\}[^}])*\}\}");

        let s = "--";
        let e = "--";
        let r = ScannerConfig::format_block_comment(s, e);
        assert_eq!(r.unwrap(), r"--([^-]|-[^-])*--");

        let s = "#";
        let e = "#";
        let r = ScannerConfig::format_block_comment(s, e);
        assert_eq!(r.unwrap(), r"#[^#]*#");

        let s = r"\{";
        let e = r"\}";
        let r = ScannerConfig::format_block_comment(s, e);
        assert_eq!(r.unwrap(), r"\{[^}]*\}");
    }

    scan_test!(
        test_block_comment_1,
        scanner1,
        Scanner1,
        r"/\*/?([^/]|[^*]/)*\*/",
        "code /* comment */ more code",
        &[("/* comment */", 5, 18)],
        "Test 1: Simple block comment"
    );

    scan_test!(
        test_block_comment_2,
        scanner2,
        Scanner2,
        r"/\*/?([^/]|[^*]/)*\*/",
        "code /***/ more code /* comment */ /* com*ment */",
        &[
            ("/***/", 5, 10),
            ("/* comment */", 21, 34),
            ("/* com*ment */", 35, 49)
        ],
        "Test 2: Multiple block comments with stars inside"
    );

    // Tests for issue #828 - Edge cases with block comment parsing
    scan_test!(
        test_block_comment_empty,
        scanner3,
        Scanner3,
        r"/\*/?([^/]|[^*]/)*\*/",
        "code /**/ more code",
        &[("/**/", 5, 9)],
        "Test 3: Empty block comment"
    );

    scan_test!(
        test_block_comment_triple_star,
        scanner4,
        Scanner4,
        r"/\*/?([^/]|[^*]/)*\*/",
        "code /****/ more code",
        &[("/****/", 5, 11)],
        "Test 4: Triple star comment"
    );

    scan_test!(
        test_block_comment_start_end_token,
        scanner5,
        Scanner5,
        r"/\*/?([^/]|[^*]/)*\*/",
        "code /***/ more code",
        &[("/***/", 5, 10)],
        "Test 5: Block comment with only start of end token"
    );

    scan_test!(
        test_block_comment_regular_content,
        scanner6,
        Scanner6,
        r"/\*/?([^/]|[^*]/)*\*/",
        "/* normal comment */ /* another * comment */",
        &[
            ("/* normal comment */", 0, 20),
            ("/* another * comment */", 21, 44)
        ],
        "Test 6: Regular block comments with content"
    );

    scan_test!(
        test_block_comment_multiple_sequence,
        scanner7,
        Scanner7,
        r"/\*/?([^/]|[^*]/)*\*/",
        "/**/ /* a */ /****/ /* b*c */ /**/",
        &[
            ("/**/", 0, 4),
            ("/* a */", 5, 12),
            ("/****/", 13, 19),
            ("/* b*c */", 20, 29),
            ("/**/", 30, 34)
        ],
        "Test 7: Multiple block comments in sequence"
    );

    scan_test!(
        test_block_comment_complex_edge_cases,
        scanner8,
        Scanner8,
        r"/\*/?([^/]|[^*]/)*\*/",
        "/*/ not end */ /* ** */ /***/",
        &[
            ("/*/ not end */", 0, 14),
            ("/* ** */", 15, 23),
            ("/***/", 24, 29)
        ],
        "Test 8: Complex edge cases with various star patterns"
    );
}
