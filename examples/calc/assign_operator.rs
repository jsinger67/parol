use std::fmt::{Debug, Display, Error, Formatter};
use std::result::Result;

#[derive(Debug, Clone)]
pub enum AssignOperator {
    Assign,
    PlusAssign,
    MinusAssign,
    MulAssign,
    DivAssign,
    ModAssign,
    ShiftLeftAssign,
    ShiftRightAssign,
    BitwiseAndAssign,
    BitwiseXOrAssign,
    BitwiseOrAssign,
}

impl Display for AssignOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Self::Assign => write!(f, "="),
            Self::PlusAssign => write!(f, "+="),
            Self::MinusAssign => write!(f, "-="),
            Self::MulAssign => write!(f, "*="),
            Self::DivAssign => write!(f, "/="),
            Self::ModAssign => write!(f, "%="),
            Self::ShiftLeftAssign => write!(f, "<<="),
            Self::ShiftRightAssign => write!(f, ">>="),
            Self::BitwiseAndAssign => write!(f, "&="),
            Self::BitwiseXOrAssign => write!(f, "^="),
            Self::BitwiseOrAssign => write!(f, "|="),
        }
    }
}

impl From<&str> for AssignOperator {
    fn from(s: &str) -> Self {
        match s {
            "=" => Self::Assign,
            "+=" => Self::PlusAssign,
            "-=" => Self::MinusAssign,
            "*=" => Self::MulAssign,
            "/=" => Self::DivAssign,
            "%=" => Self::ModAssign,
            "<<=" => Self::ShiftLeftAssign,
            ">>=" => Self::ShiftRightAssign,
            "&=" => Self::BitwiseAndAssign,
            "^=" => Self::BitwiseXOrAssign,
            "|=" => Self::BitwiseOrAssign,
            _ => panic!("Unexpected assignment operator {}", s),
        }
    }
}
