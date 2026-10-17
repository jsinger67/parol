use super::fmt_options::IndentStyle;

pub(crate) struct Indent;

impl Indent {
    pub(crate) fn make_indent(depth: u16, style: &IndentStyle) -> String {
        match style {
            IndentStyle::Spaces(spaces) => {
                let spaces = *spaces as usize;
                let mut indent = String::with_capacity((depth as usize) * spaces);
                indent.extend(" ".repeat(depth as usize * spaces).drain(..));
                indent
            }
            IndentStyle::Tabs => {
                let mut indent = String::with_capacity(depth as usize);
                indent.extend("	".repeat(depth as usize).drain(..));
                indent
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::formatting::{FmtOptions, indent::Indent};

    #[test]
    fn test_make_indent() {
        assert_eq!(String::from(""), Indent::make_indent(0, &FmtOptions::default().indent_style));
        let options = FmtOptions::new();
        assert_eq!(String::from(""), Indent::make_indent(options.nesting_depth, &options.indent_style));
        let next_options = options.clone();
        let next_depth = next_options.next_depth().nesting_depth;
        assert_eq!(
            String::from("    "),
            Indent::make_indent(next_depth, &options.indent_style)
        );
    }
}
