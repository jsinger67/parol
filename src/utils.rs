use lsp_types::{Position, Range};
use miette::SourceSpan;
use parol_runtime::lexer::Location;

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
