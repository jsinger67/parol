use super::{Decorate, SymbolAttribute};
use crate::analysis::k_tuple::TerminalMappings;
use crate::parser::parol_grammar::{Factor, LookaheadExpression, UserDefinedTypeName};
use crate::parser::to_grammar_config::try_from_factor;
use anyhow::{anyhow, Result};
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

    /// Behavioral equivalence
    /// ```
    /// use parol::TerminalKind;
    ///
    /// assert!(TerminalKind::Legacy.behaves_like(TerminalKind::Legacy));
    /// assert!(TerminalKind::Legacy.behaves_like(TerminalKind::Regex));
    /// assert!(!TerminalKind::Legacy.behaves_like(TerminalKind::Raw));
    /// assert!(TerminalKind::Regex.behaves_like(TerminalKind::Regex));
    /// assert!(TerminalKind::Regex.behaves_like(TerminalKind::Legacy));
    /// assert!(!TerminalKind::Regex.behaves_like(TerminalKind::Raw));
    /// assert!(TerminalKind::Raw.behaves_like(TerminalKind::Raw));
    /// assert!(!TerminalKind::Raw.behaves_like(TerminalKind::Regex));
    /// assert!(!TerminalKind::Raw.behaves_like(TerminalKind::Legacy));
    /// ```
    ///
    pub fn behaves_like(&self, other: TerminalKind) -> bool {
        match self {
            TerminalKind::Legacy | TerminalKind::Regex => match other {
                TerminalKind::Legacy | TerminalKind::Regex => true,
                TerminalKind::Raw => false,
            },
            TerminalKind::Raw => match other {
                TerminalKind::Legacy | TerminalKind::Regex => false,
                TerminalKind::Raw => true,
            },
        }
    }

    /// Equivalence regarding expansion result
    /// ```
    /// use parol::TerminalKind;
    ///
    /// assert!(TerminalKind::expands_like(
    ///     "\n", TerminalKind::Legacy,
    ///     "\n", TerminalKind::Regex));
    /// assert!(TerminalKind::expands_like(
    ///     "{", TerminalKind::Raw,
    ///     r"\{", TerminalKind::Regex));
    /// assert!(TerminalKind::expands_like(
    ///     "{", TerminalKind::Raw,
    ///     r"\{", TerminalKind::Legacy));
    /// assert!(!TerminalKind::expands_like(
    ///     r"\{", TerminalKind::Raw,
    ///     r"\{", TerminalKind::Legacy));
    /// ```
    ///
    pub fn expands_like(
        this_term: &str,
        this_kind: TerminalKind,
        other_term: &str,
        other_kind: TerminalKind,
    ) -> bool {
        this_kind.expand(this_term) == other_kind.expand(other_term)
    }

    /// The actual preparation for scanner regex generation
    /// * Raw strings and legacy strings are not specially treaded
    /// * Regex strings are escaped using regex::escape
    pub fn expand(&self, term: &str) -> String {
        match self {
            crate::TerminalKind::Legacy | crate::TerminalKind::Regex => term.to_string(),
            crate::TerminalKind::Raw => regex::escape(term),
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
        Option<LookaheadExpression>,
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
        Self::Trm(t.to_owned(), TerminalKind::Legacy, s, a, None, None)
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
            Symbol::T(Terminal::Trm(t, k, s, a, u, l)) => {
                Terminal::Trm(t.to_string(), *k, s.to_vec(), *a, u.clone(), l.clone())
            }
            Symbol::T(Terminal::End) => Terminal::End,
            _ => panic!("Unexpected symbol type: {:?}", s),
        }
    }

    /// Adds a scanner index
    pub fn add_scanner(&mut self, sc: usize) {
        match self {
            Terminal::Trm(_, _, s, _, _, _) => {
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
            Self::Trm(t, k, s, a, u, l) => {
                let mut d = String::new();
                let delimiter = k.delimiter();
                a.decorate(&mut d, &format!("{}{}{}", delimiter, t, delimiter))
                    .map_err(|e| anyhow!("Decorate error!: {}", e))?;
                if let Some(la) = l {
                    write!(d, " {}", la.to_par()).map_err(|e| anyhow!(e))?;
                }
                if let Some(ref user_type) = u {
                    let user_type =
                        if let Some(alias) = user_type_resolver(user_type.to_string().as_str()) {
                            alias
                        } else {
                            user_type.to_string()
                        };
                    write!(d, " : {}", user_type).map_err(|e| anyhow!(e))?;
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
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
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
    #[inline]
    fn eps() -> Terminal {
        Self::Eps
    }

    #[inline]
    fn end() -> Terminal {
        Self::End
    }

    #[inline]
    fn is_eps(&self) -> bool {
        self.is_eps()
    }

    #[inline]
    fn is_end(&self) -> bool {
        self.is_end()
    }

    #[inline]
    fn is_inv(&self) -> bool {
        false
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
            Symbol::N(_, a, _) | Symbol::T(Terminal::Trm(_, _, _, a, _, _)) => *a,
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
                a.decorate(&mut s, n)
                    .map_err(|e| anyhow!("Decorate error!: {}", e))?;
                if let Some(ref user_type) = u {
                    let user_type =
                        if let Some(alias) = user_type_resolver(user_type.to_string().as_str()) {
                            alias
                        } else {
                            user_type.to_string()
                        };
                    write!(s, " : {}", user_type).map_err(|e| anyhow!("IO error!: {}", e))?;
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
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
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
    type Error = anyhow::Error;
    fn try_from(factor: Factor) -> Result<Self> {
        try_from_factor(factor)
    }
}
