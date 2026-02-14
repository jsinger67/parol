use crate::parol_ls_grammar_trait::{Prolog, PrologList, PrologList0};

use super::super::fmt_options::FmtOptions;
use super::super::comments::Comments;
use super::traits::Fmt;

impl Fmt for Prolog {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        let (start_declaration, comments) = self.start_declaration.txt(options, comments);
        let (prolog_list, comments) =
            self.prolog_list
                .iter()
                .fold((String::new(), comments), |(mut acc, comments), p| {
                    let (pro_str, comments) = p.txt(options, comments);
                    acc.push_str(&pro_str);
                    (acc, comments)
                });
        let (prolog_list0, comments) =
            self.prolog_list0
                .iter()
                .fold((String::new(), comments), |(mut acc, comments), p| {
                    let (pro_str, comments) = p.txt(options, comments);
                    acc.push_str(&pro_str);
                    (acc, comments)
                });
        (
            format!("{start_declaration}{prolog_list}\n{prolog_list0}"),
            comments,
        )
    }
}

impl Fmt for PrologList {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        self.declaration.txt(options, comments)
    }
}

impl Fmt for PrologList0 {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        self.scanner_state.txt(options, comments)
    }
}
