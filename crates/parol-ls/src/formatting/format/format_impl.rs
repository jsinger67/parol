use lsp_types::{FormattingOptions, TextEdit};

use crate::{parol_ls_grammar::ParolLsGrammar, rng::Rng};

use super::super::comments::Comments;
use super::traits::{Fmt, Format};

impl Format for ParolLsGrammar {
    fn format(&self, options: &FormattingOptions, comments: Comments) -> Vec<TextEdit> {
        let Some(ref grammar) = self.grammar else {
            return Vec::new();
        };
        let mut range = Rng::from(grammar);
        if let Some(last_comment) = comments.comments.back() {
            range = range.extend(Rng::from(last_comment));
        }
        let fmt_options = options.into();
        let (new_text, comments) = grammar.txt(&fmt_options, comments);
        debug_assert!(comments.is_empty());
        vec![TextEdit {
            range: range.0,
            new_text,
        }]
    }
}
