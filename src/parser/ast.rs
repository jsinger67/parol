use super::errors::*;
use crate::lexer::OwnedToken;
use id_tree_layout::Visualize;
use std::fmt::{Display, Formatter};

///
/// The type of the elements in the syntax tree.
///
#[derive(Debug, Clone)]
pub enum AstType {
    ///
    /// An owned representation of a scanned terminal symbol.
    ///
    T(OwnedToken),

    ///
    /// A reference into the slice of non-terminal names.
    /// These names are of static lifetime.
    ///
    N(&'static str),
}

impl AstType {
    ///
    /// Tries to access the OwnedToken of the AstType.
    /// Can fail if the entry is no terminal (i.e. a non-terminal).
    ///
    pub fn token(&self) -> Result<&OwnedToken> {
        match self {
            Self::T(t) => Ok(t),
            _ => Err(format!("{} is no token!", self).into()),
        }
    }
}

///
/// Implementation of the Visualize trait to support the visualization of the
/// AstType in a tree layout.
///
impl Visualize for AstType {
    fn visualize(&self) -> std::string::String {
        match self {
            Self::T(t) => format!("{}", t),
            Self::N(n) => n.to_string(),
        }
    }
    fn emphasize(&self) -> bool {
        matches!(self, Self::T(_))
    }
}

impl Display for AstType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Self::T(t) => write!(f, "T({})", t),
            Self::N(n) => write!(f, "N({})", n),
        }
    }
}
