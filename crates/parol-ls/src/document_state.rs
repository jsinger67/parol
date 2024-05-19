use derive_new::new;
use lsp_types::{
    DocumentFormattingParams, DocumentSymbolParams, DocumentSymbolResponse, Hover, HoverParams,
    Position, PrepareRenameResponse, RenameParams, TextDocumentPositionParams, Uri, WorkspaceEdit,
};

use crate::parol_ls_grammar::ParolLsGrammar;

#[derive(Debug, Clone, Default, new)]
pub(crate) struct DocumentState {
    pub(crate) input: String,
    pub(crate) parsed_data: ParolLsGrammar,
}

impl DocumentState {
    pub(crate) fn ident_at_position(&self, position: Position) -> Option<&str> {
        self.parsed_data.ident_at_position(position).map(|s| s.0)
    }

    pub(crate) fn clear(&mut self) {
        self.parsed_data = ParolLsGrammar::default()
    }

    pub(crate) fn hover(&self, params: HoverParams) -> Hover {
        self.parsed_data.hover(params, &self.input)
    }

    pub(crate) fn document_symbols(&self, params: DocumentSymbolParams) -> DocumentSymbolResponse {
        self.parsed_data.document_symbols(params, &self.input)
    }

    pub(crate) fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> Option<PrepareRenameResponse> {
        self.parsed_data.prepare_rename(params)
    }

    pub(crate) fn rename(&self, params: RenameParams) -> Option<WorkspaceEdit> {
        self.parsed_data.rename(params)
    }

    pub(crate) fn format(
        &self,
        params: DocumentFormattingParams,
    ) -> Option<Vec<lsp_types::TextEdit>> {
        self.parsed_data.format(params)
    }
}

#[derive(Debug, new)]
pub(crate) struct LocatedDocumentState<'a> {
    pub(crate) uri: &'a Uri,
    pub(crate) document_state: &'a DocumentState,
}
