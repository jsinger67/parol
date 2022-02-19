use crate::analysis::compiled_la_dfa::TerminalIndex;
use crate::generators::NamingHelper as NmHlp;
use crate::{Cfg, Symbol, Terminal};
use parol_runtime::lexer::{BLOCK_COMMENT, EOI, LINE_COMMENT, NEW_LINE, WHITESPACE};

/// Generates a terminal name from a terminal definition
/// The parameter of type `Option<TerminalIndex>` is used to handle fixed terminal indices like EOI.
/// When only 'normal' terminal strings are processed and a terminal index is not relevant
/// simply provide None for this value.
pub fn generate_terminal_name(terminal: &str, i: Option<TerminalIndex>, cfg: &Cfg) -> String {
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
            Some(EOI) => "EndOfInput".to_owned(),
            Some(NEW_LINE) => "Newline".to_owned(),
            Some(WHITESPACE) => "Whitespace".to_owned(),
            Some(LINE_COMMENT) => "LineComment".to_owned(),
            Some(BLOCK_COMMENT) => "BlockComment".to_owned(),
            _ => {
                if let Some(nt) = primary_non_terminal(cfg, terminal) {
                    NmHlp::to_upper_camel_case(&nt)
                } else {
                    generate_name(terminal)
                }
            }
        }
    }
}
