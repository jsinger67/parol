use crate::analysis::lookahead_dfa::ProductionIndex;
use crate::grammar::{ProductionAttribute, SymbolAttribute};
use crate::{generators::NamingHelper as NmHlp, utils::generate_name};
use miette::{miette, Result};

use std::fmt::{Debug, Display, Error, Formatter};

/// Index type for Symbols
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct SymbolId(usize);

/// Index type for SymbolNames
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct NameId(usize);

/// Index type for SymbolNames
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct ScopeId(usize);

fn build_indent(amount: usize) -> String {
    const MULTIPLIER: usize = 4;
    let space = " ".to_string();
    space.repeat(amount * MULTIPLIER)
}

///
/// Type specificities of a function type
///
#[derive(Builder, Clone, Debug, Default, PartialEq)]
pub(crate) struct Function {
    /// Associated non-terminal
    pub(crate) non_terminal: String,

    /// Semantic specification
    pub(crate) sem: ProductionAttribute,

    /// Production number
    pub(crate) prod_num: ProductionIndex,

    /// The relative index of a production within its alternatives.
    pub(crate) rel_idx: usize,

    /// Formatted production in PAR syntax.
    pub(crate) prod_string: String,
}

impl Function {
    pub(crate) fn format(&self, type_id: SymbolId, symbol_table: &SymbolTable) -> String {
        "fn".to_string()
    }
}

///
/// Type information used for auto-generation
///
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum TypeEntrails {
    /// Not specified, used as prototype during generation
    None,
    /// Unit type ()
    Unit,
    /// Will be generated as Token structure
    Token,
    /// A type name
    TypeRef,
    /// A type name (without Box semantic)
    TypeName,
    /// A struct, i.e. a named collection of (name, type) tuples
    Struct,
    /// Will be generated as enum with given name
    Enum,
    /// Will be generated as Vec<T> where T is the type, similar to TypeRef
    Repeat,
    /// A trait, normally the semantic actions trait  generated for the user grammar
    Trait,
    /// A trait function
    Function(Function),
}

impl TypeEntrails {
    fn format(&self, type_id: SymbolId, symbol_table: &SymbolTable) -> String {
        format!(
            "{}",
            match self {
                TypeEntrails::None => "*TypeError*".to_string(),
                TypeEntrails::Unit => "()".to_string(),
                TypeEntrails::Token => "Token".to_string(),
                TypeEntrails::TypeRef => "TypeRef".to_string(),
                TypeEntrails::TypeName => "TypeName".to_string(),
                TypeEntrails::Struct => "struct".to_string(),
                TypeEntrails::Enum => "enum".to_string(),
                TypeEntrails::Repeat => "Vec".to_string(),
                TypeEntrails::Trait => "trait".to_string(),
                TypeEntrails::Function(f) => f.format(type_id, symbol_table),
            }
        )
    }
}

impl Default for TypeEntrails {
    fn default() -> Self {
        Self::None
    }
}

///
/// Type information used for auto-generation
///
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Type {
    /// The symbol's id in the symbol table
    pub(crate) my_id: SymbolId,

    /// The symbol name's id in the enveloping scope
    pub(crate) name_id: NameId,

    /// The type specificities
    pub(crate) entrails: TypeEntrails,

    /// The inner scope
    pub(crate) member_scope: ScopeId,
}

impl Type {
    fn format(&self, symbol_table: &SymbolTable, scope_depth: usize) -> String {
        format!(
            "{}{} {} {{ // Type: my_id {}, name_id {}\n{}\n{}}}",
            build_indent(scope_depth),
            self.entrails.format(self.my_id, &symbol_table),
            self.name(symbol_table),
            self.my_id.0,
            self.name_id.0,
            symbol_table
                .scope(self.member_scope)
                .format(symbol_table, scope_depth + 1),
            build_indent(scope_depth)
        )
    }

    pub(crate) fn name<'a>(&self, symbol_table: &'a SymbolTable) -> &'a str {
        let member_scope = symbol_table.scope(self.member_scope);
        if let Some(parent_scope_id) = member_scope.parent {
            symbol_table.scope(parent_scope_id).name(self.name_id)
        } else {
            "<unknown>"
        }
    }
}

///
/// A typed instance, usually a function argument or a struct member
///
#[derive(Builder, Clone, Debug, Default, PartialEq)]
pub(crate) struct Instance {
    /// The symbol name's id in the enveloping scope
    pub(crate) name_id: NameId,

    /// The instance's type id in the symbol table
    pub(crate) type_id: SymbolId,

    /// Indicates if the argument is used
    pub(crate) used: bool,

    /// Semantic information
    pub(crate) sem: SymbolAttribute,
}

impl Instance {
    fn format(&self, _symbol_table: &SymbolTable) -> String {
        todo!()
    }
}

///
/// A more general symbol
///
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Symbol {
    Type(Type),
    Instance(Instance),
}

impl Symbol {
    fn has_lifetime(&self, symbol_table: &SymbolTable) -> bool {
        match self {
            Self::Type(t) => match t.entrails {
                TypeEntrails::None | TypeEntrails::Unit | TypeEntrails::Function(_) => false,
                TypeEntrails::Token
                | TypeEntrails::TypeRef
                | TypeEntrails::TypeName
                | TypeEntrails::Repeat
                | TypeEntrails::Trait => true,
                TypeEntrails::Struct | TypeEntrails::Enum => symbol_table
                    .scope(t.member_scope)
                    .symbols
                    .iter()
                    .any(|e| symbol_table.has_lifetime(*e)),
            },
            Self::Instance(_) => false,
        }
    }

    pub(crate) fn lifetime(&self, symbol_table: &SymbolTable) -> String {
        if self.has_lifetime(symbol_table) {
            "<'t>".to_string()
        } else {
            "".to_string()
        }
    }

    pub(crate) fn format(&self, symbol_table: &SymbolTable, scope_depth: usize) -> String {
        match self {
            Symbol::Type(t) => t.format(symbol_table, scope_depth),
            Symbol::Instance(i) => i.format(symbol_table),
        }
    }

    fn member_scope(&self) -> Result<ScopeId> {
        match self {
            Symbol::Type(t) => Ok(t.member_scope),
            Symbol::Instance(_) => Err(miette!(
                "Instance has no member scope. Use instance's type!"
            )),
        }
    }
}

///
/// Scope with symbols inside
///
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Scope {
    pub(crate) parent: Option<ScopeId>,
    pub(crate) my_id: ScopeId,
    pub(crate) symbols: Vec<SymbolId>,
    pub(crate) names: Vec<String>,
}

impl Scope {
    pub(crate) fn with_id(my_id: ScopeId) -> Self {
        Self {
            my_id,
            ..Default::default()
        }
    }
    pub(crate) fn make_unique_name(&self, preferred_name: String) -> String {
        generate_name(&self.names, preferred_name)
    }

    pub(crate) fn add_name(&mut self, name: String) -> NameId {
        let name_id = NameId(self.names.len());
        self.names.push(name);
        name_id
    }

    fn insert_type(
        &mut self,
        name: &str,
        symbol_id: SymbolId,
        member_scope: ScopeId,
        entrails: TypeEntrails,
    ) -> Symbol {
        let type_name = match entrails {
            TypeEntrails::Function(_) => self.make_unique_name(NmHlp::to_lower_snake_case(name)),
            _ => self.make_unique_name(NmHlp::to_upper_camel_case(name)),
        };
        let name_id = self.add_name(type_name);
        self.symbols.push(symbol_id.clone());
        Symbol::Type(Type {
            my_id: symbol_id,
            name_id,
            entrails,
            member_scope,
        })
    }

    fn insert_instance(
        &mut self,
        name: &str,
        symbol_id: SymbolId,
        type_id: SymbolId,
        used: bool,
        sem: SymbolAttribute,
    ) -> Symbol {
        let instance_name = self.make_unique_name(NmHlp::to_upper_camel_case(name));
        let name_id = self.add_name(instance_name);
        self.symbols.push(symbol_id.clone());
        Symbol::Instance(Instance {
            name_id,
            type_id,
            used,
            sem,
        })
    }

    pub(crate) fn name(&self, name_id: NameId) -> &str {
        &self.names[name_id.0]
    }

    pub(crate) fn format(&self, symbol_table: &SymbolTable, scope_depth: usize) -> String {
        format!(
            "{}// Scope: my_id: {}, parent: {},\n{}",
            build_indent(scope_depth),
            self.my_id.0,
            self.parent
                .map_or("No parent".to_string(), |i| format!("{}", i.0)),
            self.symbols
                .iter()
                .map(|s| symbol_table.symbol(*s).format(symbol_table, scope_depth))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

///
/// Collection of symbols
///
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct SymbolTable {
    // All symbols, ever created
    pub(crate) symbols: Vec<Symbol>,

    // All scopes
    // The one and only global scope has always index 0
    pub(crate) scopes: Vec<Scope>,
}

impl SymbolTable {
    pub(crate) fn new() -> Self {
        Self {
            symbols: Vec::new(),
            scopes: vec![Scope::default()],
        }
    }

    pub(crate) fn add_symbol(&mut self, symbol: Symbol) -> SymbolId {
        let symbol_id = SymbolId(self.symbols.len());
        self.symbols.push(symbol);
        symbol_id
    }

    pub(crate) fn next_symbol_id(&self) -> SymbolId {
        SymbolId(self.symbols.len())
    }

    pub(crate) fn next_scope_id(&self) -> ScopeId {
        ScopeId(self.scopes.len())
    }

    pub(crate) fn has_lifetime(&self, symbol_id: SymbolId) -> bool {
        self.symbols[symbol_id.0].has_lifetime(&self)
    }

    pub(crate) fn global_scope() -> ScopeId {
        ScopeId(0)
    }

    pub(crate) fn scope(&self, scope_id: ScopeId) -> &Scope {
        &self.scopes[scope_id.0]
    }

    pub(crate) fn scope_mut(&mut self, scope_id: ScopeId) -> &mut Scope {
        &mut self.scopes[scope_id.0]
    }

    pub(crate) fn symbol(&self, symbol_id: SymbolId) -> &Symbol {
        &self.symbols[symbol_id.0]
    }

    pub(crate) fn symbol_mut(&mut self, symbol_id: SymbolId) -> &mut Symbol {
        &mut self.symbols[symbol_id.0]
    }

    pub(crate) fn insert_scope(&mut self, parent: Option<ScopeId>) -> ScopeId {
        let my_id = self.next_scope_id();
        self.scopes.push(Scope {
            parent,
            my_id,
            ..Default::default()
        });
        my_id
    }

    pub(crate) fn insert_symbol(&mut self, symbol: Symbol) -> SymbolId {
        let symbol_id = self.next_symbol_id();
        self.symbols.push(symbol);
        symbol_id
    }

    pub(crate) fn insert_type(
        &mut self,
        parent_symbol: SymbolId,
        type_name: &str,
        entrails: TypeEntrails,
    ) -> Result<SymbolId> {
        debug_assert!(parent_symbol.0 < self.symbols.len());
        let symbol_id = self.next_symbol_id();
        let parent_scope = self.scope(self.symbol(parent_symbol).member_scope()?).my_id;
        let member_scope = self.insert_scope(Some(parent_scope));
        let symbol = self
            .scope_mut(self.symbol(parent_symbol).member_scope()?)
            .insert_type(type_name, symbol_id, member_scope, entrails);
        Ok(self.insert_symbol(symbol))
    }

    pub(crate) fn insert_global_type(
        &mut self,
        type_name: &str,
        entrails: TypeEntrails,
    ) -> Result<SymbolId> {
        let symbol_id = self.next_symbol_id();
        let member_scope = self.insert_scope(Some(SymbolTable::global_scope()));
        let symbol = self.scope_mut(Self::global_scope()).insert_type(
            type_name,
            symbol_id,
            member_scope,
            entrails,
        );
        Ok(self.insert_symbol(symbol))
    }

    pub(crate) fn insert_instance(
        &mut self,
        parent_symbol: SymbolId,
        instance_name: &str,
        type_id: SymbolId,
        used: bool,
        sem: SymbolAttribute,
    ) -> Result<SymbolId> {
        debug_assert!(parent_symbol.0 < self.symbols.len());
        let symbol_id = self.next_symbol_id();
        let symbol = self
            .scope_mut(self.symbol(parent_symbol).member_scope()?)
            .insert_instance(instance_name, symbol_id, type_id, used, sem);
        Ok(self.insert_symbol(symbol))
    }
}

impl Display for SymbolTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(f, "{}", self.scope(Self::global_scope()).format(&self, 0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_scope_relations() {
        let mut symbol_table = SymbolTable::new();
        // Global scope should have been added automatically in `new`
        assert_eq!(1, symbol_table.scopes.len());
        // Global scope should have no parent
        assert_eq!(None, symbol_table.scope(SymbolTable::global_scope()).parent);

        let struct_id = symbol_table
            .insert_global_type("StructA", TypeEntrails::Struct)
            .expect("insert_global_type should succeed");
        assert_eq!(0, struct_id.0);

        // Member scope of new struct should have been added in `insert_global_type`
        assert_eq!(2, symbol_table.scopes.len());
        // New scope should have global scope as parent
        assert_eq!(
            Some(SymbolTable::global_scope()),
            symbol_table.scope(ScopeId(1)).parent
        );

        if let Symbol::Type(struct_type) = symbol_table.symbol(struct_id) {
            assert_eq!(0, struct_type.my_id.0);
            assert_eq!(0, struct_type.name_id.0);
            assert_eq!(
                Some(SymbolTable::global_scope()),
                symbol_table.scope(struct_type.member_scope).parent
            );
            assert_eq!(1, struct_type.member_scope.0);
            assert_eq!(0, symbol_table.scope(struct_type.member_scope).names.len());
            assert_eq!(
                "StructA",
                symbol_table
                    .scope(SymbolTable::global_scope())
                    .name(struct_type.name_id)
            );
        } else {
            panic!("StructA should be a type!");
        }

        let fn_id = symbol_table
            .insert_type(
                struct_id,
                "new",
                TypeEntrails::Function(Function::default()),
            )
            .expect("insert_type should succeed");

        if let Symbol::Type(struct_type) = symbol_table.symbol(struct_id) {
            assert_eq!(1, symbol_table.scope(struct_type.member_scope).names.len());
        } else {
            panic!("StructA should be a type!");
        }

        // Member scope of new function should have been added in `insert_type`
        assert_eq!(3, symbol_table.scopes.len());

        if let Symbol::Type(fn_type) = symbol_table.symbol(fn_id) {
            assert_eq!(1, fn_type.my_id.0);
            assert_eq!(0, fn_type.name_id.0);
            assert_eq!(
                Some(ScopeId(1)),
                symbol_table.scope(fn_type.member_scope).parent
            );
            assert_eq!(2, fn_type.member_scope.0);
            assert_eq!(0, symbol_table.scope(fn_type.member_scope).names.len());
            assert_eq!(
                "new",
                symbol_table
                    .scope(symbol_table.scope(fn_type.member_scope).parent.unwrap())
                    .name(fn_type.name_id)
            );
        }
    }
}
