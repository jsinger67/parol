use lsp_types::{FormattingOptions, TextEdit};

use super::super::{Comments, FmtOptions};

pub(crate) trait Format {
    fn format(&self, options: &FormattingOptions, comments: Comments) -> Vec<TextEdit>;
}

pub(crate) trait Fmt {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments);
}
