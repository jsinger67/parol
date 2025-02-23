use std::collections::{HashMap, HashSet};

use lsp_types::{Position, Range};

use crate::{parol_ls_grammar::OwnedToken, utils::location_to_range};

// DefsAndRefs is a structure that holds the definitions and references of a symbol.
// Definitions are stored in a set of ranges where the symbol is defined. Multiple definitions of
// the same symbol are allowed and actually needed for multiple productions of the same non-terminal
// symbol.
// References are stored in a set of ranges where the symbol is referenced.
#[derive(Debug, Clone, Default)]
pub(crate) struct DefsAndRefs {
    definitions: HashSet<Range>,
    references: HashSet<Range>,
}

// SymbolDefs is a structure that holds the definitions and references of symbols in the document.
// It is used to provide symbol information to the language server.
// The structure is a map from symbol names to DefsAndRefs structures.
// The structure provides methods to add definitions and references, and to find definitions,
// references, and the name of a reference at a given position.
#[derive(Debug, Clone, Default)]
pub(crate) struct SymbolDefs {
    pub(crate) symbols: HashMap<String, DefsAndRefs>,
}

impl SymbolDefs {
    // Add a definition of the symbol
    pub(crate) fn add_definition(&mut self, name: String, range: Range) {
        // eprintln!("add_definition: {:?}", name);
        self.symbols
            .entry(name)
            .or_default()
            .definitions
            .insert(range);
    }

    // Add a definition of the symbol by token
    pub(crate) fn add_definition_by_token(&mut self, token: &OwnedToken) {
        // eprintln!("add_definition_by_token: {:?}", token);
        self.symbols
            .entry(token.text().to_string())
            .or_default()
            .definitions
            .insert(location_to_range(&token.location));
    }

    // Add a reference to the symbol
    pub(crate) fn add_reference(&mut self, range: Range, name: &str) {
        self.symbols
            .entry(name.to_string())
            .or_default()
            .references
            .insert(range);
    }

    // Add a reference to the symbol by token
    pub(crate) fn add_reference_by_token(&mut self, token: &OwnedToken) {
        self.symbols
            .entry(token.text().to_string())
            .or_default()
            .references
            .insert(location_to_range(&token.location));
    }

    // Find the name of the reference at the given position
    pub(crate) fn find_reference(&self, position: Position) -> Option<&str> {
        for (name, dr) in &self.symbols {
            for range in &dr.references {
                if range.start <= position && range.end > position {
                    return Some(name);
                }
            }
        }
        None
    }

    // Find the definitions of the given name
    // Return only a vector of ranges if there are definitions, otherwise return None
    pub(crate) fn find_definitions(&self, name: &str) -> Option<Vec<Range>> {
        self.symbols.get(name).and_then(|s| {
            let definitions: Vec<Range> = s.definitions.iter().cloned().collect();
            if definitions.is_empty() {
                None
            } else {
                Some(definitions)
            }
        })
    }

    // Find the ranges of all references to the given name
    pub(crate) fn find_references(&self, name: &str) -> Vec<&Range> {
        self.symbols
            .get(name)
            .map_or_else(Vec::new, |s| s.references.iter().collect())
    }

    // Find the range of the reference at the given position
    pub(crate) fn find_reference_range(&self, name: &str, position: Position) -> Option<&Range> {
        self.symbols.get(name).and_then(|s| {
            s.references
                .iter()
                .find(|range| range.start <= position && range.end > position)
        })
    }
}
