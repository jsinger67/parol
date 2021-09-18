use std::fmt::{Debug, Display, Error, Formatter};
use std::result::Result;

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    BitShl,
    BitShr,
    BitAnd,
    BitOr,
    LogAnd,
    LogOr,
}

impl Display for BinaryOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Self::Add => write!(f, "+"),
            Self::Sub => write!(f, "-"),
            Self::Mul => write!(f, "*"),
            Self::Div => write!(f, "/"),
            Self::Mod => write!(f, "%"),
            Self::Pow => write!(f, "**"),
            Self::Eq => write!(f, "=="),
            Self::Ne => write!(f, "!="),
            Self::Lt => write!(f, "<"),
            Self::Le => write!(f, "<="),
            Self::Gt => write!(f, ">"),
            Self::Ge => write!(f, ">="),
            Self::BitShl => write!(f, "<<"),
            Self::BitShr => write!(f, ">>"),
            Self::BitAnd => write!(f, "&"),
            Self::BitOr => write!(f, "|"),
            Self::LogAnd => write!(f, "&&"),
            Self::LogOr => write!(f, "||"),
        }
    }
}

impl From<&str> for BinaryOperator {
    fn from(s: &str) -> Self {
        match s {
            "+" => Self::Add,
            "-" => Self::Sub,
            "*" => Self::Mul,
            "/" => Self::Div,
            "%" => Self::Mod,
            "**" => Self::Pow,
            "==" => Self::Eq,
            "!=" => Self::Ne,
            "<" => Self::Lt,
            "<=" => Self::Le,
            ">" => Self::Gt,
            ">=" => Self::Ge,
            "<<" => Self::BitShl,
            ">>" => Self::BitShr,
            "&" => Self::BitAnd,
            "|" => Self::BitOr,
            "&&" => Self::LogAnd,
            "||" => Self::LogOr,
            _ => panic!("Unexpected assignment operator {}", s),
        }
    }
}
