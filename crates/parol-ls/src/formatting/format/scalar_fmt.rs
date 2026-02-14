use crate::parol_ls_grammar_trait::{
    DoubleColon, Identifier, LiteralString, LookAheadGroup, Regex, TokenLiteral, UserTypeNameList,
};

use super::super::comments::Comments;
use super::super::fmt_options::FmtOptions;
use super::traits::Fmt;

impl Fmt for DoubleColon {
    fn txt(&self, _options: &FmtOptions, comments: Comments) -> (String, Comments) {
        (self.double_colon.text().to_string(), comments)
    }
}

impl Fmt for Identifier {
    fn txt(&self, _options: &FmtOptions, comments: Comments) -> (String, Comments) {
        (self.identifier.text().to_string(), comments)
    }
}

impl Fmt for LiteralString {
    fn txt(&self, _options: &FmtOptions, comments: Comments) -> (String, Comments) {
        (self.literal_string.text().to_string(), comments)
    }
}

impl Fmt for LookAheadGroup {
    fn txt(&self, _options: &FmtOptions, comments: Comments) -> (String, Comments) {
        let lookahead_group_str = match self {
            LookAheadGroup::PositiveLookahead(look_ahead_group_positive_lookahead) => {
                look_ahead_group_positive_lookahead
                    .positive_lookahead
                    .positive_lookahead
                    .text()
            }
            LookAheadGroup::NegativeLookahead(look_ahead_group_negative_lookahead) => {
                look_ahead_group_negative_lookahead
                    .negative_lookahead
                    .negative_lookahead
                    .text()
            }
        };
        (lookahead_group_str.to_string(), comments)
    }
}

impl Fmt for Regex {
    fn txt(&self, _options: &FmtOptions, comments: Comments) -> (String, Comments) {
        (self.regex.text().to_string(), comments)
    }
}

impl Fmt for TokenLiteral {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        match self {
            TokenLiteral::String(s) => s.string.txt(options, comments),
            TokenLiteral::LiteralString(l) => l.literal_string.txt(options, comments),
            TokenLiteral::Regex(r) => r.regex.txt(options, comments),
        }
    }
}

impl Fmt for UserTypeNameList {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        let (identifier, comments) = self.identifier.txt(options, comments);
        (
            format!("{}{}", self.double_colon.double_colon, identifier),
            comments,
        )
    }
}
