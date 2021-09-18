use crate::analysis::k_tuple::TerminalMappings;
use crate::parser::parol_grammar::Factor;
use crate::parser::to_grammar_config::try_from_factor;
use std::convert::TryFrom;
use std::fmt::{Debug, Display, Error, Formatter};

#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum Terminal {
    ///
    /// A physical terminal symbol
    /// Entities that are provided by the lexer.
    ///
    Trm(String),

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
    pub fn t(t: &str) -> Self {
        Self::Trm(t.to_owned())
    }
    pub fn is_trm(&self) -> bool {
        matches!(self, Self::Trm(_))
    }
    pub fn is_eps(&self) -> bool {
        matches!(self, Self::Eps)
    }
    pub fn is_end(&self) -> bool {
        matches!(self, Self::End)
    }

    pub fn create(s: &Symbol) -> Self {
        match s {
            Symbol::T(Terminal::Trm(t)) => Terminal::Trm(t.to_string()),
            Symbol::T(Terminal::End) => Terminal::End,
            _ => panic!("Unexpected symbol type: {:?}", s),
        }
    }
}

impl Display for Terminal {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Self::Trm(t) => write!(f, "\"{}\"", t),
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
    T(Terminal),
}

impl Symbol {
    pub fn t(t: &str) -> Self {
        Self::T(Terminal::Trm(t.to_owned()))
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
