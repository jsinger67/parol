use crate::{parol_ls_grammar::OwnedToken, utils::location_to_range};

impl From<&OwnedToken> for lsp_types::DocumentSymbol {
    fn from(token: &OwnedToken) -> Self {
        #[allow(deprecated)]
        lsp_types::DocumentSymbol {
            name: token.text().to_owned(),
            detail: None,
            kind: lsp_types::SymbolKind::PROPERTY,
            tags: None,
            deprecated: None,
            range: location_to_range(&token.location),
            selection_range: location_to_range(&token.location),
            children: None,
        }
    }
}
