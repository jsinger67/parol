//! This module is used to serialize the regex patterns to a byte array.
//! The byte array is then used to build a regex from the byte array in the generated parser.
//!

use std::fmt::Display;

use anyhow::{anyhow, Result};
use parol_runtime::{log::trace, TerminalIndex};
use regex_automata::dfa::regex::Regex;

/// This struct is used to store a byte array and implement the Display trait for it.
/// The Display trait is used to print the byte array in Rust format into the generated parser.
pub(crate) struct ByteArray(Vec<u8>);

impl ByteArray {
    pub fn new(data: Vec<u8>) -> Self {
        ByteArray(data)
    }

    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }
}

impl Display for ByteArray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Display the byte array as a list of numbers in square brackets.
        // Write a fixed size per line.
        let mut count = 0;
        writeln!(f, "[")?;
        for b in self.0.iter() {
            if count == 0 {
                write!(f, "    ")?;
            }
            // Write the byte value in hexadecimal format and add a comma and a space.
            write!(f, "{:#04x}, ", b)?;
            count += 1;
            if count == 16 {
                count = 0;
                writeln!(f)?;
            }
        }
        if count != 0 {
            // If the last line is not full, add a newline.
            writeln!(f)?;
        }
        write!(f, "]")
    }
}

/// This struct is used to serialize the regex patterns to byte arrays.
pub(crate) struct RegexSerializer {}

impl RegexSerializer {
    /// Serializes the regex patterns to two byte arrays.
    /// The first byte array contains the forward DFA and the second byte array the reverse DFA.
    /// The byte arrays are then used to build a regex from the byte arrays in the generated parser.
    ///
    /// ## augmented_terminals
    /// All valid terminals of the grammar. These include the specific common terminals
    /// `EOI`, `NEW_LINE`, `WHITESPACE`, `LINE_COMMENT`, `BLOCK_COMMENT` with the value
    /// `UNMATCHABLE_TOKEN` to provide consistent index handling for all scanner states.
    ///
    /// ## scanner_specifics
    /// The values of the five scanner specific common terminals `EOI`, `NEW_LINE`, `WHITESPACE`,
    /// `LINE_COMMENT` and `BLOCK_COMMENT`
    ///
    /// ## scanner_terminal_indices
    /// The indices of token types belonging to this scanner state. These indices are pointing into
    /// `augmented_terminals`.
    ///
    /// # Errors
    ///
    /// This function will return an error if the regex patterns can't be compiled.
    pub(crate) fn serialize(
        augmented_terminals: &[String],
        scanner_specifics: &[String],
        scanner_terminal_indices: &[TerminalIndex],
        token_types: &mut Vec<TerminalIndex>,
    ) -> Result<(ByteArray, ByteArray)> {
        debug_assert_eq!(5, scanner_specifics.len());
        let internal_terminals = scanner_specifics.iter().enumerate().fold(
            Vec::with_capacity(augmented_terminals.len()),
            |mut acc, (i, t)| {
                if *t != "UNMATCHABLE_TOKEN" {
                    acc.push(t.clone());
                    token_types.push(i as TerminalIndex);
                }
                acc
            },
        );
        let mut patterns = scanner_terminal_indices
            .iter()
            .map(|term_idx| (*term_idx, augmented_terminals[*term_idx as usize].clone()))
            .fold(internal_terminals, |mut acc, (term_idx, pattern)| {
                acc.push(pattern);
                token_types.push(term_idx);
                acc
            });
        let error_token_type = (augmented_terminals.len() - 1) as TerminalIndex;

        patterns.push(augmented_terminals[error_token_type as usize].clone());
        token_types.push(error_token_type);

        debug_assert_eq!(
            patterns.len(),
            token_types.len(),
            "Error in mapping of PatternID to TerminalIndex"
        );

        trace!("Generated regex for scanner:\n{:?}", patterns);
        let rx = Regex::builder()
            .build_many(&patterns)
            .map_err(|e| anyhow!(e))?;

        let (fwd, rev) = (rx.forward(), rx.reverse());
        let fwd_bytes = fwd.to_bytes_native_endian().0;
        let rev_bytes = rev.to_bytes_native_endian().0;
        Ok((ByteArray::new(fwd_bytes), ByteArray::new(rev_bytes)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_byte_array_display() {
        let data = vec![1, 2, 3, 4, 5];
        let byte_array = ByteArray::new(data);
        assert_eq!(
            format!("{}", byte_array),
            r#"[
    0x01, 0x02, 0x03, 0x04, 0x05, 
]"#
        );
    }

    #[test]
    fn test_byte_array_display_multiple_lines() {
        let data = vec![
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
        ];
        let byte_array = ByteArray::new(data);
        assert_eq!(
            format!("{}", byte_array),
            r#"[
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10, 
    0x11, 0x12, 0x13, 0x14, 
]"#
        );
    }

    #[test]
    fn test_byte_array_display_multiple_lines_complete() {
        let data = vec![
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
            25, 26, 27, 28, 29, 30, 31, 32,
        ];
        let byte_array = ByteArray::new(data);
        assert_eq!(
            format!("{}", byte_array),
            r#"[
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10, 
    0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20, 
]"#
        );
    }
}
