//! This module provides functionality for generating variable names and type names so that they
//! adhere to the Rust naming conventions.

use crate::{Symbol, Terminal};

const KEYWORDS: &[&str; 52] = &[
    "abstract", "as", "async", "await", "become", "box", "break", "const", "continue", "crate",
    "do", "dyn", "else", "enum", "extern", "false", "final", "fn", "for", "if", "impl", "in",
    "let", "loop", "macro", "match", "mod", "move", "mut", "override", "priv", "pub", "ref",
    "return", "Self", "self", "static", "struct", "super", "trait", "true", "try", "type",
    "typeof", "union", "unsafe", "unsized", "use", "virtual", "where", "while", "yield",
];

/// Struct used as namespace only
pub struct NamingHelper;

impl NamingHelper {
    /// Checks whether the given name is a reserved Rust keyword
    /// ```
    /// use parol::generators::NamingHelper as NmHlp;
    /// assert!(!NmHlp::is_rust_keyword("Type"));
    /// assert!(NmHlp::is_rust_keyword("type"));
    /// ```
    pub fn is_rust_keyword(name: &str) -> bool {
        KEYWORDS.iter().any(|kw| kw == &name)
    }

    /// If the given name is a reserved Rust keyword it is converted into a raw identifier
    /// ```
    /// use parol::generators::NamingHelper as NmHlp;
    /// assert_eq!("Type".to_string(), NmHlp::escape_rust_keyword("Type".to_string()));
    /// assert_eq!("r#type".to_string(), NmHlp::escape_rust_keyword("type".to_string()));
    /// ```
    pub fn escape_rust_keyword(name: String) -> String {
        if Self::is_rust_keyword(&name) {
            format!("r#{}", name)
        } else {
            name
        }
    }

    /// Returns an underscore string if the item is not used.
    pub fn item_unused_indicator(used: bool) -> &'static str {
        if used {
            ""
        } else {
            "_"
        }
    }

    ///
    /// Produces a lower snake camel case version of the given name.
    /// Since these names are supposed to be used as identifiers a clash with rust keywords is detected
    /// and prevented.
    ///
    /// ```
    /// use parol::generators::NamingHelper as NmHlp;
    /// assert_eq!("prolog0", NmHlp::to_lower_snake_case("Prolog0"));
    /// assert_eq!("_prolog_0_", NmHlp::to_lower_snake_case("_prolog_0_"));
    /// assert_eq!("_prolog_0_1_3", NmHlp::to_lower_snake_case("_prolog_0_1__3"));
    /// assert_eq!("_", NmHlp::to_lower_snake_case("_____"));
    /// assert_eq!("calc_lst1_1", NmHlp::to_lower_snake_case("calc_lst1_1"));
    /// assert_eq!("nor_op_23", NmHlp::to_lower_snake_case("nor_op_23"));
    /// assert_eq!("r#type", NmHlp::to_lower_snake_case("type"));
    /// ```
    pub fn to_lower_snake_case(name: &str) -> String {
        let mut last_char = '.';
        Self::escape_rust_keyword(name.chars().fold(String::new(), |mut acc, c| {
            if acc.is_empty() {
                acc.push(c.to_lowercase().next().unwrap())
            } else if c == '_' {
                if !acc.ends_with('_') {
                    acc.push('_');
                }
            } else if c.is_ascii_digit() && last_char.is_ascii_alphabetic() {
                acc.push(c.to_lowercase().next().unwrap())
            } else if c.is_ascii_uppercase() {
                if !acc.ends_with('_') {
                    acc.push('_');
                }
                acc.push(c.to_lowercase().next().unwrap())
            } else {
                acc.push(c);
            }
            last_char = c;
            acc
        }))
    }

    ///
    /// Produces an upper camel case version of the given name.
    /// Separated numbers are kept separated.
    /// Camel case compliant input should be preserved.
    ///
    /// ```
    /// use parol::generators::NamingHelper as NmHlp;
    /// assert_eq!("Prolog0", NmHlp::to_upper_camel_case("_prolog_0"));
    /// assert_eq!("Prolog0", NmHlp::to_upper_camel_case("_prolog_0_"));
    /// assert_eq!("Prolog0", NmHlp::to_upper_camel_case("_prolog_0__"));
    /// assert_eq!("Prolog0_1", NmHlp::to_upper_camel_case("_prolog_0__1"));
    /// assert_eq!("Prolog0_1_10_20", NmHlp::to_upper_camel_case("_prolog_0__1_10___20__"));
    /// assert_eq!("Prolog0A", NmHlp::to_upper_camel_case("_prolog_0__a"));
    /// assert_eq!("PrologAA", NmHlp::to_upper_camel_case("_prolog_a_a"));
    /// assert_eq!("PrologItem", NmHlp::to_upper_camel_case("prolog_item"));
    /// assert_eq!("PrologItem", NmHlp::to_upper_camel_case("PrologItem"));
    /// assert_eq!("AA", NmHlp::to_upper_camel_case("_a_a_"));
    /// ```
    pub fn to_upper_camel_case(name: &str) -> String {
        let mut up = true;
        let mut last_char = '.';
        name.chars().fold(String::new(), |mut acc, c| {
            if c == '_' {
                up = true;
            } else if up {
                if last_char.is_ascii_digit() && c.is_ascii_digit() {
                    acc.push('_');
                }
                last_char = c.to_uppercase().next().unwrap();
                acc.push(last_char);
                up = false;
            } else {
                last_char = c;
                acc.push(last_char);
            }
            acc
        })
    }

    /// Generates a member name from a symbol that stems from a production's right-hand side
    pub fn generate_member_name(
        s: &Symbol,
        arg_index: usize,
        terminals: &[String],
        terminal_names: &[String],
    ) -> String {
        let get_terminal_index = |tr: &str| terminals.iter().position(|t| *t == tr).unwrap();
        match s {
            Symbol::N(n, _) => {
                format!("{}_{}", Self::to_lower_snake_case(n), arg_index)
            }
            Symbol::T(Terminal::Trm(t, _)) => {
                let terminal_name = &terminal_names[get_terminal_index(t)];
                format!("{}_{}", Self::to_lower_snake_case(terminal_name), arg_index)
            }
            _ => panic!("Invalid symbol type {}", s),
        }
    }

    /// Convenience function
    pub fn generate_member_names(
        rhs: &[Symbol],
        terminals: &[String],
        terminal_names: &[String],
    ) -> Vec<String> {
        rhs.iter()
            .enumerate()
            .filter(|(_, s)| s.is_n() || s.is_t())
            .map(|(i, s)| Self::generate_member_name(s, i, terminals, terminal_names))
            .collect::<Vec<String>>()
    }
}
