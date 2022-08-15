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
    let mut offset = 0;
    for line in input.lines().into_iter().take(pos.line as usize) {
        offset += line.len();
        // The lines returned from the Lines iterator will not have a newline byte (the 0xA byte)
        // or CRLF (0xD, 0xA bytes) at the end.
        // See https://doc.rust-lang.org/std/io/trait.BufRead.html#method.lines
        // We manually correct the offset after examining the newline bytes in the input.
        if input.split_at(offset).1.starts_with("\r\n") {
            offset += 2;    // Windows
        } else {
            offset += 1;    // Linux, Mac
        }
    }
    offset = input
        .char_indices()
        .into_iter()
        .skip(offset)
        .skip(pos.character as usize)
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

#[cfg(test)]
mod tests {
    use super::*;
    use lsp_types::{Position, Range};

    const CONTENT: &str = r#"%start List
%title "A possibly empty comma separated list of integers"
%comment "A trailing comma is allowed."
%user_type Number = crate::list_grammar::Number
%user_type Numbers = crate::list_grammar::Numbers

%%

List: [Items: Numbers] TrailingComma^;
Items: Num {","^ Num};
Num: "0|[1-9][0-9]*": Number;
TrailingComma: [","^];
"#;

    #[test]
    fn test_pos_to_offset() {
        {
            let start = Position {
                line: 0,
                character: 0,
            };
            let end = Position {
                line: 0,
                character: 6,
            };
            let rng = Rng::new(Range { start, end });
            assert_eq!("%start", extract_text_range(CONTENT, rng));
        }
        {
            let start = Position {
                line: 1,
                character: 0,
            };
            let end = Position {
                line: 1,
                character: 6,
            };
            let rng = Rng::new(Range { start, end });
            assert_eq!("%title", extract_text_range(CONTENT, rng));
        }
        {
            let start = Position {
                line: 6,
                character: 0,
            };
            let end = Position {
                line: 6,
                character: 2,
            };
            let rng = Rng::new(Range { start, end });
            assert_eq!("%%", extract_text_range(CONTENT, rng));
        }
        {
            let start = Position {
                line: 11,
                character: 21,
            };
            let end = Position {
                line: 11,
                character: 22,
            };
            let rng = Rng::new(Range { start, end });
            assert_eq!(";", extract_text_range(CONTENT, rng));
        }
    }
}
