use std::fmt::{Debug, Display, Error, Formatter};
use std::result::Result;

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Negation,
}

impl Display for UnaryOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Self::Negation => write!(f, "-"),
        }
    }
}

impl From<&str> for UnaryOperator {
    fn from(s: &str) -> Self {
        match s {
            "-" => Self::Negation,
            _ => panic!("Unexpected assignment operator {}", s),
        }
    }
}
