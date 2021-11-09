use crate::analysis::compiled_la_dfa::TerminalIndex;
use crate::{Cfg, Symbol, Terminal};
use parol_runtime::lexer::{BLOCK_COMMENT, EOI, LINE_COMMENT, NEW_LINE, WHITESPACE};

pub fn generate_terminal_name(terminal: &str, i: TerminalIndex, cfg: &Cfg) -> String {
    fn primary_non_terminal(cfg: &Cfg, terminal: &str) -> Option<String> {
        cfg.pr
            .iter()
            .find(|r| {
                if r.len() == 1 {
                    if let Symbol::T(Terminal::Trm(n, _)) = &r.1[0] {
                        n == terminal && cfg.matching_productions(&r.get_n()).len() == 1
                    } else {
                        false
                    }
                } else {
                    false
                }
            })
            .map(|r| r.get_n())
    }
    fn capitalize(s: String) -> String {
        if !s.is_empty() {
            let mut prev_char = '_';
            let mut res = String::new();
            for c in s.chars() {
                if prev_char == '_' {
                    res.push(c.to_uppercase().next().unwrap());
                } else if c != '_' {
                    res.push(c);
                }
                prev_char = c;
            }
            res
        } else {
            s
        }
    }
    fn generate_name(s: &str) -> String {
        let (_, name) = s
            .chars()
            .fold((true, String::new()), |(mut cap, mut acc), c| {
                if c.is_alphanumeric() {
                    if cap {
                        acc.push(c.to_uppercase().next().unwrap());
                    } else {
                        acc.push(c);
                    }
                    cap = false;
                } else {
                    acc.push_str(match c {
                        '\\' => "",
                        '|' => "Or",
                        '(' => "LParen",
                        ')' => "RParen",
                        '[' => "LBracket",
                        ']' => "RBracket",
                        '{' => "LBrace",
                        '}' => "RBrace",
                        '+' => "Plus",
                        '-' => "Minus",
                        '*' => "Star",
                        '/' => "Slash",
                        '=' => "Equ",
                        '!' => "Bang",
                        '.' => "Dot",
                        '~' => "Tilde",
                        '$' => "Dollar",
                        '%' => "Percent",
                        '<' => "LT",
                        '>' => "GT",
                        '?' => "Quest",
                        '@' => "At",
                        ':' => "Colon",
                        ';' => "Semicolon",
                        '^' => "Circumflex",
                        '_' => "Underscore",
                        '&' => "Amp",
                        '§' => "Para",
                        '\'' => "Tick",
                        '"' => "Quote",
                        '`' => "Backtick",
                        '´' => "Accent",
                        ',' => "Comma",
                        '#' => "Hash",
                        _ => "_",
                    });
                    cap = true;
                }
                (cap, acc)
            });
        if name.is_empty() && !s.is_empty() {
            "Esc".to_owned()
        } else {
            name
        }
    }
    if terminal == "ERROR_TOKEN" {
        "Error".to_owned()
    } else {
        match i {
            EOI => "EndOfInput".to_owned(),
            NEW_LINE => "Newline".to_owned(),
            WHITESPACE => "Whitespace".to_owned(),
            LINE_COMMENT => "LineComment".to_owned(),
            BLOCK_COMMENT => "BlockComment".to_owned(),
            _ => {
                if let Some(nt) = primary_non_terminal(cfg, terminal) {
                    capitalize(nt)
                } else {
                    generate_name(terminal)
                }
            }
        }
    }
}
