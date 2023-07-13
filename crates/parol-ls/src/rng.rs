use crate::parol_ls_grammar::OwnedToken;
use derive_new::new;
use lsp_types::{Position, Range};

use crate::utils::location_to_range;

/// The Rng type is a customized Range that can handle empty ranges.
/// Empty ranges are null elements for extend operations:
///
/// R1 + Empty = R1
/// Empty + R1 = R1
/// Empty + Empty = Empty
///
/// Rng and lsp_types::Range are convertible into each other.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, new)]
pub(crate) struct Rng(pub(crate) Range);

#[allow(unused)]
impl Rng {
    /// Specially handled empty range.
    /// Can be found on optional elements or empty lists.
    pub(crate) fn is_empty(&self) -> bool {
        self.0.start.line == 0
            && self.0.start.character == 0
            && self.0.end.line == 0
            && self.0.end.character == 0
    }

    /// Takes ownership of self and the right range.
    /// It handles the special empty range.
    pub(crate) fn extend(mut self, right: Rng) -> Rng {
        if self.is_empty() {
            right
        } else if right.is_empty() {
            self
        } else {
            debug_assert!(self.0.start <= right.0.end);
            self.0.end = right.0.end;
            self
        }
    }

    pub(crate) fn extend_to_end(mut self) -> Rng {
        self.0.end = Position {
            line: u32::MAX,
            character: u32::MAX,
        };
        self
    }

    /// Tests if self comes completely before other
    pub(crate) fn comes_before(&self, other: &Self) -> bool {
        self.0.end.line < other.0.start.line
            || (self.0.end.line == other.0.start.line
                && self.0.end.character <= other.0.start.character)
    }

    pub(crate) fn from_slice<'a, T>(slc: &'a [T]) -> Self
    where
        &'a T: Into<Rng>,
    {
        if slc.is_empty() {
            Rng::default()
        } else {
            let first: &T = slc.first().unwrap();
            let rng: Rng = first.into();
            let last: &T = slc.last().unwrap();
            rng.extend(last.into())
        }
    }
}

impl From<&OwnedToken> for Rng {
    fn from(t: &OwnedToken) -> Self {
        Self(location_to_range(&t.location))
    }
}

impl From<Rng> for Range {
    fn from(val: Rng) -> Self {
        val.0
    }
}

impl From<&Rng> for Range {
    fn from(val: &Rng) -> Self {
        val.0
    }
}

// impl<'a, 'b, T: 'a> From<&'a [T]> for Rng where &'b T: Into<Rng>, 'a: 'b {
//     fn from(slc: &'a [T]) -> Self {
//         if slc.is_empty() {
//             Rng::default()
//         } else {
//             let first: &'b T = slc.first().unwrap();
//             let last: &'b T = slc.last().unwrap();
//             first.into().extend(last.into())
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use lsp_types::{Position, Range};

    #[test]
    fn test_extend() {
        let rng1 = Rng::new(Range {
            start: Position::new(0, 0),
            end: Position::new(0, 5),
        });
        let rng2 = Rng::new(Range {
            start: Position::new(0, 12),
            end: Position::new(0, 15),
        });
        let extended_rng = Rng::new(Range {
            start: Position::new(0, 0),
            end: Position::new(0, 15),
        });
        assert_eq!(extended_rng, rng1.extend(rng2));
        let empty_rng = Rng::default();
        assert!(empty_rng.is_empty());
        assert_eq!(empty_rng, empty_rng.extend(empty_rng));
        assert_eq!(rng1, empty_rng.extend(rng1));
        assert_eq!(rng1, rng1.extend(empty_rng));
    }

    #[test]
    fn test_comes_before() {
        let rng1 = Rng::new(Range {
            start: Position::new(0, 0),
            end: Position::new(0, 5),
        });
        let rng2 = Rng::new(Range {
            start: Position::new(0, 12),
            end: Position::new(0, 15),
        });
        let rng3 = Rng::new(Range {
            start: Position::new(1, 12),
            end: Position::new(1, 15),
        });
        let rng4 = Rng::new(Range {
            start: Position::new(0, 0),
            end: Position::new(0, 4),
        });
        assert!(rng1.comes_before(&rng2));
        assert!(rng1.comes_before(&rng3));
        assert!(rng2.comes_before(&rng3));
        assert!(!rng2.comes_before(&rng1));
        assert!(!rng3.comes_before(&rng1));
        assert!(!rng1.comes_before(&rng4));
    }
}
