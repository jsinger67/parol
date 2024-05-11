use anyhow::anyhow;
use std::convert::TryFrom;
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

impl TryFrom<&str> for AssignOperator {
    type Error = anyhow::Error;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "=" => Ok(Self::Assign),
            "+=" => Ok(Self::Plus),
            "-=" => Ok(Self::Minus),
            "*=" => Ok(Self::Mul),
            "/=" => Ok(Self::Div),
            "%=" => Ok(Self::Mod),
            "<<=" => Ok(Self::ShiftLeft),
            ">>=" => Ok(Self::ShiftRight),
            "&=" => Ok(Self::BitwiseAnd),
            "^=" => Ok(Self::BitwiseXOr),
            "|=" => Ok(Self::BitwiseOr),
            _ => Err(anyhow!("Unexpected assignment operator {}", s)),
        }
    }
}
