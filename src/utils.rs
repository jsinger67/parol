use lsp_types::{Position, Range};
use miette::SourceSpan;
use parol_runtime::lexer::Location;
use regex::Regex;

use crate::rng::Rng;

lazy_static! {
    static ref RX_NEW_LINE: Regex = Regex::new(r"\r?\n").expect("error parsing regex: RX_NEW_LINE");
}

// #[derive(Debug, new)]
// pub(crate) struct Loc<'a>(&'a Location);

// impl<'a> From<&'a Location> for Loc<'a> {
//     fn from(location: &'a Location) -> Self {
//         Self(location)
//     }
// }

// impl From<Loc<'_>> for Range {
//     fn from(val: Loc<'_>) -> Self {
//         location_to_range(val.0)
//     }
// }

///
/// Converts parol_runtime::lexer::Location to a lsp_types::Range.
/// The line is kept for both start and end position in the result which could lead to problems!
/// This can only be fixed by changing the location data of `parol_runtime`!
///
pub(crate) fn location_to_range(location: &Location) -> Range {
    let line = location.line as u32 - 1;
    let start_char = location.column as u32 - 1;
    let end_char = start_char + location.length as u32 + 1;
    Range {
        start: Position {
            line,
            character: start_char,
        },
        end: Position {
            line,
            character: end_char,
        },
    }
}

///
/// Converts miette::protocol::SourceSpan to a lsp_types::Range.
/// The line is kept for both start and end position in the result which could lead to problems!
///
pub(crate) fn source_code_span_to_range(input: &str, span: &SourceSpan) -> Range {
    let input = input.split_at(span.offset()).0;
    let line = input.lines().count() as u32 - 1;
    let start_char = input.lines().last().map_or(0, |l| l.len()) as u32;
    let end_char = start_char + span.len() as u32;
    let range = Range {
        start: Position {
            line,
            character: start_char,
        },
        end: Position {
            line,
            character: end_char,
        },
    };
    eprintln!("{:?} => {:?}", span, range);
    range
}

pub(crate) fn pos_to_offset(input: &str, pos: Position) -> usize {
    let mut start_line = pos.line;
    let mut last_char_was_line_end = false;
    let mut offset = 0;
    input
        .char_indices()
        .into_iter()
        .skip_while(|c| {
            offset = c.0;
            if (c.1 == '\n' || c.1 == '\r') && !last_char_was_line_end {
                last_char_was_line_end = true;
                if start_line > 0 {
                    start_line -= 1;
                }
            } else {
                last_char_was_line_end = false;
            }
            start_line > 0
        })
        .last()
        .unwrap_or_default()
        .0;
    offset = input
        .char_indices()
        .into_iter()
        .skip(offset)
        .skip(pos.character as usize + 1)
        .next()
        .unwrap_or_default()
        .0;
    offset
}

pub(crate) fn extract_text_range(input: &str, rng: Rng) -> &str {
    let start = pos_to_offset(input, rng.0.start);
    let end = pos_to_offset(input, rng.0.end);
    input.split_at(start).1.split_at(end - start).0
}

pub(crate) fn to_markdown(input: &str) -> String {
    format!(
        "```parol  \n{}  \n```",
        RX_NEW_LINE.replace_all(input, "  \n").to_string()
    )
}
