use std::convert::TryFrom;
use std::fmt::{Debug, Display, Error, Formatter};
use std::result::Result;
use miette::miette;

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

impl TryFrom<&str> for BinaryOperator {
    type Error = miette::Error;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "+" => Ok(Self::Add),
            "-" => Ok(Self::Sub),
            "*" => Ok(Self::Mul),
            "/" => Ok(Self::Div),
            "%" => Ok(Self::Mod),
            "**" => Ok(Self::Pow),
            "==" => Ok(Self::Eq),
            "!=" => Ok(Self::Ne),
            "<" => Ok(Self::Lt),
            "<=" => Ok(Self::Le),
            ">" => Ok(Self::Gt),
            ">=" => Ok(Self::Ge),
            "<<" => Ok(Self::BitShl),
            ">>" => Ok(Self::BitShr),
            "&" => Ok(Self::BitAnd),
            "|" => Ok(Self::BitOr),
            "&&" => Ok(Self::LogAnd),
            "||" => Ok(Self::LogOr),
            _ => Err(miette!("Unexpected assignment operator {}", s)),
        }
    }
}
