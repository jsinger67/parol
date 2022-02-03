use std::fmt::{Debug, Display, Error, Formatter};
use std::result::Result;

#[derive(Debug, Clone)]
pub enum AssignOperator {
    Assign,
    Plus,
    Minus,
    Mul,
    Div,
    Mod,
    ShiftLeft,
    ShiftRight,
    BitwiseAnd,
    BitwiseXOr,
    BitwiseOr,
}

impl Display for AssignOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Self::Assign => write!(f, "="),
            Self::Plus => write!(f, "+="),
            Self::Minus => write!(f, "-="),
            Self::Mul => write!(f, "*="),
            Self::Div => write!(f, "/="),
            Self::Mod => write!(f, "%="),
            Self::ShiftLeft => write!(f, "<<="),
            Self::ShiftRight => write!(f, ">>="),
            Self::BitwiseAnd => write!(f, "&="),
            Self::BitwiseXOr => write!(f, "^="),
            Self::BitwiseOr => write!(f, "|="),
        }
    }
}

impl From<&str> for AssignOperator {
    fn from(s: &str) -> Self {
        match s {
            "=" => Self::Assign,
            "+=" => Self::Plus,
            "-=" => Self::Minus,
            "*=" => Self::Mul,
            "/=" => Self::Div,
            "%=" => Self::Mod,
            "<<=" => Self::ShiftLeft,
            ">>=" => Self::ShiftRight,
            "&=" => Self::BitwiseAnd,
            "^=" => Self::BitwiseXOr,
            "|=" => Self::BitwiseOr,
            _ => panic!("Unexpected assignment operator {}", s),
        }
    }
}
