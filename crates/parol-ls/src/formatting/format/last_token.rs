use crate::{
    parol_ls_grammar::OwnedToken,
    parol_ls_grammar_trait::{
        ScannerDirectivesPercentOnIdentifierListScannerStateDirectives, ScannerStateDirectives,
        TokenLiteral, UserTypeName,
    },
};

pub(super) trait LastToken {
    fn get_last_token(&self) -> &OwnedToken;
}

impl LastToken for UserTypeName {
    fn get_last_token(&self) -> &OwnedToken {
        if self.user_type_name_list.is_empty() {
            &self.identifier.identifier
        } else {
            &self
                .user_type_name_list
                .last()
                .unwrap()
                .identifier
                .identifier
        }
    }
}

impl LastToken for TokenLiteral {
    fn get_last_token(&self) -> &OwnedToken {
        match self {
            TokenLiteral::String(s) => &s.string.string,
            TokenLiteral::LiteralString(l) => &l.literal_string.literal_string,
            TokenLiteral::Regex(r) => &r.regex.regex,
        }
    }
}

impl LastToken for ScannerDirectivesPercentOnIdentifierListScannerStateDirectives {
    fn get_last_token(&self) -> &OwnedToken {
        match &self.scanner_state_directives {
            ScannerStateDirectives::PercentEnterIdentifier(
                scanner_state_directives_percent_enter_identifier,
            ) => {
                &scanner_state_directives_percent_enter_identifier
                    .identifier
                    .identifier
            }
            ScannerStateDirectives::PercentPushIdentifier(
                scanner_state_directives_percent_push_identifier,
            ) => {
                &scanner_state_directives_percent_push_identifier
                    .identifier
                    .identifier
            }
            ScannerStateDirectives::PercentPop(scanner_state_directives_percent_pop) => {
                &scanner_state_directives_percent_pop.percent_pop
            }
        }
    }
}
