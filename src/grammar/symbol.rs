use crate::analysis::k_tuple::TerminalMappings;
use crate::parser::parol_grammar::Factor;
use crate::parser::to_grammar_config::try_from_factor;
use std::convert::TryFrom;
use std::fmt::{Debug, Display, Error, Formatter};

#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum Terminal {
    ///
    /// A physical terminal symbol with the scanner state it belongs to
    /// Entities that are provided by the lexer.
    ///
    Trm(String, usize),

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
    pub fn t(t: &str, s: usize) -> Self {
        Self::Trm(t.to_owned(), s)
    }
    pub fn is_trm(&self) -> bool {
        matches!(self, Self::Trm(_, _))
    }
    pub fn is_eps(&self) -> bool {
        matches!(self, Self::Eps)
    }
    pub fn is_end(&self) -> bool {
        matches!(self, Self::End)
    }

    pub fn create(s: &Symbol) -> Self {
        match s {
            Symbol::T(Terminal::Trm(t, s)) => Terminal::Trm(t.to_string(), *s),
            Symbol::T(Terminal::End) => Terminal::End,
            _ => panic!("Unexpected symbol type: {:?}", s),
        }
    }

    ///
    /// Get the scanner state in front of the terminal
    ///
    pub fn format<R>(&self, scanner_state_resolver: R) -> String
    where
        R: Fn(usize) -> String,
    {
        match self {
            Self::Trm(t, s) => {
                if *s == 0 {
                    format!("\"{}\"", t)
                } else {
                    format!("<{}>\"{}\"", scanner_state_resolver(*s), t)
                }
            }
            Self::Eps => format!("\u{03B5}"), // Lower creek letter Epsilon (ε)
            Self::End => format!("$"),
        }
    }
}

impl Display for Terminal {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Self::Trm(t, _) => write!(f, "\"{}\"", t),
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

#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum Symbol {
    ///
    /// Non-terminal symbol, Meta symbol of the grammar.
    ///
    N(String),

    ///
    /// Terminal symbol of the grammar.
    ///
    T(Terminal),
}

impl Symbol {
    pub fn t(t: &str, s: usize) -> Self {
        Self::T(Terminal::Trm(t.to_owned(), s))
    }
    pub fn n(n: &str) -> Self {
        Self::N(n.to_owned())
    }
    pub fn e() -> Self {
        Self::T(Terminal::End)
    }
    pub fn is_t(&self) -> bool {
        matches!(self, Self::T(_))
    }
    pub fn is_n(&self) -> bool {
        matches!(self, Self::N(_))
    }
    pub fn is_end(&self) -> bool {
        matches!(self, Self::T(Terminal::End))
    }
    pub fn get_t(&self) -> Option<Terminal> {
        if let Self::T(t) = &self {
            Some(t.clone())
        } else {
            None
        }
    }
    pub fn get_t_ref(&self) -> Option<&Terminal> {
        if let Self::T(t) = &self {
            Some(t)
        } else {
            None
        }
    }
    pub fn get_n(&self) -> Option<String> {
        if let Self::N(n) = &self {
            Some(n.clone())
        } else {
            None
        }
    }
    pub fn get_n_ref(&self) -> Option<&String> {
        if let Self::N(n) = &self {
            Some(n)
        } else {
            None
        }
    }

    pub fn format<R>(&self, scanner_state_resolver: &R) -> String
    where
        R: Fn(usize) -> String,
    {
        match self {
            Self::N(n) => format!("{}", n),
            Self::T(t) => t.format(scanner_state_resolver),
        }
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Self::N(n) => write!(f, "{}", n),
            Self::T(t) => write!(f, "{}", t),
        }
    }
}

impl TryFrom<Factor> for Symbol {
    type Error = crate::errors::Error;
    fn try_from(factor: Factor) -> crate::errors::Result<Self> {
        try_from_factor(factor)
    }
}
