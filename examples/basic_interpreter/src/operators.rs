use std::convert::TryFrom;
use std::fmt::{Debug, Display, Error, Formatter};
use std::result::Result;

use log::trace;

use crate::basic_grammar::DefinitionRange;

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    LogAnd,
    LogOr,
    LogNor,
}

impl BinaryOperator {
    pub(crate) fn apply_binary_operation(
        lhs: DefinitionRange,
        op: &BinaryOperator,
        rhs: DefinitionRange,
        context: &str,
    ) -> miette::Result<DefinitionRange> {
        trace!(
            "apply_binary_operation: {}: {} {} {}",
            context,
            lhs,
            op,
            rhs
        );
        let result = match op {
            Self::Add => lhs + rhs,
            Self::Sub => lhs - rhs,
            Self::Mul => lhs * rhs,
            Self::Div => {
                if rhs == 0.0 {
                    bail!("Division by zero detected!");
                }
                lhs / rhs
            }
            Self::Eq => {
                if lhs == rhs {
                    1.0
                } else {
                    0.0
                }
            }
            Self::Ne => {
                if lhs != rhs {
                    1.0
                } else {
                    0.0
                }
            }
            Self::Lt => {
                if lhs < rhs {
                    1.0
                } else {
                    0.0
                }
            }
            Self::Le => {
                if lhs <= rhs {
                    1.0
                } else {
                    0.0
                }
            }
            Self::Gt => {
                if lhs > rhs {
                    1.0
                } else {
                    0.0
                }
            }
            Self::Ge => {
                if lhs >= rhs {
                    1.0
                } else {
                    0.0
                }
            }
            Self::LogAnd => {
                if (lhs != 0.0) && (rhs != 0.0) {
                    1.0
                } else {
                    0.0
                }
            }
            Self::LogOr => {
                if (lhs != 0.0) || (rhs != 0.0) {
                    1.0
                } else {
                    0.0
                }
            }
            Self::LogNor => {
                if (lhs == 0.0) && (rhs == 0.0) {
                    1.0
                } else {
                    0.0
                }
            }
        };

        trace!("apply_binary_operation:      = {}", result);

        Ok(result)
    }
}

impl Display for BinaryOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Self::Add => write!(f, "+"),
            Self::Sub => write!(f, "-"),
            Self::Mul => write!(f, "*"),
            Self::Div => write!(f, "/"),
            Self::Eq => write!(f, "="),
            Self::Ne => write!(f, "<>"),
            Self::Lt => write!(f, "<"),
            Self::Le => write!(f, "<="),
            Self::Gt => write!(f, ">"),
            Self::Ge => write!(f, ">="),
            Self::LogAnd => write!(f, "AND"),
            Self::LogOr => write!(f, "OR"),
            Self::LogNor => write!(f, "NOR"),
        }
    }
}

impl TryFrom<&str> for BinaryOperator {
    type Error = miette::Error;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let s = s.replace(' ', "");
        match s.as_str() {
            "+" => Ok(Self::Add),
            "-" => Ok(Self::Sub),
            "*" => Ok(Self::Mul),
            "/" => Ok(Self::Div),
            "=" => Ok(Self::Eq),
            "<>" => Ok(Self::Ne),
            "<" => Ok(Self::Lt),
            "<=" => Ok(Self::Le),
            ">" => Ok(Self::Gt),
            ">=" => Ok(Self::Ge),
            "AND" => Ok(Self::LogAnd),
            "OR" => Ok(Self::LogOr),
            "NOR" => Ok(Self::LogNor),
            _ => Err(miette!("Unexpected binary operator {}", s)),
        }
    }
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    LogNot,
}

impl UnaryOperator {
    pub(crate) fn apply_unary_operation(
        op: &Self,
        val: DefinitionRange,
        context: &str,
    ) -> miette::Result<DefinitionRange> {
        trace!("apply_unary_operation: {}: {} {}", context, op, val);
        let result = match op {
            Self::LogNot => {
                if val == 0.0 {
                    1.0
                } else {
                    0.0
                }
            }
        };

        trace!("apply_unary_operation:      = {}", result);

        Ok(result)
    }
}

impl Display for UnaryOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Self::LogNot => write!(f, "NOT"),
        }
    }
}

impl TryFrom<&str> for UnaryOperator {
    type Error = miette::Error;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let s = s.replace(' ', "");
        match s.as_str() {
            "NOT" => Ok(Self::LogNot),
            _ => Err(miette!("Unexpected unary operator {}", s)),
        }
    }
}
