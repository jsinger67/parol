use super::super::fmt_options::FmtOptions;

/// Offset of the first character of a continuation line, i.e. the number of
/// characters added by the line break itself.
pub(super) const START_LINE_OFFSET: usize = 6;

/// A single token of the grammar language as rendered during formatting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Token {
    pub(super) text: String,
    start: usize,
    end: usize,
}

impl Token {
    fn is_pipe(&self) -> bool {
        self.text == "|"
    }
}

/// Splits formatted grammar source into its tokens. Quoted strings, regex
/// literals and identifiers are kept as single tokens.
fn tokenize(text: &str) -> Vec<Token> {
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let byte_of = |idx: usize| -> usize {
        if idx == 0 {
            0
        } else {
            chars[idx - 1].0 + chars[idx - 1].1.len_utf8()
        }
    };
    let is_word_char = |c: char| c.is_alphanumeric() || c == '_';
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i].1.is_whitespace() {
            i += 1;
            continue;
        }
        let start = i;
        let c = chars[i].1;
        let end = if c == '"' || c == '\'' {
            let mut j = i + 1;
            while j < chars.len() && chars[j].1 != c {
                j += 1;
            }
            if j < chars.len() {
                j += 1;
            }
            j
        } else if c == '/' {
            let mut j = i + 1;
            while j < chars.len() && chars[j].1 != '/' {
                if chars[j].1 == '\\' {
                    j += 1;
                }
                j += 1;
            }
            if j < chars.len() {
                j += 1;
            }
            j
        } else if is_word_char(c) {
            let mut j = i;
            while j < chars.len() && is_word_char(chars[j].1) {
                j += 1;
            }
            j
        } else {
            i + 1
        };
        tokens.push(Token {
            text: text[byte_of(start)..byte_of(end)].to_string(),
            start: byte_of(start),
            end: byte_of(end),
        });
        i = end;
    }
    tokens
}

/// Computes the length of the rendered token sequence, i.e. the number of
/// characters of the formatted source it consists of.
fn rendered_length(tokens: &[Token]) -> usize {
    let mut length = 0;
    for (i, token) in tokens.iter().enumerate() {
        length += token.end - token.start;
        if let Some(next) = tokens.get(i + 1) {
            length += next.start - token.end;
        }
    }
    length
}

/// Checks whether the given (possibly wrapped) right-hand side of a
/// production or a wrapper exceeds the maximum line length.
pub(super) fn rhs_exceeds_max_line_length(options: &FmtOptions, rhs: &str) -> bool {
    let tokens = tokenize(rhs);
    rendered_length(&tokens) + START_LINE_OFFSET > options.max_line_length
}

/// Splits a formatted alternatives string into its alternatives, i.e. into
/// the token sequences separated by the alternation operator.
pub(super) fn token_alternatives(text: &str) -> Vec<String> {
    let tokens = tokenize(text);
    let mut alternatives: Vec<String> = Vec::new();
    let mut current: Vec<Token> = Vec::new();
    let flush = |alternatives: &mut Vec<String>, current: &mut Vec<Token>| {
        if !current.is_empty() {
            let first = current.first().unwrap();
            let last = current.last().unwrap();
            alternatives.push(text[first.start..last.end].trim().to_string());
        }
        current.clear();
    };
    for token in &tokens {
        if token.is_pipe() {
            flush(&mut alternatives, &mut current);
        } else {
            current.push(token.clone());
        }
    }
    flush(&mut alternatives, &mut current);
    alternatives
}

/// Flattens wrapped alternatives into a single logical line.
pub(super) fn normalize_to_single_line(text: &str, space_around_pipe: bool) -> String {
    let separator = if space_around_pipe { " | " } else { "|" };
    token_alternatives(text).join(separator)
}

/// Renders the alternatives, one alternative per line, with the alternation
/// operator prefixed to all but the first one.
pub(super) fn split_alternatives_on_lines(
    alternatives: &[String],
    space_around_pipe: bool,
    indent: &str,
) -> String {
    let mut result = String::new();
    for (i, alternative) in alternatives.iter().enumerate() {
        if i > 0 {
            result.push('\n');
            result.push_str(indent);
            result.push('|');
            if space_around_pipe {
                result.push(' ');
            }
        }
        result.push_str(alternative);
    }
    result
}
