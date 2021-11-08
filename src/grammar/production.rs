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
    /// let pr = Pr::new("S", vec![Symbol::t(","), Symbol::n("N")]);
    /// assert_eq!(r#"S: "," N;"#, format!("{}", pr));
    /// let pr = Pr::new("S", vec![Symbol::t("d", 0)]);
    /// assert_eq!(r#"S: "d";"#, format!("{}", pr));
    /// let pr = Pr::new("S", vec![Symbol::t(r#"\d"#), Symbol::t("e", 0)]);
    /// assert_eq!(r#"S: "\d" "e";"#, format!("{}", pr));
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
        if !r.iter().all(|s| Self::is_allowed_symbol(s)) {
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
    pub fn get_r_mut(&mut self) -> &mut Rhs {
        &mut self.1
    }
    pub fn take(self) -> (String, Rhs) {
        (self.0.get_n().unwrap(), self.1)
    }
    pub fn set_n(&mut self, n: String) {
        self.0 = Symbol::N(n);
    }
    pub fn set_r(&mut self, r: Rhs) {
        if !r.iter().all(|s| Self::is_allowed_symbol(s)) {
            panic!("Unexpected symbol kind!");
        }
        self.1 = r;
    }
    pub fn is_empty(&self) -> bool {
        self.1.is_empty()
    }
    pub fn len(&self) -> usize {
        self.1.len()
    }
    ///
    /// Calculates the length of the terminal symbols at the beginning of the RHS.
    ///
    /// ```
    /// use parol::{Pr, Symbol};
    ///
    /// let pr = Pr::new("S", vec![]);
    /// assert_eq!(0, pr.first_len());
    /// let pr = Pr::new("S", vec![Symbol::n("N"), Symbol::n("L")]);
    /// assert_eq!(0, pr.first_len());
    /// let pr = Pr::new("S", vec![Symbol::n("I"), Symbol::n("L")]);
    /// assert_eq!(0, pr.first_len());
    /// let pr = Pr::new("S", vec![Symbol::t(","), Symbol::n("N")]);
    /// assert_eq!(1, pr.first_len());
    /// let pr = Pr::new("S", vec![Symbol::t("d", 0)]);
    /// assert_eq!(1, pr.first_len());
    /// let pr = Pr::new("S", vec![Symbol::t("d", 0), Symbol::t("e", 0)]);
    /// assert_eq!(2, pr.first_len());
    /// ```
    ///
    pub fn first_len(&self) -> usize {
        self.1
            .iter()
            .take_while(|s| {
                matches!(s, Symbol::T(Terminal::Trm(_, _))) || matches!(s, Symbol::T(Terminal::End))
            })
            .count()
    }

    ///
    /// Calculates the length of the terminal symbols starting from an offset within the RHS.
    ///
    /// ```
    /// use parol::{Pr, Symbol};
    ///
    /// let pr = Pr::new("S", vec![]);
    /// assert_eq!(0, pr.first_len_at(0));
    /// assert_eq!(0, pr.first_len_at(1));
    /// let pr = Pr::new("S", vec![Symbol::n("N"), Symbol::n("L")]);
    /// assert_eq!(0, pr.first_len_at(0));
    /// assert_eq!(0, pr.first_len_at(1));
    /// assert_eq!(0, pr.first_len_at(2));
    /// assert_eq!(0, pr.first_len_at(3));
    /// let pr = Pr::new("S", vec![Symbol::t(","), Symbol::n("N")]);
    /// assert_eq!(1, pr.first_len_at(0));
    /// assert_eq!(0, pr.first_len_at(1));
    /// assert_eq!(0, pr.first_len_at(2));
    /// let pr = Pr::new("S", vec![Symbol::n("N"), Symbol::t("d", 0), Symbol::t("e", 0)]);
    /// assert_eq!(0, pr.first_len_at(0));
    /// assert_eq!(2, pr.first_len_at(1));
    /// assert_eq!(1, pr.first_len_at(2));
    /// assert_eq!(0, pr.first_len_at(3));
    /// let pr = Pr::new("S", vec![Symbol::t("c", 0), Symbol::t("d", 0), Symbol::t("e", 0)]);
    /// assert_eq!(3, pr.first_len_at(0));
    /// assert_eq!(2, pr.first_len_at(1));
    /// assert_eq!(1, pr.first_len_at(2));
    /// assert_eq!(0, pr.first_len_at(3));
    /// let pr = Pr::new("S", vec![Symbol::t("c", 0), Symbol::t("d", 0), Symbol::t("e", 0), Symbol::n("N")]);
    /// assert_eq!(3, pr.first_len_at(0));
    /// assert_eq!(2, pr.first_len_at(1));
    /// assert_eq!(1, pr.first_len_at(2));
    /// assert_eq!(0, pr.first_len_at(3));
    /// assert_eq!(0, pr.first_len_at(4));
    /// ```
    ///
    pub fn first_len_at(&self, sy_idx: usize) -> usize {
        self.1
            .iter()
            .skip(sy_idx)
            .take_while(|s| {
                matches!(s, Symbol::T(Terminal::Trm(_, _))) || matches!(s, Symbol::T(Terminal::End))
            })
            .count()
    }

    ///
    /// Calculates whether a next derivation is necessary and possible to get
    /// a first length of k.
    /// If the first_len is already equal or greater than k there is no derivation
    /// necessary.
    /// Else if no non-terminal is contained then a derivation is not possible
    /// Else it returns true.
    ///
    /// ```
    /// use parol::{Pr, Symbol};
    ///
    /// let pr = Pr::new("S", vec![]);
    /// assert!(!pr.is_k_derivable(1), "Empty production - not possible");
    /// let pr = Pr::new("S", vec![Symbol::n("N"), Symbol::n("L")]);
    /// assert!(pr.is_k_derivable(1), "k_len == 0 but containing Nt - possible");
    /// let pr = Pr::new("S", vec![Symbol::t(","), Symbol::n("N")]);
    /// assert!(!pr.is_k_derivable(1), "k_len == 1 - not necessary");
    /// assert!(pr.is_k_derivable(2), "k_len == 1 but containing Nt - not necessary");
    /// let pr = Pr::new("S", vec![Symbol::t("d", 0)]);
    /// assert!(!pr.is_k_derivable(1), "k_len == 1 - not necessary");
    /// assert!(!pr.is_k_derivable(2), "k_len == 1, containing no Nt - not possible");
    /// let pr = Pr::new("S", vec![Symbol::t("d", 0), Symbol::t("e", 0)]);
    /// assert!(!pr.is_k_derivable(1), "k_len == 2 - not necessary");
    /// assert!(!pr.is_k_derivable(2), "k_len == 2 - not necessary");
    /// assert!(!pr.is_k_derivable(3), "k_len == 2, containing no Nt - not possible");
    /// ```
    ///
    pub fn is_k_derivable(&self, k: usize) -> bool {
        // Necessary
        self.first_len() < k &&
        // Possible
        self.1.iter().any(|s| matches!(s, Symbol::N(_)))
    }

    ///
    /// Calculates whether a next derivation is necessary and possible to get
    /// a first length of k at a given symbol offset.
    /// If the first_len_at is already equal or greater than k there is no derivation
    /// necessary.
    /// Else if no non-terminal is contained then a derivation is not possible
    /// Else it returns true.
    ///
    /// ```
    /// use parol::{Pr, Symbol};
    ///
    /// let pr = Pr::new("S", vec![]);
    /// assert!(!pr.is_k_derivable_at(1, 0), "Empty production - not possible");
    /// assert!(!pr.is_k_derivable_at(1, 1), "Empty production, invalid index - not possible");
    /// let pr = Pr::new("S", vec![Symbol::n("N"), Symbol::n("L")]);
    /// assert!(pr.is_k_derivable_at(1, 0), "k_len == 0 but containing Nt - possible");
    /// assert!(pr.is_k_derivable_at(1, 1), "k_len == 0 but containing Nt - possible");
    /// assert!(!pr.is_k_derivable_at(1, 2), "invalid index - not possible");
    /// let pr = Pr::new("S", vec![Symbol::t(","), Symbol::n("N")]);
    /// assert!(!pr.is_k_derivable_at(1, 0), "k_len == 1 - not necessary");
    /// assert!(pr.is_k_derivable_at(2, 0), "k_len == 1 but containing Nt - possible");
    /// assert!(pr.is_k_derivable_at(2, 1), "k_len == 0 but containing Nt - possible");
    /// assert!(!pr.is_k_derivable_at(2, 2), "invalid index - not possible");
    /// let pr = Pr::new("S", vec![Symbol::t("d", 0)]);
    /// assert!(!pr.is_k_derivable_at(1, 0), "k_len == 1 - not necessary");
    /// assert!(!pr.is_k_derivable_at(2, 0), "k_len == 1, containing no Nt - not possible");
    /// assert!(!pr.is_k_derivable_at(1, 1), "invalid index - not possible");
    /// assert!(!pr.is_k_derivable_at(2, 1), "invalid index - not possible");
    /// let pr = Pr::new("S", vec![Symbol::t("d", 0), Symbol::t("e", 0)]);
    /// assert!(!pr.is_k_derivable_at(1, 0), "k_len == 2 - not necessary");
    /// assert!(!pr.is_k_derivable_at(2, 0), "k_len == 2 - not necessary");
    /// assert!(!pr.is_k_derivable_at(3, 0), "k_len == 2, containing no Nt - not possible");
    /// assert!(!pr.is_k_derivable_at(1, 1), "k_len == 1 - not necessary");
    /// assert!(!pr.is_k_derivable_at(2, 1), "k_len == 1 - not possible");
    /// assert!(!pr.is_k_derivable_at(3, 1), "invalid index - not possible");
    /// ```
    ///
    pub fn is_k_derivable_at(&self, k: usize, sy_idx: usize) -> bool {
        // Necessary
        self.first_len_at(sy_idx) < k &&
        // Possible
        self.1.iter().skip(sy_idx).any(|s| matches!(s, Symbol::N(_)))
    }

    fn is_allowed_symbol(s: &Symbol) -> bool {
        !(matches!(s, Symbol::T(Terminal::Eps)))
    }
    pub fn is_valid_index(&self, idx: usize) -> bool {
        idx < self.1.len()
    }
    pub fn try_get_rhs_symbol(&self, sy_index: usize) -> Option<&Symbol> {
        if self.is_valid_index(sy_index) {
            Some(&self[sy_index])
        } else {
            None
        }
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
