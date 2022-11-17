use super::{Decorate, SymbolAttribute};
use crate::analysis::k_tuple::TerminalMappings;
use crate::parser::parol_grammar::{Factor, UserDefinedTypeName};
use crate::parser::to_grammar_config::try_from_factor;
use miette::{IntoDiagnostic, Result};
use parol_runtime::parser::ScannerIndex;
use std::convert::TryFrom;
use std::fmt::{Debug, Display, Error, Formatter, Write};

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
///
/// Determines how the terminal literal is interpreted by parol
///
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum TerminalKind {
    /// Uninterpreted by parol, same as regex - the old double quoted form
    Legacy,
    /// Uninterpreted by parol, usually used for regular expressions
    Regex,
    /// Meta characters will be escaped when regex for scanner is created
    Raw,
}

impl TerminalKind {
    /// Retrieves the syntactic delimiter character
    pub fn delimiter(&self) -> char {
        match self {
            TerminalKind::Legacy => '"',
            TerminalKind::Regex => '/',
            TerminalKind::Raw => '\'',
        }
    }
}

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
///
/// A terminal symbol with different specificities
///
#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum Terminal {
    ///
    /// A physical terminal symbol with the scanner states it belongs to
    /// Entities that are provided by the lexer.
    ///
    Trm(
        String,
        TerminalKind,
        Vec<usize>,
        SymbolAttribute,
        Option<UserDefinedTypeName>,
    ),

    ///
    /// Epsilon symbol, the empty word
    /// Can be contained in FIRST sets
    /// Cannot be contained in productions of our definition of grammar -
    /// epsilon productions are simply empty.
    ///
    Eps,

    ///
    /// End of input symbol, End of grammar symbol (not belonging to any grammar)
    ///
    End,
}

impl Terminal {
    /// Creates a terminal
    pub fn t(t: &str, s: Vec<usize>, a: SymbolAttribute) -> Self {
        Self::Trm(t.to_owned(), TerminalKind::Legacy, s, a, None)
    }
    /// Checks if self is a terminal
    pub fn is_trm(&self) -> bool {
        matches!(self, Self::Trm(..))
    }
    /// Checks if self is an epsilon
    pub fn is_eps(&self) -> bool {
        matches!(self, Self::Eps)
    }
    /// Checks if self is an end of input terminal
    pub fn is_end(&self) -> bool {
        matches!(self, Self::End)
    }

    /// Creates a terminal from a [Symbol]
    pub fn create(s: &Symbol) -> Self {
        match s {
            Symbol::T(Terminal::Trm(t, k, s, a, u)) => {
                Terminal::Trm(t.to_string(), *k, s.to_vec(), *a, u.clone())
            }
            Symbol::T(Terminal::End) => Terminal::End,
            _ => panic!("Unexpected symbol type: {:?}", s),
        }
    }

    /// Adds a scanner index
    pub fn add_scanner(&mut self, sc: usize) {
        match self {
            Terminal::Trm(_, _, s, _, _) => {
                if !s.contains(&sc) {
                    s.push(sc);
                    s.sort_unstable();
                }
            }
            _ => panic!("Unexpected symbol type: {:?}", self),
        }
    }

    ///
    /// Formats self with the help of a scanner state resolver
    ///
    pub fn format<R, S>(&self, scanner_state_resolver: &R, user_type_resolver: &S) -> Result<String>
    where
        R: Fn(&[usize]) -> String,
        S: Fn(&str) -> Option<String>,
    {
        match self {
            Self::Trm(t, k, s, a, u) => {
                let mut d = String::new();
                let delimiter = k.delimiter();
                a.decorate(&mut d, &format!("{}{}{}", delimiter, t, delimiter))
                    .into_diagnostic()?;
                if let Some(ref user_type) = u {
                    let user_type =
                        if let Some(alias) = user_type_resolver(user_type.to_string().as_str()) {
                            alias
                        } else {
                            user_type.to_string()
                        };
                    write!(d, " : {}", user_type).into_diagnostic()?;
                }
                if *s == vec![0] {
                    // Don't print state if terminal is only in state INITIAL (0)
                    Ok(d)
                } else {
                    Ok(format!("<{}>{}", scanner_state_resolver(s), d))
                }
            }
            Self::Eps => Ok("\u{03B5}".to_string()), // Lower creek letter Epsilon (ε)
            Self::End => Ok("$".to_string()),
        }
    }
}

impl Display for Terminal {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Self::Trm(t, k, ..) => {
                let delimiter = k.delimiter();
                write!(f, "{}{}{}", delimiter, t, delimiter)
            }
            Self::Eps => write!(f, "\u{03B5}"), // Lower creek letter Epsilon (ε)
            Self::End => write!(f, "$"),
        }
    }
}

impl TerminalMappings<Terminal> for Terminal {
    fn eps() -> Terminal {
        Self::Eps
    }

    fn end() -> Terminal {
        Self::End
    }

    fn is_eps(&self) -> bool {
        self.is_eps()
    }

    fn is_end(&self) -> bool {
        self.is_end()
    }
}

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
///
/// A grammar symbol with different specificities
///
#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum Symbol {
    ///
    /// Non-terminal symbol, Meta symbol of the grammar.
    ///
    N(String, SymbolAttribute, Option<UserDefinedTypeName>),

    ///
    /// Terminal symbol of the grammar.
    ///
    T(Terminal),

    ///
    /// Instruction to switch scanner state
    ///
    S(ScannerIndex),

    ///
    /// Instruction to push the index of the current scanner and switch to a scanner configuration
    /// with the given index
    ///
    Push(ScannerIndex),

    ///
    /// Instruction to pop the index of the scanner pushed before and switch to the scanner
    /// configuration with that index
    ///
    Pop,
}

impl Symbol {
    // /// Creates a terminal symbol
    // pub fn t(t: &str, s: Vec<usize>, a: SymbolAttribute) -> Self {
    //     Self::T(Terminal::Trm(t.to_owned(), s, a, None))
    // }
    // /// Creates a terminal symbol with default symbol attribute
    // pub fn t_n(t: &str, s: Vec<usize>) -> Self {
    //     Self::T(Terminal::Trm(
    //         t.to_owned(),
    //         s,
    //         SymbolAttribute::default(),
    //         None,
    //     ))
    // }
    /// Creates a non-terminal symbol
    pub fn n(n: &str) -> Self {
        Self::N(n.to_owned(), SymbolAttribute::default(), None)
    }
    /// Creates a end-of-input terminal symbol
    pub fn e() -> Self {
        Self::T(Terminal::End)
    }
    /// Creates a scanner index
    pub fn s(s: usize) -> Self {
        Self::S(s)
    }
    /// Checks if self is a terminal
    pub fn is_t(&self) -> bool {
        matches!(self, Self::T(_))
    }
    /// Checks if self is a non-terminal
    pub fn is_n(&self) -> bool {
        matches!(self, Self::N(..))
    }
    /// Checks if self is a end-of-input terminal
    pub fn is_end(&self) -> bool {
        matches!(self, Self::T(Terminal::End))
    }
    /// Checks if self is a scanner switch instruction
    pub fn is_switch(&self) -> bool {
        matches!(self, Self::S(_)) || matches!(self, Self::Push(_)) || matches!(self, Self::Pop)
    }
    /// Returns a terminal if available
    pub fn get_t(&self) -> Option<Terminal> {
        if let Self::T(t) = &self {
            Some(t.clone())
        } else {
            None
        }
    }
    /// Returns a terminal reference if available
    pub fn get_t_ref(&self) -> Option<&Terminal> {
        if let Self::T(t) = &self {
            Some(t)
        } else {
            None
        }
    }
    /// Returns a non-terminal if available
    pub fn get_n(&self) -> Option<String> {
        if let Self::N(n, ..) = &self {
            Some(n.clone())
        } else {
            None
        }
    }
    /// Returns a non-terminal reference if available
    pub fn get_n_ref(&self) -> Option<&str> {
        if let Self::N(n, ..) = &self {
            Some(n)
        } else {
            None
        }
    }

    /// Get the symbol attribute or a default value
    pub fn attribute(&self) -> SymbolAttribute {
        match self {
            Symbol::N(_, a, _) | Symbol::T(Terminal::Trm(_, _, _, a, _)) => *a,
            _ => SymbolAttribute::None,
        }
    }

    /// Formats self with the help of a scanner state resolver
    pub fn format<R, S>(&self, scanner_state_resolver: &R, user_type_resolver: &S) -> Result<String>
    where
        R: Fn(&[usize]) -> String,
        S: Fn(&str) -> Option<String>,
    {
        match self {
            Self::N(n, a, u) => {
                let mut s = String::new();
                a.decorate(&mut s, n).into_diagnostic()?;
                if let Some(ref user_type) = u {
                    let user_type =
                        if let Some(alias) = user_type_resolver(user_type.to_string().as_str()) {
                            alias
                        } else {
                            user_type.to_string()
                        };
                    write!(s, " : {}", user_type).into_diagnostic()?;
                }
                Ok(s)
            }
            Self::T(t) => t.format(scanner_state_resolver, user_type_resolver),
            Self::S(s) => {
                if *s == 0 {
                    Ok("%sc()".to_string())
                } else {
                    Ok(format!("%sc({})", scanner_state_resolver(&[*s])))
                }
            }
            Self::Push(s) => Ok(format!("%push({})", scanner_state_resolver(&[*s]))),
            Self::Pop => Ok("%pop()".to_string()),
        }
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Self::N(n, a, u) => {
                let mut s = String::new();
                a.decorate(&mut s, n)?;
                if let Some(ref user_type) = u {
                    write!(s, " : {} ", user_type)?;
                }
                write!(f, "{}", s)
            }
            Self::T(t) => {
                let mut d = String::new();
                self.attribute().decorate(&mut d, &format!("{}", t))?;
                write!(f, "{}", d)
            }
            Self::S(s) => write!(f, "S({})", s),
            Self::Push(s) => write!(f, "Push({})", s),
            Self::Pop => write!(f, "Pop"),
        }
    }
}

impl TryFrom<Factor> for Symbol {
    type Error = miette::Error;
    fn try_from(factor: Factor) -> miette::Result<Self> {
        try_from_factor(factor)
    }
}
