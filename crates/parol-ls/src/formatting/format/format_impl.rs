use lsp_types::{FormattingOptions, Range, TextEdit};

use crate::{parol_ls_grammar_trait::ParolLs, rng::Rng};

use super::super::comments::Comments;
use super::traits::{Fmt, Format};

impl Format for &ParolLs {
    fn format(&self, options: &FormattingOptions, comments: Comments) -> Vec<TextEdit> {
        let range = Rng::new(Range::default()).extend_to_end().0;
        let fmt_options = options.into();
        let (new_text, comments) = self.txt(&fmt_options, comments);
        debug_assert!(comments.is_empty());
        vec![TextEdit { range, new_text }]
    }
}
