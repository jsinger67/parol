use crate::grammar::SymbolAttribute;
use crate::{Symbol, Terminal};
use miette::Result;
use std::fmt::{Debug, Display, Error, Formatter};
use std::hash::Hash;
use std::ops::Index;

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
///
/// Right-hand side of a production.
/// A collection of [Symbol]s
///
pub type Rhs = Vec<Symbol>;

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
///
/// Production type
///
#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Pr(pub Symbol, pub Rhs);

impl Display for Pr {
    ///
    /// The output format for a production roughly follows the Yacc format.
    ///
    /// ```
    /// use parol::{Pr, Symbol};
    ///
    /// let pr = Pr::new("S", vec![]);
    /// assert_eq!("S: ;", format!("{}", pr));
    /// let pr = Pr::new("S", vec![Symbol::n("N"), Symbol::n("L")]);
    /// assert_eq!("S: N L;", format!("{}", pr));
    /// let pr = Pr::new("S", vec![Symbol::n("I"), Symbol::n("L")]);
    /// assert_eq!("S: I L;", format!("{}", pr));
    /// let pr = Pr::new("S", vec![Symbol::t(",", vec![0]), Symbol::n("N")]);
    /// assert_eq!(r#"S: "," N;"#, format!("{}", pr));
    /// let pr = Pr::new("S", vec![Symbol::t("d", vec![0])]);
    /// assert_eq!(r#"S: "d";"#, format!("{}", pr));
    /// let pr = Pr::new("S", vec![Symbol::t(r#"[0-9]"#, vec![0]), Symbol::t("e", vec![0])]);
    /// assert_eq!(r#"S: "[0-9]" "e";"#, format!("{}", pr));
    /// ```
    ///
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "{}: {};",
            self.0,
            self.1
                .iter()
                .fold(Vec::new(), |mut acc, s| {
                    acc.push(format!("{}", s));
                    acc
                })
                .join(" ")
        )
    }
}

impl Default for Pr {
    fn default() -> Self {
        Self(Symbol::n(""), Rhs::default())
    }
}

impl Pr {
    /// Creates a new item from a non-terminal name and a [Rhs]
    pub fn new(n: &str, r: Rhs) -> Self {
        if !r.iter().all(Self::is_allowed_symbol) {
            panic!("Unexpected symbol kind!");
        }
        Self(Symbol::n(n), r)
    }

    /// Returns a clone of the non-terminal
    pub fn get_n(&self) -> String {
        self.0.get_n().unwrap()
    }

    /// Returns a reference of the non-terminal
    pub fn get_n_str(&self) -> &str {
        self.0.get_n_ref().unwrap()
    }

    /// Returns a reference of the ride-hand side
    pub fn get_r(&self) -> &Rhs {
        &self.1
    }

    /// Extracts the members of self while consuming self
    pub fn take(self) -> (String, Rhs) {
        (self.0.get_n().unwrap(), self.1)
    }

    /// Sets the non-terminal
    pub fn set_n(&mut self, n: String) {
        self.0 = Symbol::N(n, SymbolAttribute::default());
    }

    /// Checks if [Rhs] is empty
    pub fn is_empty(&self) -> bool {
        self.1.is_empty()
    }

    /// Returns the length of [Rhs]
    pub fn len(&self) -> usize {
        self.1.len()
    }

    fn is_allowed_symbol(s: &Symbol) -> bool {
        !(matches!(s, Symbol::T(Terminal::Eps)))
    }

    /// Formats self with the help of a scanner state resolver
    pub fn format<R>(&self, scanner_state_resolver: &R) -> Result<String>
    where
        R: Fn(&[usize]) -> String,
    {
        Ok(format!(
            "{}: {};",
            self.0,
            self.1
                .iter()
                .fold(Ok(Vec::new()), |acc: Result<Vec<String>>, s| {
                    if let Ok(mut acc) = acc {
                        acc.push(s.format(scanner_state_resolver)?);
                        Ok(acc)
                    } else {
                        acc
                    }
                })
                .map(|v| v.join(" "))?
        ))
    }
}

impl Index<usize> for Pr {
    type Output = Symbol;

    fn index(&self, idx: usize) -> &Self::Output {
        if idx == 0 {
            &self.0
        } else {
            &self.1[idx]
        }
    }
}
