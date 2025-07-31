use crate::grammar::SymbolAttribute;
use crate::utils::str_vec::StrVec;
use anyhow::{Result, bail};

use super::symbol_table::{
    Instance, MetaSymbolKind, ScopeId, ScopedNameId, Symbol, SymbolId, SymbolKind, SymbolTable,
    Type, TypeEntrails,
};

pub(crate) trait SymbolFacade<'a> {
    /// Returns the name of the symbol. If the symbol is unnamed, None returned.
    fn name(&self) -> String;
    fn kind(&self) -> &'a SymbolKind;
    fn to_rust(&self) -> String;
    fn my_id(&self) -> SymbolId;
    fn name_id(&self) -> ScopedNameId;
}

pub(crate) trait InstanceFacade<'a>: SymbolFacade<'a> {
    fn type_id(&self) -> SymbolId;
    fn description(&self) -> &str;
    fn sem(&self) -> SymbolAttribute;
    fn used(&self) -> bool;
}

pub(crate) trait TypeFacade<'a>: SymbolFacade<'a> {
    fn inner_name(&self) -> String;
    fn member_scope(&self) -> ScopeId;
    fn entrails(&self) -> &TypeEntrails;
    fn generate_range_calculation(&self) -> Result<String>;
    fn members(&self) -> &[SymbolId];
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
        if self.symbol.name_id.is_unnamed() {
            "UNNAMED".to_string()
        } else {
            self.symbol_table.name(self.symbol.my_id).to_string()
        }
    }

    fn kind(&self) -> &'a SymbolKind {
        &self.symbol.kind
    }

    fn to_rust(&self) -> String {
        self.symbol.to_rust(self.symbol_table)
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

    fn description(&self) -> &str {
        &self.instance.description
    }

    fn sem(&self) -> SymbolAttribute {
        self.instance.sem
    }

    fn used(&self) -> bool {
        self.instance.entrails.used
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
        let type_symbol = self
            .symbol_item
            .symbol_table
            .symbol_as_type(self.symbol_item.my_id());
        match type_symbol.entrails() {
            TypeEntrails::Box(t)
            | TypeEntrails::Ref(t)
            | TypeEntrails::Surrogate(t)
            | TypeEntrails::EnumVariant(t)
            | TypeEntrails::Vec(t)
            | TypeEntrails::Option(t)
            | TypeEntrails::UserDefinedType(MetaSymbolKind::NonTerminal(t), _) => {
                self.symbol_item.symbol_table.symbol(*t).name()
            }
            TypeEntrails::Clipped(t) => t.to_string(),
            _ => self
                .symbol_item
                .symbol_table
                .name(type_symbol.my_id())
                .to_owned(),
        }
    }

    fn kind(&self) -> &'a SymbolKind {
        self.symbol_item.kind()
    }

    fn to_rust(&self) -> String {
        self.symbol_item.to_rust()
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

    fn generate_range_calculation(&self) -> Result<String> {
        let symbol_table = self.symbol_item.symbol_table;
        match self.entrails() {
            TypeEntrails::Struct => {
                let relevant_symbols = symbol_table
                    .scope(self.member_scope())
                    .symbols
                    .iter()
                    .filter(|s| {
                        let member = symbol_table.symbol_as_instance(**s);
                        member.sem() != SymbolAttribute::Clipped
                    })
                    .cloned()
                    .collect::<Vec<SymbolId>>();

                if relevant_symbols.is_empty() {
                    Ok("        Span::default()".to_string())
                } else {
                    Ok(format!(
                        "{}",
                        relevant_symbols.iter().enumerate().fold(
                            StrVec::new(8),
                            |mut acc, (i, m)| {
                                let addition = if i > 0 { "+ " } else { "" };
                                let member = symbol_table.symbol_as_instance(*m);
                                let member_type = symbol_table.symbol_as_type(member.type_id());
                                match member_type.entrails() {
                                    TypeEntrails::Vec(_) => {
                                        acc.push(format!(
                                            "{}self.{}.first().map_or(Span::default(), |f| f.span())",
                                            addition,
                                            member.name()
                                        ));
                                        acc.push(format!(
                                            "+ self.{}.last().map_or(Span::default(), |l| l.span())",
                                            member.name()
                                        ));
                                    }
                                    TypeEntrails::Option(_) => acc.push(format!(
                                        "{}self.{}.as_ref().map_or(Span::default(), |o| o.span())",
                                        addition,
                                            member.name()
                                    )),
                                    _ => acc.push(format!(
                                        "{}self.{}.span()",
                                        addition,
                                            member.name()
                                    )),
                                }
                                acc
                            }
                        )
                    ))
                }
            }
            TypeEntrails::Enum => {
                let mut enum_data = EnumRangeCalcBuilder::default().build().unwrap();
                enum_data.enum_variants = self
                    .symbol_item
                    .symbol_table
                    .scope(self.member_scope())
                    .symbols
                    .iter()
                    .fold(StrVec::new(8), |mut acc, v| {
                        let v = self.symbol_item.symbol_table.symbol_as_type(*v);
                        if let TypeEntrails::EnumVariant(a) = v.entrails() {
                            let enum_variant_type = self.symbol_item.symbol_table.symbol_as_type(*a);
                            match enum_variant_type.entrails() {
                                TypeEntrails::Vec(_) => {
                                    acc.push(format!(
                                        "{}::{}(v) => v.first().map_or(Span::default(), |f| f.span())",
                                        self.name(),
                                        v.inner_name()
                                    ));
                                    acc.push(
                                        "+ v.last().map_or(Span::default(), |l| l.span()),".to_string(),
                                    );
                                }
                                TypeEntrails::Option(_) => acc.push(format!(
                                    "{}::{}(o) => o.as_ref().map_or(Span::default(), |o| o.span()),",
                                    self.name(),
                                    v.inner_name()
                                )),
                                _ => {
                                    // Expr::CommentExpr(v) => v.span(),
                                    acc.push(format!("{}::{}(v) => v.span(),", self.name(), v.inner_name()))
                                }
                            }
                        } else {
                            panic!("Expecting enum variant here!")
                        }
                        acc
                    });
                Ok(format!("{enum_data}"))
            }
            _ => bail!("Unexpected type for range calculation!"),
        }
    }

    fn members(&self) -> &[SymbolId] {
        &self
            .symbol_item
            .symbol_table
            .scope(self.member_scope())
            .symbols
    }
}

#[derive(Builder, Debug, Default)]
pub(crate) struct EnumRangeCalc {
    #[builder(default)]
    pub enum_variants: StrVec,
}

impl std::fmt::Display for EnumRangeCalc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let EnumRangeCalc { enum_variants } = self;
        f.write_fmt(ume::ume! {
            match self {
                #enum_variants
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_range_calculation() {
        let mut symbol_table = SymbolTable::new();
        let enum_type_id = symbol_table
            .insert_global_type("MyEnum", TypeEntrails::Enum)
            .unwrap();
        let enum_variant1_base = symbol_table
            .insert_global_type("VariantABase", TypeEntrails::Struct)
            .unwrap();
        let enum_variant2_base = symbol_table
            .insert_global_type("VariantBBase", TypeEntrails::Struct)
            .unwrap();

        // Add enum variants
        let _variant_a_id = symbol_table.insert_type(
            enum_type_id,
            "VariantA",
            TypeEntrails::EnumVariant(enum_variant1_base),
        );

        let _variant_b_id = symbol_table.insert_type(
            enum_type_id,
            "VariantB",
            TypeEntrails::EnumVariant(enum_variant2_base),
        );

        // Call the generation of range calculation code on the enum type facade
        let enum_type = symbol_table.symbol_as_type(enum_type_id);
        assert_eq!(
            enum_type.generate_range_calculation().unwrap(),
            "match self {         MyEnum::VariantA(v) => v.span(),\n        MyEnum::VariantB(v) => v.span(),\n }"
        );
    }
}
