use std::ops::{Add, Deref, Range};

/// This trait should be implemented by generated AST data types
pub trait ToSpan {
    /// Calculates the span of the implementing item
    fn span(&self) -> Span;
}

/// The Span type is a customized Range that can handle extension of ranges.
/// Span and std::ops::Range are convertible into each other.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Span(Range<usize>);

impl Span {
    /// Returns a new Span instance with the given start end end indices
    pub fn new(start: usize, end: usize) -> Span {
        Span(Range { start, end })
    }

    /// Generates a union of both ranges.
    /// It handles the special empty range.
    /// Empty ranges are null elements for `extend` operations:
    ///
    /// R1 + Empty = R1
    /// Empty + R1 = R1
    /// Empty + Empty = Empty
    ///
    pub fn extend(&self, right: &Span) -> Span {
        if self.is_empty() {
            right.clone()
        } else if right.is_empty() {
            self.clone()
        } else {
            Span::new(self.start, right.end)
        }
    }

    /// Same as [extend] but consuming RHS
    pub fn extend_to(&self, right: Span) -> Span {
        if self.is_empty() {
            right
        } else if right.is_empty() {
            self.clone()
        } else {
            Span::new(self.start, right.end)
        }
    }

    // pub fn from_slice<'a, T>(slc: &'a [T]) -> Self
    // where
    //     &'a T: Into<Span>,
    // {
    //     if slc.is_empty() {
    //         Span::default()
    //     } else {
    //         let first: &T = slc.first().unwrap();
    //         let rng: Span = first.into();
    //         let last: &T = slc.last().unwrap();
    //         let span = last.into();
    //         rng.extend(&span)
    //     }
    // }
}

impl Deref for Span {
    type Target = Range<usize>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Add for Span {
    type Output = Self;

    /// Same as [extend] but consuming LHS and RHS
    fn add(self, rhs: Self) -> Self::Output {
        if self.is_empty() {
            rhs
        } else if rhs.is_empty() {
            self
        } else {
            // Addition is commutative, so the order is irrelevant for the result
            Span::new(
                std::cmp::min(self.start, rhs.start),
                std::cmp::max(self.end, rhs.end),
            )
        }
    }
}

impl From<Span> for Range<usize> {
    fn from(val: Span) -> Range<usize> {
        val.0
    }
}

impl From<&Span> for Range<usize> {
    fn from(val: &Span) -> Range<usize> {
        val.0.clone()
    }
}

impl From<Range<usize>> for Span {
    fn from(val: Range<usize>) -> Span {
        Span(val)
    }
}

impl From<&Range<usize>> for Span {
    fn from(val: &Range<usize>) -> Span {
        Span(val.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::Range;

    #[test]
    fn test_extend() {
        let rng1 = Span::new(0, 5);
        let rng2 = Span::new(12, 15);
        let extended_rng = Span::new(0, 15);
        assert_eq!(extended_rng, rng1.extend_to(rng2));
        let empty_rng = Span::default();
        assert!(empty_rng.is_empty());
        assert_eq!(empty_rng, empty_rng.extend(&empty_rng));
        assert_eq!(rng1, empty_rng.extend(&rng1));
        assert_eq!(rng1, rng1.extend(&empty_rng));
    }

    #[test]
    fn test_add() {
        // With non-overlapping ranges
        assert_eq!(Span::new(0, 15), Span::new(0, 5) + Span::new(12, 15));
        assert_eq!(Span::new(0, 15), Span::new(12, 15) + Span::new(0, 5));

        // With overlapping ranges
        assert_eq!(Span::new(0, 7), Span::new(0, 5) + Span::new(3, 7));
        assert_eq!(Span::new(0, 7), Span::new(3, 7) + Span::new(0, 5));

        // With smallest overlapping ranges
        assert_eq!(Span::new(0, 7), Span::new(0, 5) + Span::new(4, 7));
        assert_eq!(Span::new(0, 7), Span::new(4, 7) + Span::new(0, 5));

        // With adjoining ranges
        assert_eq!(Span::new(0, 7), Span::new(0, 5) + Span::new(5, 7));
        assert_eq!(Span::new(0, 7), Span::new(5, 7) + Span::new(0, 5));

        // With empty ranges
        assert_eq!(Span::default(), Span::default() + Span::default());
        assert_eq!(Span::new(0, 5), Span::default() + Span::new(0, 5));
        assert_eq!(Span::new(0, 5), Span::new(0, 5) + Span::default());
    }

    #[test]
    fn test_conversions() {
        let span: Span = Range { start: 1, end: 2 }.into();
        assert_eq!(Span::new(1, 2), span);
        let span: Span = (&Range { start: 2, end: 3 }).into();
        assert_eq!(Span::new(2, 3), span);

        let range: Range<usize> = Span::new(1, 2).into();
        assert_eq!(Range { start: 1, end: 2 }, range);
        let range: Range<usize> = (&Span::new(2, 3)).into();
        assert_eq!(Range { start: 2, end: 3 }, range);
    }

    #[test]
    fn test_string_application() {
        let s = "Löwe 老虎 Léopard";
        // Bytes:
        // 0  1  2  3  4  5  6  7  8  9  10 11 12 13 14 15 16 17 18 19 20
        // 4c c3 b6 77 65 20 e8 80 81 e8 99 8e 20 4c c3 a9 6f 70 61 72 64
        // Chars:
        // 0  1     2  3  4  5        6        7  8  9     10 11 12 13 14
        // L  ö     w  e  _  老       虎       _  L  é     o  p  a  r  d
        assert_eq!(21, s.bytes().len());
        // for (i, b) in s.bytes().enumerate() {
        //     print!("{i}: 0x{b:x}");
        // }
        assert_eq!(15, s.chars().count());
        // for (i, c) in s.chars().enumerate() {
        //     println!("{i}: {c}");
        // }
        assert_eq!("老", &s[6..9]);
        assert_eq!("虎", &s[9..12]);
        assert_eq!("L", &s[13..14]);

        assert_eq!("老", &s[Range { start: 6, end: 9 }]);
        assert_eq!("虎", &s[Into::<Range<usize>>::into(Span::new(9, 12))]);
        assert_eq!("L", &s[Into::<Range<usize>>::into(&Span::new(13, 14))]);
    }
}
