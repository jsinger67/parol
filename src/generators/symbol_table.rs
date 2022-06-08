//! The module symbol_table provides means to mimic the uniqueness of names per scope.
//! For auto-generation of symbols we need to adhere these rules of uniqueness.
use crate::analysis::lookahead_dfa::ProductionIndex;
use crate::grammar::{ProductionAttribute, SymbolAttribute};
use crate::{generators::NamingHelper as NmHlp, utils::generate_name};
use log::trace;
use miette::{bail, miette, Result};

use std::fmt::{Debug, Display, Error, Formatter};

/// Index type for Symbols
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct SymbolId(usize);

impl Display for SymbolId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(f, "Sym({})", self.0)
    }
}

/// Scope local index type for SymbolNames
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct ScopedNameId(ScopeId, usize);

impl Display for ScopedNameId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(f, "Nam({}, {})", self.0, self.1)
    }
}

/// Index type for SymbolNames
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct ScopeId(usize);

impl Display for ScopeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(f, "Sco({})", self.0)
    }
}

fn build_indent(amount: usize) -> String {
    const SPACES_PER_TAB: usize = 4;
    let space = " ".to_string();
    space.repeat(amount * SPACES_PER_TAB)
}

///
/// Type specificities of a function type
///
#[derive(Builder, Clone, Debug, Default, PartialEq)]
pub(crate) struct Function {
    /// Associated non-terminal
    pub(crate) non_terminal: String,

    /// Semantic specification
    #[builder(default)]
    pub(crate) sem: ProductionAttribute,

    /// Production number
    #[builder(default)]
    pub(crate) prod_num: ProductionIndex,

    /// The relative index of a production within its alternatives.
    #[builder(default)]
    pub(crate) rel_idx: usize,

    /// Number of alternatives, the number of productions that exist in the grammar which have the
    /// same non-terminal
    #[builder(default)]
    pub(crate) alts: usize,

    /// Formatted production in PAR syntax.
    #[builder(default)]
    pub(crate) prod_string: String,
}

impl Function {
    pub(crate) fn format(&self, fn_name: String) -> String {
        format!(
            "fn {} /* NT: {}{} Prod: {}, Rel: {}, Alts: {} */",
            fn_name,
            self.non_terminal,
            match self.sem {
                ProductionAttribute::None => "".to_string(),
                _ => format!(", {}", self.sem),
            },
            self.prod_num,
            self.rel_idx,
            self.alts,
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum SymbolKind {
    Token,
    NonTerminal,
}

impl Display for SymbolKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match self {
            SymbolKind::Token => write!(f, "Tok"),
            SymbolKind::NonTerminal => write!(f, "Nt"),
        }
    }
}

///
/// Type information used for auto-generation
///
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum TypeEntrails {
    /// Not specified, used as prototype during generation
    None,
    // Unit type ()
    //Unit,
    /// Will be generated as Token structure
    Token,
    /// A type with Box semantic
    Box(SymbolId),
    // A type name (without Box semantic)
    //TypeName,
    /// A struct, i.e. a named collection of (name, type) tuples
    Struct,
    /// Will be generated as enum with given name
    Enum,
    /// A variant of an enum with a type
    EnumVariant(SymbolId),
    /// Will be generated as Vec<T> where T is the type, similar to TypeRef
    Vec(SymbolId),
    /// A trait, normally the semantic actions trait  generated for the user grammar
    Trait,
    /// A trait function
    Function(Function),
    /// An Option type
    Option(SymbolId),
    /// An invisible type
    Clipped(SymbolKind),
}

impl TypeEntrails {
    fn format(&self, type_id: SymbolId, symbol_table: &SymbolTable) -> String {
        let uses_type_name = || {
            matches!(self, Self::Struct)
                | matches!(self, Self::Enum)
                | matches!(self, Self::EnumVariant(_))
                | matches!(self, Self::Function(_))
                | matches!(self, Self::Trait)
        };
        let my_type_name = if uses_type_name() {
            let my_type = symbol_table.symbol_as_type(type_id).unwrap();
            my_type.name(symbol_table)
        } else {
            String::default()
        };
        let lifetime = symbol_table.lifetime(type_id);
        match self {
            TypeEntrails::None => "*TypeError*".to_string(),
            TypeEntrails::Token => format!("Token{}", lifetime),
            TypeEntrails::Box(r) => format!(
                "Box<{}{}>",
                symbol_table.symbol(*r).name(symbol_table),
                symbol_table.lifetime(*r)
            ),
            TypeEntrails::Struct => format!("struct {}{}", my_type_name, lifetime),
            TypeEntrails::Enum => format!("enum {}{}", my_type_name, lifetime),
            TypeEntrails::EnumVariant(t) => format!(
                "{}({}{}),",
                my_type_name,
                symbol_table.symbol(*t).name(symbol_table),
                symbol_table.lifetime(*t)
            ),
            TypeEntrails::Vec(r) => format!(
                "Vec<{}{}>",
                symbol_table.symbol(*r).name(symbol_table),
                symbol_table.lifetime(*r)
            ),
            TypeEntrails::Trait => format!("trait {}{}", my_type_name, lifetime),
            TypeEntrails::Function(f) => f.format(my_type_name),
            TypeEntrails::Option(o) => format!(
                "Option<Box<{}{}>>",
                symbol_table.symbol(*o).name(symbol_table),
                symbol_table.lifetime(*o)
            ),
            TypeEntrails::Clipped(k) => format!("Clipped({})", k),
        }
    }

    pub(crate) fn inner_name(&self, symbol_table: &SymbolTable) -> Result<String> {
        match self {
            TypeEntrails::Box(t) | TypeEntrails::Vec(t) | TypeEntrails::Option(t) => {
                Ok(symbol_table.symbol(*t).name(symbol_table))
            }
            _ => bail!("No inner name available!"),
        }
    }

    fn to_rust(&self, type_id: SymbolId, symbol_table: &SymbolTable) -> String {
        self.format(type_id, symbol_table)
    }

    pub(crate) fn sem(&self) -> SymbolAttribute {
        SymbolAttribute::None
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
    pub(crate) name_id: ScopedNameId,

    /// The type specificities
    pub(crate) entrails: TypeEntrails,

    /// The inner scope
    pub(crate) member_scope: ScopeId,
}

impl Type {
    fn format(&self, symbol_table: &SymbolTable, scope_depth: usize) -> String {
        let scope = if !matches!(self.entrails, TypeEntrails::EnumVariant(_)) {
            format!(
                " {{\n{}\n{}}}",
                symbol_table
                    .scope(self.member_scope)
                    .format(symbol_table, scope_depth + 1),
                build_indent(scope_depth),
            )
        } else {
            ",".to_string()
        };
        format!(
            "{}{} /* Type: my_id {}, name_id: {} */ {}",
            build_indent(scope_depth),
            self.entrails.format(self.my_id, symbol_table),
            self.my_id,
            self.name_id,
            scope,
        )
    }

    pub(crate) fn to_rust(&self, symbol_table: &SymbolTable) -> String {
        self.entrails.to_rust(self.my_id, symbol_table)
    }

    pub(crate) fn name(&self, symbol_table: &SymbolTable) -> String {
        if self.name_id.1 == Scope::UNNAMED_TYPE_NAME_ID {
            self.entrails.format(self.my_id, symbol_table)
        } else {
            symbol_table.name(self.name_id).to_string()
        }
    }

    pub(crate) fn inner_name(&self, symbol_table: &SymbolTable) -> Result<String> {
        if self.name_id.1 == Scope::UNNAMED_TYPE_NAME_ID {
            self.entrails.inner_name(symbol_table)
        } else {
            Ok(symbol_table.name(self.name_id).to_string())
        }
    }

    // Used to suppress lifetime on the container types
    pub(crate) fn is_container(&self) -> bool {
        matches!(
            self.entrails,
            TypeEntrails::Box(_) | TypeEntrails::Vec(_) | TypeEntrails::Option(_)
        )
    }

    pub(crate) fn sem(&self) -> SymbolAttribute {
        self.entrails.sem()
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
    pub(crate) name_id: ScopedNameId,

    /// The scope where the instance resides
    pub(crate) scope: ScopeId,

    /// The instance's type id in the symbol table
    pub(crate) type_id: SymbolId,

    /// Indicates if the argument is used
    pub(crate) used: bool,

    /// Semantic information
    pub(crate) sem: SymbolAttribute,

    /// Description
    pub(crate) description: String,
}

impl Instance {
    fn format(&self, symbol_table: &SymbolTable, scope_depth: usize) -> String {
        let desc = if self.description.is_empty() {
            String::default()
        } else {
            format!(" /* {} */", self.description)
        };
        format!(
            "{}{}: {}{}{}{}",
            build_indent(scope_depth),
            self.name(symbol_table),
            symbol_table.symbol(self.type_id).name(symbol_table),
            symbol_table.lifetime(self.type_id),
            desc,
            match self.sem {
                SymbolAttribute::None => "".to_string(),
                _ => format!(" /* {} */", self.sem),
            }
        )
    }

    fn to_rust(&self, symbol_table: &SymbolTable) -> String {
        let desc = if self.description.is_empty() {
            String::default()
        } else {
            format!("/* {} */", self.description)
        };
        format!(
            "{}: {}{},",
            self.name(symbol_table),
            symbol_table.symbol(self.type_id).to_rust(symbol_table),
            desc,
        )
    }

    pub(crate) fn name(&self, symbol_table: &SymbolTable) -> String {
        symbol_table.name(self.name_id).to_string()
    }

    pub(crate) fn sem(&self, symbol_table: &SymbolTable) -> SymbolAttribute {
        if self.sem != SymbolAttribute::None {
            self.sem
        } else if let Ok(type_symbol) = symbol_table.symbol_as_type(self.type_id) {
            type_symbol.sem()
        } else {
            SymbolAttribute::None
        }
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
    #[named]
    fn has_lifetime(&self, symbol_table: &SymbolTable) -> bool {
        match self {
            Self::Type(t) => {
                trace!("{}: {}", function_name!(), self.my_id());
                match t.entrails {
                    TypeEntrails::None | TypeEntrails::Function(_) | TypeEntrails::Clipped(_) => {
                        trace!("{}: => false", function_name!());
                        false
                    }
                    TypeEntrails::Token => {
                        trace!("{}: => true", function_name!());
                        true
                    }
                    TypeEntrails::Struct | TypeEntrails::Enum | TypeEntrails::Trait => symbol_table
                        .scope(t.member_scope)
                        .symbols
                        .iter()
                        .any(|e| symbol_table.has_lifetime(*e)),
                    TypeEntrails::Vec(t)
                    | TypeEntrails::EnumVariant(t)
                    | TypeEntrails::Box(t)
                    | TypeEntrails::Option(t) => symbol_table.has_lifetime(t),
                }
            }
            Self::Instance(i) => {
                i.sem != SymbolAttribute::Clipped && symbol_table.has_lifetime(i.type_id)
            }
        }
    }

    pub(crate) fn my_id(&self) -> SymbolId {
        match self {
            Symbol::Type(t) => t.my_id,
            Symbol::Instance(i) => i.my_id,
        }
    }

    pub(crate) fn is_container(&self, symbol_table: &SymbolTable) -> bool {
        match self {
            Symbol::Type(me) => {
                if let Ok(me) = symbol_table.symbol_as_type(me.my_id) {
                    me.is_container()
                } else {
                    false
                }
            }
            Symbol::Instance(_) => false,
        }
    }

    pub(crate) fn lifetime(&self, symbol_table: &SymbolTable) -> String {
        if self.has_lifetime(symbol_table) && !self.is_container(symbol_table) {
            "<'t>".to_string()
        } else {
            "".to_string()
        }
    }

    pub(crate) fn imposes_lifetime(&self, symbol_table: &SymbolTable) -> String {
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

    pub(crate) fn to_rust(&self, symbol_table: &SymbolTable) -> String {
        match self {
            Symbol::Type(t) => t.to_rust(symbol_table),
            Symbol::Instance(i) => i.to_rust(symbol_table),
        }
    }

    pub(crate) fn member_scope(&self) -> Result<ScopeId> {
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

    pub(crate) fn _sem(&self, symbol_table: &SymbolTable) -> SymbolAttribute {
        match self {
            Symbol::Type(t) => t.sem(),
            Symbol::Instance(i) => i.sem(symbol_table),
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
    pub(crate) const UNNAMED_TYPE_NAME_ID: usize = 0;

    pub(crate) fn new(parent: Option<ScopeId>, my_id: ScopeId) -> Self {
        Self {
            parent,
            my_id,
            symbols: Vec::new(),
            names: vec![SymbolTable::UNNAMED_TYPE.to_string()],
        }
    }

    pub(crate) fn make_unique_name(&self, preferred_name: String) -> String {
        if preferred_name == SymbolTable::UNNAMED_TYPE {
            SymbolTable::UNNAMED_TYPE.to_string()
        } else {
            generate_name(&self.names, preferred_name)
        }
    }

    pub(crate) fn add_name(&mut self, name: String) -> ScopedNameId {
        if name == SymbolTable::UNNAMED_TYPE {
            ScopedNameId(self.my_id, Self::UNNAMED_TYPE_NAME_ID)
        } else {
            let name_id = ScopedNameId(self.my_id, self.names.len());
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
        self.symbols.push(symbol_id);
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
        description: String,
    ) -> Symbol {
        let instance_name = self.make_unique_name(NmHlp::to_lower_snake_case(name));
        let name_id = self.add_name(instance_name);
        self.symbols.push(symbol_id);
        Symbol::Instance(Instance {
            my_id: symbol_id,
            name_id,
            scope: self.my_id,
            type_id,
            used,
            sem,
            description,
        })
    }

    pub(crate) fn format(&self, symbol_table: &SymbolTable, scope_depth: usize) -> String {
        format!(
            "{}// Scope: my_id: {}, parent: {}\n{}",
            build_indent(scope_depth),
            self.my_id,
            self.parent
                .map_or("No parent".to_string(), |i| format!("{}", i)),
            self.symbols
                .iter()
                .map(|s| symbol_table.symbol(*s).format(symbol_table, scope_depth))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

impl Display for Scope {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(
            f,
            "// Scope: my_id: {}, parent: {}\n//   Symbols:\n{}\n//   Names:\n{}",
            self.my_id,
            self.parent
                .map_or("No parent".to_string(), |i| format!("{}", i)),
            self.symbols
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
                .join("\n"),
            self.names
                .iter()
                .enumerate()
                .map(|(i, n)| format!("{}: {}", ScopedNameId(self.my_id, i), n))
                .collect::<Vec<String>>()
                .join("\n"),
        )
    }
}

///
/// Collection of symbols
///
/// Mimics rust's rules of uniqueness of symbol names within a certain scope.
/// This struct models the scope and symbols within them only to the extend needed to auto-generate
/// flawless type and instance names.
/// Especially the deduction of the existence of lifetime parameter on generated types is modelled
/// as simple as possible.
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

    #[named]
    pub(crate) fn has_lifetime(&self, symbol_id: SymbolId) -> bool {
        trace!("{}: {}", function_name!(), symbol_id);
        self.symbols[symbol_id.0].has_lifetime(self)
    }

    pub(crate) fn lifetime(&self, symbol_id: SymbolId) -> String {
        self.symbols[symbol_id.0].lifetime(self)
    }

    pub(crate) fn imposes_lifetime(&self, symbol_id: SymbolId) -> String {
        self.symbols[symbol_id.0].imposes_lifetime(self)
    }

    pub(crate) fn name(&self, name_id: ScopedNameId) -> &str {
        &self.scope(name_id.0).names[name_id.1]
    }

    pub(crate) fn members(&self, type_id: SymbolId) -> Result<&Vec<SymbolId>> {
        let type_symbol = self.symbol_as_type(type_id)?;
        Ok(&self.scope(type_symbol.member_scope).symbols)
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

    pub(crate) fn type_name(&self, symbol_id: SymbolId) -> Result<&str> {
        let type_symbol = self.symbol_as_type(symbol_id)?;
        Ok(self.name(type_symbol.name_id))
    }

    pub(crate) fn _instance_name(&self, symbol_id: SymbolId) -> Result<&str> {
        let instance_symbol = self.symbol_as_instance(symbol_id)?;
        Ok(self.name(instance_symbol.name_id))
    }

    pub(crate) fn symbol_as_instance(&self, symbol_id: SymbolId) -> Result<&Instance> {
        match &self.symbols[symbol_id.0] {
            Symbol::Type(_) => bail!("Ain't no instance!"),
            Symbol::Instance(i) => Ok(i),
        }
    }

    pub(crate) fn symbol_as_instance_mut(&mut self, symbol_id: SymbolId) -> Result<&mut Instance> {
        match &mut self.symbols[symbol_id.0] {
            Symbol::Type(_) => bail!("Ain't no instance!"),
            Symbol::Instance(i) => Ok(i),
        }
    }

    pub(crate) fn symbol_as_type(&self, symbol_id: SymbolId) -> Result<&Type> {
        match &self.symbols[symbol_id.0] {
            Symbol::Type(t) => Ok(t),
            Symbol::Instance(_) => bail!("Ain't no type!"),
        }
    }

    pub(crate) fn symbol_as_function(&self, symbol_id: SymbolId) -> Result<&Function> {
        let function_type = self.symbol_as_type(symbol_id)?;
        match &function_type.entrails {
            TypeEntrails::Function(f) => Ok(f),
            _ => bail!("Expecting a function here"),
        }
    }

    pub(crate) fn function_type_semantic(
        &self,
        symbol_id: SymbolId,
    ) -> Result<ProductionAttribute> {
        let function_type = self.symbol_as_type(symbol_id)?;
        match &function_type.entrails {
            TypeEntrails::Function(f) => Ok(f.sem),
            _ => bail!("Expecting a function here"),
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
        let symbol =
            self.scope_mut(parent_scope)
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
        description: String,
    ) -> Result<SymbolId> {
        debug_assert!(parent_symbol.0 < self.symbols.len());
        let symbol_id = self.next_symbol_id();
        let member_scope = self.symbol(parent_symbol).member_scope()?;
        let symbol = self.scope_mut(member_scope).insert_instance(
            instance_name,
            symbol_id,
            type_id,
            used,
            sem,
            description,
        );
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
                type_symbol.entrails == entrails
                    || matches!(type_symbol.entrails, TypeEntrails::Token)
                        && matches!(entrails, TypeEntrails::Token)
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
        writeln!(f, "// Symbols:")?;
        for (i, sym) in self.symbols.iter().enumerate() {
            writeln!(f, "Sym({}): {}", i, sym.format(&self, 0))?;
        }
        writeln!(f, "// Scopes:")?;
        for scope in &self.scopes {
            writeln!(f, "{}", scope)?;
        }
        writeln!(f, "// Scope hierarchy:")?;
        write!(f, "{}", self.scope(Self::GLOBAL_SCOPE).format(self, 0))
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
            assert_eq!(1, struct_type.name_id.1);
            assert_eq!(
                Some(SymbolTable::GLOBAL_SCOPE),
                symbol_table.scope(struct_type.member_scope).parent
            );
            assert_eq!(1, struct_type.member_scope.0);
            // UNNAMED_TYPE's pseudo name $$ is already inserted
            assert_eq!(1, symbol_table.scope(struct_type.member_scope).names.len());
            assert_eq!("StructA", symbol_table.name(struct_type.name_id));
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
            assert_eq!(1, fn_type.name_id.1);
            assert_eq!(
                Some(ScopeId(1)),
                symbol_table.scope(fn_type.member_scope).parent
            );
            assert_eq!(2, fn_type.member_scope.0);
            assert_eq!(1, symbol_table.scope(fn_type.member_scope).names.len());
            assert_eq!("new", symbol_table.name(fn_type.name_id));
        }
    }
}
