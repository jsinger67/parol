//! Attributes can be attached to Symbols and to Productions.
//! They convey information that is temporarily available during the phase of grammar
//! transformation.
use std::fmt::{Debug, Display, Error, Formatter, Write};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Id type for tracking of optionals during grammar transformation
#[derive(Debug, Clone, Copy, Hash, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct OptionalId(usize);

/// Used to decorate an object's printed format
pub trait Decorate<T, W>
where
    T: Display,
    W: Write,
{
    /// Function used for decorated formatting
    fn decorate(&self, out: &mut W, decoratee: &T) -> std::result::Result<(), Error>;
}

///
/// Attributes applicable to a production or an alternation
///
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum ProductionAttribute {
    /// No valid attribute, default value
    None,
    /// Indicates a start of repetition, i.e. a collection
    CollectionStart,
    /// Add to a collection
    AddToCollection,
    /// Some case of an optional
    OptionalSome,
    /// None case of an optional
    OptionalNone,
}

impl Display for ProductionAttribute {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match self {
            Self::None => write!(f, "-"),
            Self::CollectionStart => write!(f, "Vec<T>::New"),
            Self::AddToCollection => write!(f, "Vec<T>::Push"),
            Self::OptionalSome => write!(f, "Option<T>::Some"),
            Self::OptionalNone => write!(f, "Option<T>::None"),
        }
    }
}

impl Default for ProductionAttribute {
    fn default() -> Self {
        Self::None
    }
}

impl<T, W> Decorate<T, W> for ProductionAttribute
where
    T: Display,
    W: Write,
{
    fn decorate(&self, out: &mut W, decoratee: &T) -> std::result::Result<(), Error> {
        match self {
            Self::None => out.write_fmt(format_args!("{decoratee}")),
            Self::CollectionStart => out.write_fmt(format_args!("{decoratee} /* Vec<T>::New */")),
            Self::AddToCollection => out.write_fmt(format_args!("{decoratee} /* Vec<T>::Push */")),
            Self::OptionalSome => out.write_fmt(format_args!("{decoratee} /* Option<T>::Some */")),
            Self::OptionalNone => out.write_fmt(format_args!("{decoratee} /* Option<T>::None */")),
        }
    }
}

///
/// Attributes applicable to a grammar symbol
///
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum SymbolAttribute {
    /// No valid attribute, default value
    None,

    /// The symbol is actually a collection, i.e. a vector
    /// Is attached to a non-terminal symbol.
    /// If an argument with this attribute appears in the argument list of a semantic action
    /// this collection should be reversed.
    RepetitionAnchor,

    /// The symbol is an option with the inner type that is determined by the non-terminal
    Option,

    /// The symbol is not relevant for the AST and should not be propagated.
    Clipped,
}

impl Display for SymbolAttribute {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match self {
            Self::None => Ok(()),
            Self::RepetitionAnchor => write!(f, "`Vec<T>`"),
            Self::Option => write!(f, "`Option<T>`"),
            Self::Clipped => write!(f, "Clipped"),
        }
    }
}

impl Default for SymbolAttribute {
    fn default() -> Self {
        Self::None
    }
}

impl<T, W> Decorate<T, W> for SymbolAttribute
where
    T: Display,
    W: std::fmt::Write,
{
    fn decorate(&self, out: &mut W, decoratee: &T) -> std::result::Result<(), Error> {
        match self {
            Self::None => out.write_fmt(format_args!("{decoratee}")),
            Self::RepetitionAnchor => out.write_fmt(format_args!("{decoratee} /* Vec */")),
            Self::Option => out.write_fmt(format_args!("{decoratee} /* Option */")),
            Self::Clipped => out.write_fmt(format_args!("{decoratee}^ /* Clipped */")),
        }
    }
}
