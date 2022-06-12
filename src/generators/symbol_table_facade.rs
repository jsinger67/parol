use crate::grammar::SymbolAttribute;

use super::symbol_table::{
    Instance, ScopeId, ScopedNameId, Symbol, SymbolId, SymbolKind, SymbolTable, Type, TypeEntrails,
};

pub(crate) trait SymbolFacade<'a> {
    fn name(&self) -> String;
    fn kind(&self) -> &'a SymbolKind;
    fn to_rust(&self) -> String;
    fn format(&self, scope_depth: usize) -> String;
    fn my_id(&self) -> SymbolId;
    fn name_id(&self) -> ScopedNameId;
}

pub(crate) trait InstanceFacade<'a>: SymbolFacade<'a> {
    fn type_id(&self) -> SymbolId;
    fn description(&self) -> String;
    fn sem(&self) -> SymbolAttribute;
    fn used(&self) -> bool;
}

pub(crate) trait TypeFacade<'a>: SymbolFacade<'a> {
    fn inner_name(&self) -> String;
    fn member_scope(&self) -> ScopeId;
    fn entrails(&self) -> &TypeEntrails;
    fn is_container(&self) -> bool;
}

pub(crate) struct SymbolItem<'a> {
    symbol: &'a Symbol,
    symbol_table: &'a SymbolTable,
}

impl<'a> SymbolItem<'a> {
    pub(crate) fn new(symbol: &'a Symbol, symbol_table: &'a SymbolTable) -> Self {
        Self {
            symbol,
            symbol_table,
        }
    }
}

impl<'a> SymbolFacade<'a> for SymbolItem<'a> {
    fn name(&self) -> String {
        self.symbol_table.name(self.symbol.name_id).to_string()
    }

    fn kind(&self) -> &'a SymbolKind {
        &self.symbol.kind
    }

    fn to_rust(&self) -> String {
        self.symbol.to_rust(self.symbol_table)
    }

    fn format(&self, scope_depth: usize) -> String {
        self.symbol.format(self.symbol_table, scope_depth)
    }

    fn my_id(&self) -> SymbolId {
        self.symbol.my_id
    }

    fn name_id(&self) -> ScopedNameId {
        self.symbol.name_id
    }
}

pub(crate) struct InstanceItem<'a> {
    symbol_item: SymbolItem<'a>,
    instance: &'a Instance,
}

impl<'a> InstanceItem<'a> {
    pub(crate) fn new(symbol_item: SymbolItem<'a>, instance: &'a Instance) -> Self {
        Self {
            symbol_item,
            instance,
        }
    }
}

impl<'a> SymbolFacade<'a> for InstanceItem<'a> {
    fn name(&self) -> String {
        self.symbol_item.name()
    }

    fn kind(&self) -> &'a SymbolKind {
        self.symbol_item.kind()
    }

    fn to_rust(&self) -> String {
        self.symbol_item.to_rust()
    }

    fn format(&self, scope_depth: usize) -> String {
        self.symbol_item.format(scope_depth)
    }

    fn my_id(&self) -> SymbolId {
        self.symbol_item.my_id()
    }

    fn name_id(&self) -> ScopedNameId {
        self.symbol_item.name_id()
    }
}

impl<'a> InstanceFacade<'a> for InstanceItem<'a> {
    fn type_id(&self) -> SymbolId {
        self.instance.type_id
    }

    fn description(&self) -> String {
        self.instance.description.clone()
    }

    fn sem(&self) -> SymbolAttribute {
        self.instance.sem
    }

    fn used(&self) -> bool {
        self.instance.used
    }
}

pub(crate) struct TypeItem<'a> {
    symbol_item: SymbolItem<'a>,
    my_type: &'a Type,
}

impl<'a> TypeItem<'a> {
    pub(crate) fn new(symbol_item: SymbolItem<'a>, my_type: &'a Type) -> Self {
        Self {
            symbol_item,
            my_type,
        }
    }
}

impl<'a> SymbolFacade<'a> for TypeItem<'a> {
    fn name(&self) -> String {
        if self.symbol_item.name_id().is_unnamed() {
            self.my_type
                .entrails
                .format(self.symbol_item.my_id(), self.symbol_item.symbol_table)
        } else {
            self.symbol_item.name()
        }
    }

    fn kind(&self) -> &'a SymbolKind {
        self.symbol_item.kind()
    }

    fn to_rust(&self) -> String {
        self.symbol_item.to_rust()
    }

    fn format(&self, scope_depth: usize) -> String {
        self.symbol_item.format(scope_depth)
    }

    fn my_id(&self) -> SymbolId {
        self.symbol_item.my_id()
    }

    fn name_id(&self) -> ScopedNameId {
        self.symbol_item.name_id()
    }
}

impl<'a> TypeFacade<'a> for TypeItem<'a> {
    fn inner_name(&self) -> String {
        self.my_type
            .inner_name(self.symbol_item.symbol_table, self.symbol_item.symbol)
    }

    fn member_scope(&self) -> ScopeId {
        self.my_type.member_scope
    }

    fn entrails(&self) -> &TypeEntrails {
        &self.my_type.entrails
    }

    fn is_container(&self) -> bool {
        self.my_type.is_container()
    }
}
