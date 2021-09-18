use crate::grammar::SymbolString;
use std::collections::BTreeSet;
use std::fmt::{Display, Error, Formatter};
use std::iter::FromIterator;

///
/// Ordered set type of symbol strings
///
#[derive(Debug, Clone, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct SymbolStrings(pub BTreeSet<SymbolString>);

impl SymbolStrings {
    pub fn new() -> Self {
        Self(BTreeSet::new())
    }

    pub fn insert(&mut self, sy_str: SymbolString) -> bool {
        self.0.insert(sy_str)
    }

    pub fn append(&mut self, other: &mut Self) {
        self.0.append(&mut other.0)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Display for SymbolStrings {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|e| format!("{}", e))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

impl FromIterator<SymbolString> for SymbolStrings {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = SymbolString>,
    {
        let mut c = Self::new();
        for i in iter {
            c.insert(i);
        }
        c
    }
}
