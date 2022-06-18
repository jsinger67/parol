//! This module provides functionality for generating variable names and type names so that they
//! adhere to the Rust naming conventions.

const KEYWORDS: &[&str; 52] = &[
    "abstract", "as", "async", "await", "become", "box", "break", "const", "continue", "crate",
    "do", "dyn", "else", "enum", "extern", "false", "final", "fn", "for", "if", "impl", "in",
    "let", "loop", "macro", "match", "mod", "move", "mut", "override", "priv", "pub", "ref",
    "return", "Self", "self", "static", "struct", "super", "trait", "true", "try", "type",
    "typeof", "union", "unsafe", "unsized", "use", "virtual", "where", "while", "yield",
];

/// We need to decide if a user given type has to be included via `use` statement.
/// Actually we should include all types that are present by default through the Rust prelude.
const BUILT_IN_TYPES: &[&str; 18] = &[
    "i8", "u8", "i16", "u16", "i32", "u32", "i64", "u64", "i128", "u128", "isize", "usize", "f32",
    "f64", "char", "str", "bool", "String",
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

    /// Checks whether the given name is a built-in Rust type
    /// ```
    /// use parol::generators::NamingHelper as NmHlp;
    /// assert!(!NmHlp::is_rust_built_in_type("Tuple"));
    /// assert!(NmHlp::is_rust_built_in_type("bool"));
    /// ```
    pub fn is_rust_built_in_type(name: &str) -> bool {
        BUILT_IN_TYPES.iter().any(|ty| ty == &name)
    }

    /// Checks whether the symbol starts with "r#"
    ///
    /// ```
    /// use parol::generators::naming_helper::NamingHelper;
    ///
    /// assert!(NamingHelper::is_raw_identifier("r#let"));
    /// assert!(!NamingHelper::is_raw_identifier("_let"));
    /// ```
    pub fn is_raw_identifier(name: &str) -> bool {
        name.starts_with("r#")
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

    /// Returns an underscore string if the item is not used.
    ///
    /// ```
    /// use parol::generators::naming_helper::NamingHelper;
    ///
    /// assert_eq!(NamingHelper::add_unused_indicator(false, "r#let"), "_let");
    /// assert_eq!(NamingHelper::add_unused_indicator(false, "x"), "_x");
    /// assert_eq!(NamingHelper::add_unused_indicator(true, "x"), "x");
    /// ```
    pub fn add_unused_indicator(used: bool, name: &str) -> String {
        if !used && !Self::is_raw_identifier(name) {
            format!("_{}", name)
        } else if !used {
            name.to_string().replace("r#", "_")
        } else {
            name.to_string()
        }
    }

    ///
    /// Produces a lower snake camel case version of the given name.
    /// Since these names are supposed to be used as identifiers a clash with rust keywords is
    /// detected and prevented.
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

    /// This is a very restrictive definition of allowed characters in identifiers `parol` allows.
    /// Invalid characters in terminal names, non-terminal names and module names are typically
    /// replaced by the underscore character and can later be removed during name generations using
    /// special casing rules like UpperCamelCase.
    ///
    /// ```
    /// use parol::generators::naming_helper::NamingHelper;
    ///
    /// assert!(NamingHelper::is_valid_name_character('a'));
    /// assert!(NamingHelper::is_valid_name_character('A'));
    /// assert!(NamingHelper::is_valid_name_character('1'));
    /// assert!(NamingHelper::is_valid_name_character('_'));
    ///
    /// assert!(!NamingHelper::is_valid_name_character('-'));
    /// assert!(!NamingHelper::is_valid_name_character(':'));
    /// assert!(!NamingHelper::is_valid_name_character('/'));
    /// ```
    pub fn is_valid_name_character(c: char) -> bool {
        c.is_alphanumeric() || c == '_'
    }

    /// Replaces all invalid characters from the given name with underscores. It is used to process
    /// user given names which are for instance given as command arguments.
    ///
    /// # Examples
    ///
    /// ```
    /// use parol::generators::naming_helper::NamingHelper;
    ///
    /// assert_eq!(NamingHelper::purge_name("test-module"), "test_module");
    /// ```
    pub fn purge_name(name: &str) -> String {
        let mut result = String::with_capacity(name.len());
        for c in name.chars() {
            result.push(if Self::is_valid_name_character(c) {
                c
            } else {
                '_'
            });
        }
        result
    }
}
