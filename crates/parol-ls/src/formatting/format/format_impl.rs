use lsp_types::{FormattingOptions, TextEdit};

use crate::{parol_ls_grammar::ParolLsGrammar, rng::Rng};

use super::super::comments::Comments;
use super::super::fmt_options::FmtOptions;
use super::traits::{Fmt, Format};
use std::collections::HashMap;

impl Format for ParolLsGrammar {
    fn format(&self, options: &FormattingOptions, comments: Comments) -> Vec<TextEdit> {
        let Some(ref grammar) = self.grammar else {
            return Vec::new();
        };
        
        // Build a map of comment classifications
        let mut comment_classifications = HashMap::new();
        for comment in &comments.comments {
            let classification = self.classify_comment(comment);
            let key = (comment.location.start_line, comment.location.start_column);
            comment_classifications.insert(key, classification);
        }
        
        let mut range = Rng::from(grammar);
        if let Some(last_comment) = comments.comments.back() {
            range = range.extend(Rng::from(last_comment));
        }
        let mut fmt_options: FmtOptions = options.into();
        fmt_options.comment_classifications = comment_classifications;
        
        let (new_text, comments) = grammar.txt(&fmt_options, comments);
        debug_assert!(comments.is_empty());
        vec![TextEdit {
            range: range.0,
            new_text,
        }]
    }
}
