use crate::analysis::lookahead_dfa::ProductionIndex;
use crate::grammar::{ProductionAttribute, SymbolAttribute};
use crate::{generators::NamingHelper as NmHlp, utils::generate_name};
use miette::{bail, miette, Result};

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
    pub(crate) fn format(&self) -> String {
        format!(
            "fn /* NT: {}{} */",
            self.non_terminal,
            match self.sem {
                ProductionAttribute::None => "".to_string(),
                _ => format!(", {}", self.sem),
            }
        )
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
    Token(String),
    /// A type with Box semantic
    Box(SymbolId),
    /// A type name (without Box semantic)
    TypeName,
    /// A struct, i.e. a named collection of (name, type) tuples
    Struct,
    /// Will be generated as enum with given name
    Enum,
    /// Will be generated as Vec<T> where T is the type, similar to TypeRef
    Vec(SymbolId),
    /// A trait, normally the semantic actions trait  generated for the user grammar
    Trait,
    /// A trait function
    Function(Function),
}

impl TypeEntrails {
    fn format(&self, _type_id: SymbolId, symbol_table: &SymbolTable) -> String {
        format!(
            "{}",
            match self {
                TypeEntrails::None => "*TypeError*".to_string(),
                TypeEntrails::Unit => "()".to_string(),
                TypeEntrails::Token(t) => format!("Token<'t> /* {} */", t),
                TypeEntrails::Box(r) =>
                    format!("Box<{}>", symbol_table.symbol(*r).name(symbol_table)),
                TypeEntrails::TypeName => "TypeName".to_string(),
                TypeEntrails::Struct => "struct".to_string(),
                TypeEntrails::Enum => "enum".to_string(),
                TypeEntrails::Vec(r) =>
                    format!("Vec<{}>", symbol_table.symbol(*r).name(symbol_table)),
                TypeEntrails::Trait => "trait".to_string(),
                TypeEntrails::Function(f) => f.format(),
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

    pub(crate) fn name(&self, symbol_table: &SymbolTable) -> String {
        let member_scope = symbol_table.scope(self.member_scope);
        if let Some(parent_scope_id) = member_scope.parent {
            let name = symbol_table.scope(parent_scope_id).name(self.name_id);
            if name == SymbolTable::UNNAMED_TYPE {
                match self.entrails {
                    TypeEntrails::Box(t) | TypeEntrails::Vec(t) => self.entrails.format(t, symbol_table),
                    _ => panic!("Should not happen: expecting Box or Vec!"),
                }
            } else {
                name.to_string()
            }
        } else {
            "<unknown>".to_string()
        }
    }
}

///
/// A typed instance, usually a function argument or a struct member
///
#[derive(Builder, Clone, Debug, Default, PartialEq)]
pub(crate) struct Instance {
    /// The symbol's id in the symbol table
    pub(crate) my_id: SymbolId,

    /// The symbol name's id in the enveloping scope
    pub(crate) name_id: NameId,

    /// The scope where the instance resides
    pub(crate) scope: ScopeId,

    /// The instance's type id in the symbol table
    pub(crate) type_id: SymbolId,

    /// Indicates if the argument is used
    pub(crate) used: bool,

    /// Semantic information
    pub(crate) sem: SymbolAttribute,
}

impl Instance {
    fn format(&self, symbol_table: &SymbolTable, scope_depth: usize) -> String {
        format!(
            "{}{}: {}{}",
            build_indent(scope_depth),
            self.name(symbol_table),
            symbol_table.symbol(self.type_id).name(symbol_table),
            match self.sem {
                SymbolAttribute::None => "".to_string(),
                _ => format!(" /* {} */", self.sem),
            }
        )
    }

    pub(crate) fn name(&self, symbol_table: &SymbolTable) -> String {
        symbol_table.scope(self.scope).name(self.name_id).to_string()
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
                TypeEntrails::Token(_)
                | TypeEntrails::Box(_)
                | TypeEntrails::TypeName
                | TypeEntrails::Vec(_)
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
            Symbol::Instance(i) => i.format(symbol_table, scope_depth),
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

    pub(crate) fn name(&self, symbol_table: &SymbolTable) -> String {
        match self {
            Symbol::Type(t) => t.name(symbol_table),
            Symbol::Instance(i) => i.name(symbol_table),
        }
    }
}
///
/// Scope with symbols inside
///
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Scope {
    pub(crate) parent: Option<ScopeId>,
    pub(crate) my_id: ScopeId,
    pub(crate) symbols: Vec<SymbolId>,
    names: Vec<String>,
}

impl Scope {
    pub(crate) const UNNAMED_TYPE_NAME_ID: NameId = NameId(0);

    pub(crate) fn new(parent: Option<ScopeId>, my_id: ScopeId) -> Self {
        Self {
            parent,
            my_id,
            symbols: Vec::new(),
            names: vec![SymbolTable::UNNAMED_TYPE.to_string()],
        }
    }

    pub(crate) fn make_unique_name(&self, preferred_name: String) -> String {
        if &preferred_name == SymbolTable::UNNAMED_TYPE {
            SymbolTable::UNNAMED_TYPE.to_string()
        } else {
            generate_name(&self.names, preferred_name)
        }
    }

    pub(crate) fn add_name(&mut self, name: String) -> NameId {
        if &name == SymbolTable::UNNAMED_TYPE {
            Self::UNNAMED_TYPE_NAME_ID
        } else {
            let name_id = NameId(self.names.len());
            self.names.push(name);
            name_id
        }
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
        let instance_name = self.make_unique_name(NmHlp::to_lower_snake_case(name));
        let name_id = self.add_name(instance_name);
        self.symbols.push(symbol_id.clone());
        Symbol::Instance(Instance {
            my_id: symbol_id,
            name_id,
            scope: self.my_id,
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
    pub(crate) const GLOBAL_SCOPE: ScopeId = ScopeId(0);
    pub(crate) const UNNAMED_TYPE: &'static str = "$$";

    pub(crate) fn new() -> Self {
        Self {
            symbols: Vec::new(),
            scopes: vec![Scope::new(None, Self::GLOBAL_SCOPE)],
        }
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

    pub(crate) fn symbol_as_type(&self, symbol_id: SymbolId) -> Result<&Type> {
        match &self.symbols[symbol_id.0] {
            Symbol::Type(t) => Ok(t),
            Symbol::Instance(_) => bail!("No type!"),
        }
    }

    fn insert_scope(&mut self, parent: Option<ScopeId>) -> ScopeId {
        let my_id = self.next_scope_id();
        self.scopes.push(Scope::new(parent, my_id));
        my_id
    }

    fn insert_symbol(&mut self, symbol: Symbol) -> SymbolId {
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
        let member_scope = self.insert_scope(Some(SymbolTable::GLOBAL_SCOPE));
        let symbol = self.scope_mut(Self::GLOBAL_SCOPE).insert_type(
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

    pub(crate) fn get_or_create_type(
        &mut self,
        type_name: &str,
        scope: ScopeId,
        entrails: TypeEntrails,
    ) -> Result<SymbolId> {
        if let Some(symbol_id) = self.scope(scope).symbols.iter().find(|symbol_id| {
            if let Ok(type_symbol) = self.symbol_as_type(**symbol_id) {
                if type_symbol.entrails == entrails
                    || matches!(type_symbol.entrails, TypeEntrails::Token(_))
                        && matches!(entrails, TypeEntrails::Token(_))
                {
                    true
                } else {
                    false
                }
            } else {
                false
            }
        }) {
            return Ok(*symbol_id);
        }

        self.insert_global_type(type_name, entrails)
    }
}

impl Display for SymbolTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(f, "{}", self.scope(Self::GLOBAL_SCOPE).format(&self, 0))
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
        assert_eq!(None, symbol_table.scope(SymbolTable::GLOBAL_SCOPE).parent);

        let struct_id = symbol_table
            .insert_global_type("StructA", TypeEntrails::Struct)
            .expect("insert_global_type should succeed");
        assert_eq!(0, struct_id.0);

        // Member scope of new struct should have been added in `insert_global_type`
        assert_eq!(2, symbol_table.scopes.len());
        // New scope should have global scope as parent
        assert_eq!(
            Some(SymbolTable::GLOBAL_SCOPE),
            symbol_table.scope(ScopeId(1)).parent
        );

        if let Symbol::Type(struct_type) = symbol_table.symbol(struct_id) {
            assert_eq!(0, struct_type.my_id.0);
            assert_eq!(1, struct_type.name_id.0);
            assert_eq!(
                Some(SymbolTable::GLOBAL_SCOPE),
                symbol_table.scope(struct_type.member_scope).parent
            );
            assert_eq!(1, struct_type.member_scope.0);
            // UNNAMED_TYPE's pseudo name $$ is already inserted
            assert_eq!(1, symbol_table.scope(struct_type.member_scope).names.len());
            assert_eq!(
                "StructA",
                symbol_table
                    .scope(SymbolTable::GLOBAL_SCOPE)
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
            assert_eq!(2, symbol_table.scope(struct_type.member_scope).names.len());
        } else {
            panic!("StructA should be a type!");
        }

        // Member scope of new function should have been added in `insert_type`
        assert_eq!(3, symbol_table.scopes.len());

        if let Symbol::Type(fn_type) = symbol_table.symbol(fn_id) {
            assert_eq!(1, fn_type.my_id.0);
            assert_eq!(1, fn_type.name_id.0);
            assert_eq!(
                Some(ScopeId(1)),
                symbol_table.scope(fn_type.member_scope).parent
            );
            assert_eq!(2, fn_type.member_scope.0);
            assert_eq!(1, symbol_table.scope(fn_type.member_scope).names.len());
            assert_eq!(
                "new",
                symbol_table
                    .scope(symbol_table.scope(fn_type.member_scope).parent.unwrap())
                    .name(fn_type.name_id)
            );
        }
    }
}
