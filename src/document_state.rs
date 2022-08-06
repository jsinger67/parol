use derive_new::new;
use lsp_types::{Position, Url};

use crate::parol_ls_grammar::ParolLsGrammar;

#[derive(Debug, Default, new)]
pub(crate) struct DocumentState {
    pub(crate) input: String,
    pub(crate) parsed_data: ParolLsGrammar,
}

impl DocumentState {
    pub(crate) fn ident_at_position(&self, position: Position) -> Option<String> {
        self.parsed_data.ident_at_position(position)
    }
}

#[derive(Debug, new)]
pub(crate) struct LocatedDocumentState<'a> {
    pub(crate) uri: &'a Url,
    pub(crate) document_state: &'a DocumentState,
}
