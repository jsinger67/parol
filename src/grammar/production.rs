use crate::{Symbol, Terminal};
use std::fmt::{Debug, Display, Error, Formatter};
use std::hash::Hash;
use std::ops::Index;

pub type Rhs = Vec<Symbol>;

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
        Self(Symbol::N("".to_owned()), Rhs::default())
    }
}

impl Pr {
    pub fn new(n: &str, r: Rhs) -> Self {
        if !r.iter().all(Self::is_allowed_symbol) {
            panic!("Unexpected symbol kind!");
        }
        Self(Symbol::N(n.to_owned()), r)
    }

    pub fn get_n(&self) -> String {
        self.0.get_n().unwrap()
    }

    pub fn get_n_str(&self) -> &str {
        self.0.get_n_ref().unwrap()
    }

    pub fn get_r(&self) -> &Rhs {
        &self.1
    }

    pub fn take(self) -> (String, Rhs) {
        (self.0.get_n().unwrap(), self.1)
    }

    pub fn set_n(&mut self, n: String) {
        self.0 = Symbol::N(n);
    }

    pub fn is_empty(&self) -> bool {
        self.1.is_empty()
    }

    pub fn len(&self) -> usize {
        self.1.len()
    }

    fn is_allowed_symbol(s: &Symbol) -> bool {
        !(matches!(s, Symbol::T(Terminal::Eps)))
    }

    pub fn format<R>(&self, scanner_state_resolver: &R) -> String
    where
        R: Fn(&[usize]) -> String,
    {
        format!(
            "{}: {};",
            self.0,
            self.1
                .iter()
                .fold(Vec::new(), |mut acc, s| {
                    acc.push(s.format(scanner_state_resolver));
                    acc
                })
                .join(" ")
        )
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
