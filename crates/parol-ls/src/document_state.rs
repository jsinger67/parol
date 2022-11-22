use derive_new::new;
use lsp_types::{
    DocumentFormattingParams, DocumentSymbolParams, DocumentSymbolResponse, Hover, HoverParams,
    Position, PrepareRenameResponse, Range, RenameParams, TextDocumentPositionParams, Url,
    WorkspaceEdit,
};

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

    pub(crate) fn clear(&mut self) {
        self.parsed_data = ParolLsGrammar::default()
    }

    pub(crate) fn _find_left_recursions(&self) -> Vec<Vec<Range>> {
        self.parsed_data._find_left_recursions()
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
    pub(crate) uri: &'a Url,
    pub(crate) document_state: &'a DocumentState,
}
