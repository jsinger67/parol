pub(crate) struct Line;

// pub(crate) static RX_NEW_LINES_AFTER_LINE_COMMENT: Lazy<regex::Regex> = Lazy::new(|| {
//     regex::Regex::new(r"//.*(\r?\n)+$")
//         .expect("error parsing regex: RX_NEW_LINES_AFTER_LINE_COMMENT")
// });

impl Line {
    pub(crate) fn ends_with_space(line: &str) -> bool {
        line.ends_with(' ')
    }

    pub(crate) fn ends_with_nl(line: &str) -> bool {
        line.ends_with(|c| c == '\n' || c == '\r')
    }

    // fn ends_with_nls_after_line_comment(line: &str) -> bool {
    //     RX_NEW_LINES_AFTER_LINE_COMMENT.is_match(line)
    // }
}
