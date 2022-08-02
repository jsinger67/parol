use lsp_types::{Position, Range};
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
