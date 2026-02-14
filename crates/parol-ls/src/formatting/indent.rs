pub(crate) struct Indent;

impl Indent {
    #[allow(unused)]
    pub(crate) fn make_indent(depth: u16) -> String {
        let mut indent = String::with_capacity((depth as usize) * 4);
        indent.extend("    ".repeat(depth as usize).drain(..));
        indent
    }
}

#[cfg(test)]
mod test {
    use crate::formatting::{FmtOptions, indent::Indent};

    #[test]
    fn test_make_indent() {
        assert_eq!(String::from(""), Indent::make_indent(0));
        let options = FmtOptions::new();
        assert_eq!(String::from(""), Indent::make_indent(options.nesting_depth));
        assert_eq!(
            String::from("    "),
            Indent::make_indent(options.next_depth().nesting_depth)
        );
    }
}
