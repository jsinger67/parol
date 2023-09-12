use std::ops::Range;

use log::trace;

use crate::TerminalIndex;

pub(crate) struct Recovery;

impl Recovery {
    ///
    /// Calculates valid match ranges for actually scanned terminals and expected terminals.
    /// The Strategy is to match a (sub) range of the scanned terminals completely with a sub range
    /// of the expected terminals. Not matching prefixes can exist and are corrected later by the
    /// parser during recovery.
    /// To be successful the sub match must either reach until the end of the input tokens or
    /// until the end of the expected tokens.
    /// This is checked in the inner loop by evaluating the result of the inner [try_expand].
    ///
    pub(crate) fn calculate_match_ranges(
        act: &[TerminalIndex],
        exp: &[TerminalIndex],
    ) -> Option<(Range<usize>, Range<usize>)> {
        if act.is_empty() || exp.is_empty() {
            return None;
        }

        // Create and fill the match matrix.
        // The matrix should be very small, i.e. significantly smaller than 10x10.
        let mut m: Vec<Vec<bool>> = vec![vec![false; exp.len()]; act.len()];
        for (ia, a) in act.iter().enumerate() {
            for (ie, e) in exp.iter().enumerate() {
                if *a == *e {
                    m[ia][ie] = true;
                }
            }
        }

        // Tries to follow a diagonal line in the match matrix.
        let try_expand = |mut ia: usize, mut ie: usize| -> Option<(usize, usize)> {
            let mut result = None;
            while m[ia][ie] {
                result = Some((ia, ie));
                ia += 1;
                ie += 1;
                if ia >= act.len() || ie >= exp.len() {
                    break;
                }
            }
            result
        };

        trace!("exp: {exp:?}");
        for (i, c) in m.iter().enumerate() {
            trace!("{}: {c:?}", act[i]);
        }

        let mut result = None;
        let mut a_start = None;
        let mut a_end = None;
        let mut e_start = None;
        let mut e_end = None;
        let ia_max = act.len() - 1;
        let ie_max = exp.len() - 1;

        'OUTER: for (ia, a) in m.iter().enumerate() {
            for (ie, eq) in a.iter().enumerate() {
                if a_start.is_none() && *eq {
                    a_start = Some(ia);
                    e_start = Some(ie);
                    if let Some((ia_end, ie_end)) = try_expand(ia, ie) {
                        if ie_end == ie_max || ia_end == ia_max {
                            a_end = Some(ia_end);
                            e_end = Some(ie_end);
                            break 'OUTER;
                        } else {
                            a_start = None;
                            e_start = None;
                            a_end = None;
                            e_end = None;
                        }
                    } else {
                        a_start = None;
                        e_start = None;
                        a_end = None;
                        e_end = None;
                    }
                }
            }
        }
        trace!("{:?}", (a_start, a_end, e_start, e_end));
        if let (Some(a0), Some(a1), Some(e0), Some(e1)) = (a_start, a_end, e_start, e_end) {
            // Ranges are excluding, thus we increment the upper limits.
            result = Some((a0..a1 + 1, e0..e1 + 1));
        }
        trace!("result: {result:?}");
        result
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    type TestData = &'static [(
        (&'static [TerminalIndex], &'static [TerminalIndex]),
        Option<(Range<usize>, Range<usize>)>,
    )];
    #[test]
    fn test_calculate_match_ranges() {
        let test_data: TestData = &[
            ((&[1, 2], &[0, 2]), Some((1..2, 1..2))),
            ((&[7, 8, 9, 10], &[8, 8, 9, 10]), Some((1..4, 1..4))),
            ((&[7, 8, 9, 10], &[8, 8, 9]), Some((1..3, 1..3))),
            ((&[8, 2, 8, 2, 8], &[1, 8, 2, 8]), Some((0..3, 1..4))),
            ((&[7, 8, 9, 10], &[8, 8]), Some((1..2, 1..2))),
            ((&[7, 8, 9, 10], &[8]), Some((1..2, 0..1))),
            ((&[7, 8, 9, 10], &[]), None),
            ((&[6, 7, 8, 9], &[8, 8, 9, 10]), Some((2..4, 1..3))),
            ((&[6, 7, 8, 9, 11], &[8, 8, 9, 10]), None),
            ((&[6, 7, 8, 9, 10], &[8, 8, 9, 10, 11]), Some((2..5, 1..4))),
        ];
        INIT.call_once(env_logger::init);
        for (i, d) in test_data.iter().enumerate() {
            assert_eq!(
                d.1,
                Recovery::calculate_match_ranges(d.0 .0, d.0 .1),
                "test case {i} failed for input {:?}",
                d.0
            );
        }
    }
}
