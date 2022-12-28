use lsp_types::{Position, Range};
use parol_runtime::lexer::Location;
use parol_runtime::once_cell::sync::Lazy;
use regex::Regex;

use crate::rng::Rng;

static RX_NEW_LINE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\r?\n").expect("error parsing regex: RX_NEW_LINE"));

///
/// Converts `parol_runtime::lexer::Location` to a `lsp_types::Range`.
///
pub(crate) fn location_to_range(location: &Location) -> Range {
    let start_char = location.start_column as u32 - 1;
    let end_char = start_char + location.length as u32;
    Range {
        start: Position {
            line: location.start_line as u32 - 1,
            character: start_char,
        },
        end: Position {
            line: location.end_line as u32 - 1,
            character: end_char,
        },
    }
}

///
/// Converts `parol_runtime::lexer::Location` to a `lsp_types::Location`.
/// Url is usually taken from the `LocatedDocumentState`.
///
pub(crate) fn location_to_location(
    location: &Location,
    uri: &lsp_types::Url,
) -> lsp_types::Location {
    lsp_types::Location {
        uri: uri.to_owned(),
        range: location_to_range(location),
    }
}

pub(crate) fn pos_to_offset(input: &str, pos: Position) -> usize {
    let mut offset = 0;
    for line in input.lines().take(pos.line as usize) {
        offset += line.len();
        // The lines returned from the Lines iterator will not have a newline byte (the 0xA byte)
        // or CRLF (0xD, 0xA bytes) at the end.
        // See https://doc.rust-lang.org/std/io/trait.BufRead.html#method.lines
        // We manually correct the offset after examining the newline bytes in the input.
        let (_, line_end) = input.split_at(offset);
        if line_end.starts_with("\r\n") {
            offset += 2; // Windows
        } else {
            offset += 1; // Linux, Mac
        }
    }
    if let Some(last_line) = input.lines().nth(pos.line as usize) {
        if !last_line.is_empty() {
            if let Some((p, _)) = last_line.char_indices().nth(pos.character as usize) {
                offset += p
            } else {
                offset += last_line.char_indices().last().unwrap().0 + 1
            }
        }
    }
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
        RX_NEW_LINE.replace_all(input, "  \n")
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
TrailingComma: [","^];"#;

    const CONTENT2: &str = r#"ÜÄÖ
EXPECTED
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
        {
            let start = Position {
                line: 1,
                character: 0,
            };
            let end = Position {
                line: 1,
                character: 8,
            };
            let rng = Rng::new(Range { start, end });
            assert_eq!("EXPECTED", extract_text_range(CONTENT2, rng));
        }
    }
}
