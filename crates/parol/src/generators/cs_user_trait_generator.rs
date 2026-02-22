use super::symbol_table::{SymbolId, SymbolTable, TypeEntrails};
use super::symbol_table_facade::{InstanceFacade, SymbolFacade, TypeFacade};
use crate::GrammarTypeInfo;
use crate::config::{CommonGeneratorConfig, UserTraitGeneratorConfig};
use crate::generators::{GrammarConfig, NamingHelper};
use crate::grammar::{ProductionAttribute, SymbolAttribute};
use crate::parser::GrammarType;
use anyhow::{Result, anyhow};
use std::fmt::Write;

/// Generator for C# user trait code.
pub struct CSUserTraitGenerator<'a> {
    grammar_config: &'a GrammarConfig,
}

impl<'a> CSUserTraitGenerator<'a> {
    /// Creates a new instance of the C# user trait generator.
    pub fn new(grammar_config: &'a GrammarConfig) -> Self {
        Self { grammar_config }
    }

    /// Generates a production-based action name for the given production index.
    ///
    /// Mirrors the Rust naming convention:
    /// - Single production for a non-terminal → `UpperCamelCase(non_terminal_name)`
    /// - Multiple alternatives → `UpperCamelCase(non_terminal_name + "_" + alternation_index)`
    fn action_name(&self, prod_index: usize) -> String {
        let pr = &self.grammar_config.cfg.pr[prod_index];
        let non_terminal = pr.get_n_str();
        let alts = self
            .grammar_config
            .cfg
            .get_alternations_count(prod_index)
            .unwrap_or(1);

        if alts == 1 {
            NamingHelper::to_upper_camel_case(non_terminal)
        } else {
            let rel_idx = self
                .grammar_config
                .cfg
                .get_alternation_index_of_production(prod_index)
                .unwrap_or(0);
            NamingHelper::to_upper_camel_case(&format!("{}_{}", non_terminal, rel_idx))
        }
    }

    fn escape_cs_string(value: &str) -> String {
        value
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\r', "\\r")
            .replace('\n', "\\n")
    }

    fn member_property_name(type_name: &str, raw_member_name: &str) -> String {
        let property_name = NamingHelper::to_upper_camel_case(raw_member_name);
        if property_name == type_name {
            format!("{}Value", property_name)
        } else {
            property_name
        }
    }

    fn enum_variant_record_name(enum_type_name: &str, variant_member_name: &str) -> String {
        format!(
            "{}{}Variant",
            enum_type_name,
            NamingHelper::to_upper_camel_case(variant_member_name)
        )
    }

    fn to_cs_type(type_id: SymbolId, symbol_table: &SymbolTable) -> Result<String> {
        let type_symbol = symbol_table.symbol_as_type(type_id);
        let type_name = symbol_table.name(type_symbol.my_id()).to_string();
        match type_symbol.entrails() {
            TypeEntrails::Token => Ok("Token".to_string()),
            TypeEntrails::Box(inner)
            | TypeEntrails::Ref(inner)
            | TypeEntrails::Surrogate(inner)
            | TypeEntrails::EnumVariant(inner) => Self::to_cs_type(*inner, symbol_table),
            TypeEntrails::Vec(inner) => {
                Ok(format!("List<{}>", Self::to_cs_type(*inner, symbol_table)?))
            }
            TypeEntrails::Option(inner) => Self::to_cs_type(*inner, symbol_table),
            TypeEntrails::UserDefinedType(_, user_defined_type) => {
                let cs_type = user_defined_type
                    .get_module_scoped_name()
                    .replace("::", ".");
                if cs_type.contains('.') {
                    Ok(format!("global::{cs_type}"))
                } else {
                    Ok(cs_type)
                }
            }
            TypeEntrails::Struct | TypeEntrails::Enum | TypeEntrails::Trait => {
                Ok(type_symbol.inner_name())
            }
            TypeEntrails::Function(_) => Ok(type_name),
            TypeEntrails::Clipped(_) | TypeEntrails::None => Ok("object".to_string()),
        }
    }

    fn child_to_value_expr(
        type_id: SymbolId,
        symbol_table: &SymbolTable,
        child_expr: &str,
    ) -> Result<String> {
        let type_symbol = symbol_table.symbol_as_type(type_id);
        match type_symbol.entrails() {
            TypeEntrails::UserDefinedType(..) => Ok(format!(
                "ConvertValue<{}>({})",
                Self::to_cs_type(type_id, symbol_table)?,
                child_expr
            )),
            TypeEntrails::Box(inner)
            | TypeEntrails::Ref(inner)
            | TypeEntrails::Surrogate(inner)
            | TypeEntrails::EnumVariant(inner) => {
                Self::child_to_value_expr(*inner, symbol_table, child_expr)
            }
            _ => Ok(format!(
                "({}){}",
                Self::to_cs_type(type_id, symbol_table)?,
                child_expr
            )),
        }
    }

    fn non_clipped_members(type_id: SymbolId, symbol_table: &SymbolTable) -> Result<Vec<SymbolId>> {
        Ok(symbol_table
            .members(type_id)?
            .iter()
            .filter_map(|m| {
                let member = symbol_table.symbol_as_instance(*m);
                if member.sem() == SymbolAttribute::Clipped {
                    None
                } else {
                    Some(*m)
                }
            })
            .collect::<Vec<_>>())
    }

    fn emit_struct_type(
        source: &mut String,
        type_id: SymbolId,
        symbol_table: &SymbolTable,
    ) -> Result<()> {
        let type_symbol = symbol_table.symbol_as_type(type_id);
        let type_name = type_symbol.inner_name();
        let members = Self::non_clipped_members(type_id, symbol_table)?;

        if members.is_empty() {
            writeln!(source, "    public sealed record {}();", type_name)?;
            return Ok(());
        }

        let mut args = Vec::with_capacity(members.len());
        for member_id in members {
            let member = symbol_table.symbol_as_instance(member_id);
            let member_type = Self::to_cs_type(member.type_id(), symbol_table)?;
            let member_name =
                Self::member_property_name(&type_name, symbol_table.name(member.my_id()));
            args.push(format!("{} {}", member_type, member_name));
        }
        writeln!(
            source,
            "    public sealed record {}({});",
            type_name,
            args.join(", ")
        )?;
        Ok(())
    }

    fn emit_enum_type(
        source: &mut String,
        type_id: SymbolId,
        symbol_table: &SymbolTable,
    ) -> Result<()> {
        let type_symbol = symbol_table.symbol_as_type(type_id);
        let type_name = type_symbol.inner_name();

        writeln!(source, "    public abstract record {};", type_name)?;
        for member in symbol_table.members(type_id)? {
            let variant_type = symbol_table.symbol_as_type(*member);
            if let TypeEntrails::EnumVariant(inner_type) = variant_type.entrails() {
                let variant_record_name =
                    Self::enum_variant_record_name(&type_name, symbol_table.name(*member));
                let inner_cs_type = Self::to_cs_type(*inner_type, symbol_table)?;
                writeln!(
                    source,
                    "    public sealed record {}({} Value) : {};",
                    variant_record_name, inner_cs_type, type_name
                )?;
            }
        }

        Ok(())
    }

    fn emit_type_declaration(
        source: &mut String,
        type_id: SymbolId,
        symbol_table: &SymbolTable,
    ) -> Result<()> {
        let type_symbol = symbol_table.symbol_as_type(type_id);
        match type_symbol.entrails() {
            TypeEntrails::Struct => Self::emit_struct_type(source, type_id, symbol_table),
            TypeEntrails::Enum => Self::emit_enum_type(source, type_id, symbol_table),
            _ => Ok(()),
        }
    }

    fn emit_struct_ctor(
        type_id: SymbolId,
        symbol_table: &SymbolTable,
        start_index_expr: &str,
    ) -> Result<String> {
        let type_symbol = symbol_table.symbol_as_type(type_id);
        if !matches!(type_symbol.entrails(), TypeEntrails::Struct) {
            return Err(anyhow!("Expected struct type for constructor generation"));
        }
        let members = Self::non_clipped_members(type_id, symbol_table)?;
        if members.is_empty() {
            return Ok(format!("new {}()", type_symbol.inner_name()));
        }

        let mut values = Vec::with_capacity(members.len());
        for (member_index, member_id) in members.iter().enumerate() {
            let member = symbol_table.symbol_as_instance(*member_id);
            let child_expr = format!("children[{} + {}]", start_index_expr, member_index);
            values.push(Self::child_to_value_expr(
                member.type_id(),
                symbol_table,
                &child_expr,
            )?);
        }
        Ok(format!(
            "new {}({})",
            type_symbol.inner_name(),
            values.join(", ")
        ))
    }

    fn emit_action_mapping_method(
        &self,
        source: &mut String,
        prod_num: usize,
        type_info: &GrammarTypeInfo,
    ) -> Result<()> {
        let action_id = *type_info
            .adapter_actions
            .get(&prod_num)
            .ok_or_else(|| anyhow!("No adapter action for production {}", prod_num))?;
        let function = type_info.symbol_table.symbol_as_function(action_id)?;
        let non_terminal = self.grammar_config.cfg.pr[prod_num].get_n();
        let nt_type_id = *type_info
            .non_terminal_types
            .get(&non_terminal)
            .ok_or_else(|| anyhow!("Missing non-terminal type for {}", non_terminal))?;
        let nt_cs_type = Self::to_cs_type(nt_type_id, &type_info.symbol_table)?;
        let production_attribute = self.grammar_config.cfg.pr[prod_num].get_attribute();
        let action_name = self.action_name(prod_num);
        let alternatives = self.grammar_config.cfg.matching_productions(&non_terminal);
        let has_empty_alternative = alternatives
            .iter()
            .any(|(_, production)| production.get_r().is_empty());
        let list_shape_fallback = (non_terminal.ends_with("List")
            || non_terminal.ends_with("_list"))
            && has_empty_alternative;
        let list_action_fallback = action_name.ends_with("List0") || action_name.ends_with("List1");
        let is_collection_helper = matches!(
            production_attribute,
            ProductionAttribute::CollectionStart | ProductionAttribute::AddToCollection
        ) || list_shape_fallback
            || list_action_fallback;
        let map_type_id = nt_type_id;
        let map_cs_type = if is_collection_helper {
            format!("List<{}>", nt_cs_type)
        } else {
            Self::to_cs_type(map_type_id, &type_info.symbol_table)?
        };
        let map_method = format!("Map{}", action_name);

        writeln!(
            source,
            "        private static {} {}(object[] children) {{",
            map_cs_type, map_method
        )?;
        writeln!(
            source,
            "            if (children == null) throw new ArgumentNullException(nameof(children));"
        )?;

        if is_collection_helper {
            let item_members = Self::non_clipped_members(nt_type_id, &type_info.symbol_table)?;
            let item_arity = item_members.len();
            let is_empty_production = self.grammar_config.cfg.pr[prod_num].get_r().is_empty();

            if is_empty_production {
                writeln!(
                    source,
                    "            if (children.Length == 0) return new List<{}>();",
                    nt_cs_type
                )?;
            }
            writeln!(
                source,
                "            if (children.Length == 1 && children[0] is List<{}> directValue) return directValue;",
                nt_cs_type
            )?;
            if !is_empty_production {
                writeln!(
                    source,
                    "            if (children.Length == {}) {{",
                    item_arity
                )?;
                writeln!(
                    source,
                    "                var item = {};",
                    Self::emit_struct_ctor(nt_type_id, &type_info.symbol_table, "0")?
                )?;
                writeln!(
                    source,
                    "                return new List<{}> {{ item }};",
                    nt_cs_type
                )?;
                writeln!(source, "            }}")?;

                writeln!(
                    source,
                    "            if (children.Length == {} + 1 && children[{}] is List<{}> previous) {{",
                    item_arity, item_arity, nt_cs_type
                )?;
                writeln!(
                    source,
                    "                var item = {};",
                    Self::emit_struct_ctor(nt_type_id, &type_info.symbol_table, "0")?
                )?;
                writeln!(
                    source,
                    "                var items = new List<{}>();",
                    nt_cs_type
                )?;
                writeln!(source, "                items.Add(item);")?;
                writeln!(source, "                items.AddRange(previous);")?;
                writeln!(source, "                return items;")?;
                writeln!(source, "            }}")?;
            }
        }

        if !is_collection_helper {
            let map_symbol = type_info.symbol_table.symbol_as_type(map_type_id);
            match map_symbol.entrails() {
                TypeEntrails::Vec(inner) => {
                    let inner_type = Self::to_cs_type(*inner, &type_info.symbol_table)?;
                    writeln!(
                        source,
                        "            if (children.Length == 0) return new List<{}>();",
                        inner_type
                    )?;
                    writeln!(
                        source,
                        "            if (children.Length == 1 && children[0] is List<{}> directValue) return directValue;",
                        inner_type
                    )?;
                    writeln!(
                        source,
                        "            var items = new List<{}>();",
                        inner_type
                    )?;
                    writeln!(source, "            foreach (var child in children) {{")?;
                    writeln!(
                        source,
                        "                if (child is List<{}> existing) items.AddRange(existing);",
                        inner_type
                    )?;
                    writeln!(source, "            }}")?;
                    writeln!(source, "            foreach (var child in children) {{")?;
                    writeln!(
                        source,
                        "                if (child is List<{}>) continue;",
                        inner_type
                    )?;
                    writeln!(source, "                try {{")?;
                    writeln!(
                        source,
                        "                    items.Add(ConvertValue<{}>(child));",
                        inner_type
                    )?;
                    writeln!(
                        source,
                        "                }} catch (InvalidCastException) {{ }}"
                    )?;
                    writeln!(source, "            }}")?;
                    writeln!(
                        source,
                        "            if (items.Count > 0 || children.Length == 0) return items;"
                    )?;
                }
                TypeEntrails::Struct => {
                    let members = Self::non_clipped_members(map_type_id, &type_info.symbol_table)?;
                    let nt_name = map_symbol.inner_name();
                    if members.len() == 1 {
                        let member = type_info.symbol_table.symbol_as_instance(members[0]);
                        let member_type = type_info.symbol_table.symbol_as_type(member.type_id());
                        let member_name = Self::member_property_name(
                            &nt_name,
                            type_info.symbol_table.name(member.my_id()),
                        );
                        match (function.sem, member_type.entrails()) {
                            (ProductionAttribute::OptionalNone, _) => {
                                writeln!(
                                    source,
                                    "            if (children.Length == 0) return new {}(default!);",
                                    nt_name
                                )?;
                            }
                            (ProductionAttribute::OptionalSome, _) => {
                                let value_expr = Self::child_to_value_expr(
                                    member.type_id(),
                                    &type_info.symbol_table,
                                    "children[0]",
                                )?;
                                writeln!(
                                    source,
                                    "            if (children.Length == 1) return new {}({});",
                                    nt_name, value_expr
                                )?;
                            }
                            (ProductionAttribute::CollectionStart, TypeEntrails::Vec(inner)) => {
                                let inner_type = Self::to_cs_type(*inner, &type_info.symbol_table)?;
                                let value_expr = Self::child_to_value_expr(
                                    *inner,
                                    &type_info.symbol_table,
                                    "children[0]",
                                )?;
                                writeln!(
                                    source,
                                    "            if (children.Length == 1) return new {}(new List<{}> {{ {} }});",
                                    nt_name, inner_type, value_expr
                                )?;
                            }
                            (ProductionAttribute::AddToCollection, TypeEntrails::Vec(inner)) => {
                                let inner_type = Self::to_cs_type(*inner, &type_info.symbol_table)?;
                                let value_expr = Self::child_to_value_expr(
                                    *inner,
                                    &type_info.symbol_table,
                                    "children[1]",
                                )?;
                                writeln!(
                                    source,
                                    "            if (children.Length == 2 && children[0] is {} previous) {{",
                                    nt_name
                                )?;
                                writeln!(
                                    source,
                                    "                var items = new List<{}>();",
                                    inner_type
                                )?;
                                writeln!(
                                    source,
                                    "                if (previous.{} != null) items.AddRange(previous.{});",
                                    member_name, member_name
                                )?;
                                writeln!(source, "                items.Add({});", value_expr)?;
                                writeln!(source, "                return new {}(items);", nt_name)?;
                                writeln!(source, "            }}")?;
                            }
                            _ => {}
                        }
                    }

                    writeln!(
                        source,
                        "            if (children.Length == {} ) return {};",
                        members.len(),
                        Self::emit_struct_ctor(map_type_id, &type_info.symbol_table, "0")?
                    )?;
                    writeln!(
                        source,
                        "            if (children.Length == 1 && children[0] is {} directValue) return directValue;",
                        nt_name
                    )?;
                }
                TypeEntrails::Enum => {
                    if let Some(prod_type_id) = type_info.production_types.get(&prod_num) {
                        let prod_type_symbol = type_info.symbol_table.symbol_as_type(*prod_type_id);
                        let nt_name = map_symbol.inner_name();
                        let variant_record_name = type_info
                            .symbol_table
                            .members(map_type_id)?
                            .iter()
                            .find_map(|member| {
                                let variant = type_info.symbol_table.symbol_as_type(*member);
                                if let TypeEntrails::EnumVariant(inner) = variant.entrails() {
                                    if *inner == *prod_type_id {
                                        Some(Self::enum_variant_record_name(
                                            &nt_name,
                                            type_info.symbol_table.name(*member),
                                        ))
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            });

                        if let Some(variant_record_name) = variant_record_name {
                            match prod_type_symbol.entrails() {
                                TypeEntrails::Struct => {
                                    let prod_members = Self::non_clipped_members(
                                        *prod_type_id,
                                        &type_info.symbol_table,
                                    )?;
                                    writeln!(
                                        source,
                                        "            if (children.Length == {}) {{",
                                        prod_members.len()
                                    )?;
                                    writeln!(
                                        source,
                                        "                var value = {};",
                                        Self::emit_struct_ctor(
                                            *prod_type_id,
                                            &type_info.symbol_table,
                                            "0"
                                        )?
                                    )?;
                                    writeln!(
                                        source,
                                        "                return new {}(value);",
                                        variant_record_name
                                    )?;
                                    writeln!(source, "            }}")?;
                                }
                                _ => {
                                    let value_expr = Self::child_to_value_expr(
                                        *prod_type_id,
                                        &type_info.symbol_table,
                                        "children[0]",
                                    )?;
                                    writeln!(
                                        source,
                                        "            if (children.Length == 1) return new {}({});",
                                        variant_record_name, value_expr
                                    )?;
                                }
                            }
                        }
                    }
                    writeln!(
                        source,
                        "            if (children.Length == 1 && children[0] is {} directValue) return directValue;",
                        map_cs_type
                    )?;
                }
                _ => {
                    let value_expr = Self::child_to_value_expr(
                        map_type_id,
                        &type_info.symbol_table,
                        "children[0]",
                    )?;
                    writeln!(
                        source,
                        "            if (children.Length == 1) return {};",
                        value_expr
                    )?;
                }
            }
        }

        let production_text =
            Self::escape_cs_string(&format!("{}", self.grammar_config.cfg.pr[prod_num]));
        writeln!(
            source,
            "            throw new InvalidOperationException(\"Unsupported C# mapping for production {} ({})\");",
            prod_num, production_text
        )?;
        writeln!(source, "        }}")?;
        Ok(())
    }

    /// Generates the C# user trait source code.
    pub fn generate_user_trait_source<C: CommonGeneratorConfig + UserTraitGeneratorConfig>(
        &self,
        config: &C,
        grammar_type: GrammarType,
        type_info: &mut GrammarTypeInfo,
    ) -> Result<String> {
        if config.minimize_boxed_types() {
            type_info.minimize_boxed_types();
        }
        type_info.set_grammar_type(grammar_type);
        type_info.build(self.grammar_config)?;

        let mut source = String::new();
        let user_type_name = config.user_type_name();
        let interface_name = format!(
            "I{}Actions",
            NamingHelper::to_upper_camel_case(user_type_name)
        );
        let class_name = format!(
            "{}Actions",
            NamingHelper::to_upper_camel_case(user_type_name)
        );

        writeln!(source, "using System;")?;
        writeln!(source, "using System.Collections.Generic;")?;
        writeln!(source, "using System.Reflection;")?;
        writeln!(source, "using Parol.Runtime;")?;
        writeln!(source, "using Parol.Runtime.Scanner;")?;
        writeln!(source)?;
        writeln!(source, "namespace {} {{", config.module_name())?;

        writeln!(source, "    // Deduced grammar types")?;
        for (non_terminal, type_id) in &type_info.non_terminal_types {
            writeln!(
                source,
                "    // Type derived for non-terminal {}",
                non_terminal
            )?;
            Self::emit_type_declaration(&mut source, *type_id, &type_info.symbol_table)?;
            writeln!(source)?;
        }

        for (prod_num, type_id) in &type_info.production_types {
            let action_id = type_info
                .adapter_actions
                .get(prod_num)
                .ok_or_else(|| anyhow!("Missing adapter action for production {}", prod_num))?;
            let function = type_info.symbol_table.symbol_as_function(*action_id)?;
            if function.alts > 1 && function.sem == ProductionAttribute::None {
                writeln!(source, "    // Type derived for production {}", prod_num)?;
                Self::emit_type_declaration(&mut source, *type_id, &type_info.symbol_table)?;
                writeln!(source)?;
            }
        }

        writeln!(source, "    /// <summary>")?;
        writeln!(
            source,
            "    /// User actions interface for the {} grammar.",
            user_type_name
        )?;
        writeln!(source, "    /// </summary>")?;
        writeln!(
            source,
            "    public interface {} : IUserActions, IProvidesValueConverter {{",
            interface_name
        )?;

        for fn_id in type_info.get_user_actions() {
            let function = type_info.symbol_table.symbol_as_function(fn_id)?;
            let non_terminal = function.non_terminal;
            let nt_type_id = *type_info
                .non_terminal_types
                .get(&non_terminal)
                .ok_or_else(|| anyhow!("Missing non-terminal type for {}", non_terminal))?;
            let nt_cs_type = Self::to_cs_type(nt_type_id, &type_info.symbol_table)?;
            let method_name = format!("On{}", NamingHelper::to_upper_camel_case(&non_terminal));
            writeln!(source, "        void {}({} arg);", method_name, nt_cs_type)?;
            writeln!(source)?;
        }

        for (i, pr) in self.grammar_config.cfg.pr.iter().enumerate() {
            let action_name = self.action_name(i);
            writeln!(source, "        /// <summary>")?;
            writeln!(source, "        /// Semantic action for production {}:", i)?;
            writeln!(source, "        /// {} ", pr)?;
            writeln!(source, "        /// </summary>")?;
            writeln!(source, "        void {}(object[] children);", action_name)?;
            writeln!(source)?;
        }

        writeln!(source, "    }}")?;
        writeln!(source)?;

        // Skeleton implementation
        writeln!(source, "    /// <summary>")?;
        writeln!(
            source,
            "    /// Base class for user actions for the {} grammar.",
            user_type_name
        )?;
        writeln!(source, "    /// </summary>")?;
        writeln!(
            source,
            "    public partial class {} : {} {{",
            class_name, interface_name
        )?;
        writeln!(source, "        /// <inheritdoc/>")?;
        writeln!(
            source,
            "        public virtual object CallSemanticActionForProductionNumber(int productionNumber, object[] children) {{"
        )?;
        writeln!(source, "            switch (productionNumber) {{")?;
        for (i, _) in self.grammar_config.cfg.pr.iter().enumerate() {
            let action_id = *type_info
                .adapter_actions
                .get(&i)
                .ok_or_else(|| anyhow!("Missing adapter action for production {}", i))?;
            let function = type_info.symbol_table.symbol_as_function(action_id)?;
            let non_terminal = function.non_terminal;
            let action_name = self.action_name(i);
            let map_method = format!("Map{}", action_name);
            let typed_method_name = if type_info.get_user_action(&non_terminal).is_ok() {
                Some(format!(
                    "On{}",
                    NamingHelper::to_upper_camel_case(&non_terminal)
                ))
            } else {
                None
            };
            if let Some(typed_method_name) = typed_method_name {
                writeln!(
                    source,
                    "                case {}: {{ var value = {}(children); {}(value); return value; }}",
                    i, map_method, typed_method_name
                )?;
            } else {
                writeln!(
                    source,
                    "                case {}: return {}(children);",
                    i, map_method
                )?;
            }
        }
        writeln!(
            source,
            "                default: throw new ArgumentException($\"Invalid production number {{productionNumber}}\");"
        )?;
        writeln!(source, "            }}")?;
        writeln!(source, "        }}")?;
        writeln!(source)?;
        writeln!(source, "        /// <inheritdoc/>")?;
        writeln!(
            source,
            "        public virtual void OnComment(Token token) {{ }}"
        )?;
        writeln!(source)?;
        writeln!(source, "        /// <inheritdoc/>")?;
        writeln!(
            source,
            "        public virtual IValueConverter ValueConverter {{ get; }} = new GeneratedValueConverter();"
        )?;
        writeln!(source)?;
        writeln!(
            source,
            "        private sealed class GeneratedValueConverter : IValueConverter {{"
        )?;
        writeln!(
            source,
            "            public bool TryConvert(object value, Type targetType, out object? convertedValue) {{"
        )?;
        writeln!(source, "                convertedValue = null;")?;
        writeln!(source, "                if (value == null) return false;")?;
        writeln!(source, "                var sourceType = value.GetType();")?;
        writeln!(
            source,
            "                foreach (var owner in new[] {{ sourceType, targetType }}) {{"
        )?;
        writeln!(
            source,
            "                    foreach (var method in owner.GetMethods(BindingFlags.Public | BindingFlags.Static)) {{"
        )?;
        writeln!(
            source,
            "                        if ((method.Name == \"op_Implicit\" || method.Name == \"op_Explicit\")"
        )?;
        writeln!(
            source,
            "                            && method.ReturnType == targetType) {{"
        )?;
        writeln!(
            source,
            "                            var parameters = method.GetParameters();"
        )?;
        writeln!(
            source,
            "                            if (parameters.Length == 1 && parameters[0].ParameterType.IsAssignableFrom(sourceType)) {{"
        )?;
        writeln!(
            source,
            "                                convertedValue = method.Invoke(null, new[] {{ value }});"
        )?;
        writeln!(source, "                                return true;")?;
        writeln!(source, "                            }}")?;
        writeln!(source, "                        }}")?;
        writeln!(source, "                    }}")?;
        writeln!(source, "                }}")?;
        writeln!(
            source,
            "                var ctor = targetType.GetConstructor(new[] {{ sourceType }});"
        )?;
        writeln!(source, "                if (ctor != null) {{")?;
        writeln!(
            source,
            "                    convertedValue = ctor.Invoke(new[] {{ value }});"
        )?;
        writeln!(source, "                    return true;")?;
        writeln!(source, "                }}")?;
        writeln!(source, "                return false;")?;
        writeln!(source, "            }}")?;
        writeln!(source, "        }}")?;
        writeln!(source)?;
        writeln!(
            source,
            "        private static TTarget ConvertValue<TTarget>(object value) {{"
        )?;
        writeln!(
            source,
            "            return RuntimeValueConverter.Convert<TTarget>(value);"
        )?;
        writeln!(source, "        }}")?;
        writeln!(source)?;

        for fn_id in type_info.get_user_actions() {
            let function = type_info.symbol_table.symbol_as_function(fn_id)?;
            let non_terminal = function.non_terminal;
            let nt_type_id = *type_info
                .non_terminal_types
                .get(&non_terminal)
                .ok_or_else(|| anyhow!("Missing non-terminal type for {}", non_terminal))?;
            let nt_cs_type = Self::to_cs_type(nt_type_id, &type_info.symbol_table)?;
            let method_name = format!("On{}", NamingHelper::to_upper_camel_case(&non_terminal));

            writeln!(source, "        /// <summary>")?;
            writeln!(
                source,
                "        /// User-facing action for non-terminal {}.",
                non_terminal
            )?;
            writeln!(source, "        /// </summary>")?;
            writeln!(
                source,
                "        public virtual void {}({} arg) {{ }}",
                method_name, nt_cs_type
            )?;
            writeln!(source)?;
        }

        for (i, pr) in self.grammar_config.cfg.pr.iter().enumerate() {
            let action_name = self.action_name(i);
            let action_id = *type_info
                .adapter_actions
                .get(&i)
                .ok_or_else(|| anyhow!("Missing adapter action for production {}", i))?;
            let function = type_info.symbol_table.symbol_as_function(action_id)?;
            let non_terminal = function.non_terminal;
            let map_method = format!("Map{}", action_name);
            let typed_method_name = if type_info.get_user_action(&non_terminal).is_ok() {
                Some(format!(
                    "On{}",
                    NamingHelper::to_upper_camel_case(&non_terminal)
                ))
            } else {
                None
            };

            writeln!(source, "        /// <summary>")?;
            writeln!(source, "        /// Semantic action for production {}:", i)?;
            writeln!(source, "        /// {} ", pr)?;
            writeln!(source, "        /// </summary>")?;
            writeln!(
                source,
                "        public virtual void {}(object[] children) {{",
                action_name
            )?;
            writeln!(source, "            var value = {}(children);", map_method)?;
            if let Some(typed_method_name) = typed_method_name {
                writeln!(source, "            {}(value);", typed_method_name)?;
            }
            writeln!(source, "        }}")?;
            writeln!(source)?;
            self.emit_action_mapping_method(&mut source, i, type_info)?;
            writeln!(source)?;
        }

        writeln!(source, "    }}")?;
        writeln!(source, "}}")?;

        Ok(source)
    }
}
