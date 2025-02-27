use super::grammar_type_generator::GrammarTypeInfo;
use super::symbol_table::{MetaSymbolKind, SymbolId, SymbolTable, TypeEntrails};
use super::symbol_table_facade::{InstanceFacade, SymbolFacade, TypeFacade};
use super::template_data::{
    NonTerminalTypeEnum, NonTerminalTypeStruct, RangeCalculationBuilder,
    UserTraitCallerFunctionDataBuilder, UserTraitDataBuilder, UserTraitFunctionDataBuilder,
    UserTraitFunctionStackPopDataBuilder,
};
use crate::config::{CommonGeneratorConfig, UserTraitGeneratorConfig};
use crate::generators::naming_helper::NamingHelper as NmHlp;
use crate::generators::GrammarConfig;
use crate::grammar::{ProductionAttribute, SymbolAttribute};
use crate::parser::GrammarType;
use crate::{Pr, StrVec};
use anyhow::{anyhow, bail, Result};
use parol_runtime::log::trace;

/// Generator for user trait code
/// The lifetime parameter `'a` refers to the lifetime of the contained references.
#[derive(Debug, Default)]
pub struct UserTraitGenerator<'a> {
    /// Compiled grammar configuration
    grammar_config: &'a GrammarConfig,
}

impl<'a> UserTraitGenerator<'a> {
    /// Creates a new instance of the user trait generator
    pub fn new(grammar_config: &'a GrammarConfig) -> Self {
        Self { grammar_config }
    }

    fn generate_adapter_function_args(
        &self,
        action_id: SymbolId,
        symbol_table: &SymbolTable,
    ) -> Result<String> {
        // We reference the parse_tree argument only if a token is in the argument list
        let lifetime = "<'t>";
        let mut arguments = Vec::new();

        for member_id in symbol_table.members(action_id)? {
            let arg_inst = symbol_table.symbol_as_instance(*member_id);
            arguments.push(format!(
                "{}: &ParseTreeType{}",
                NmHlp::add_unused_indicator(arg_inst.used(), symbol_table.name(arg_inst.my_id())),
                lifetime
            ));
        }

        Ok(arguments.join(", "))
    }

    /// Generates the code that creates the context for the adapter function.
    fn generate_context(&self, code: &mut StrVec) {
        code.push("let context = function_name!();".to_string());
        code.push("trace!(\"{}\", self.trace_item_stack(context));".to_string());
    }

    /// Generates the code that assigns the token to the token argument of the adapter function to a
    /// local variable that is later transformed into the ASTType (in `generate_result_builder`).
    /// If the token argument is a user-defined type, then the code for the conversion into it is
    /// generated.
    fn generate_token_assignments(
        &self,
        code: &mut StrVec,
        action_id: SymbolId,
        symbol_table: &SymbolTable,
    ) -> Result<()> {
        for member_id in symbol_table.members(action_id)? {
            let arg_inst = symbol_table.symbol_as_instance(*member_id);
            let arg_type = symbol_table.symbol_as_type(arg_inst.type_id());
            if matches!(arg_type.entrails(), TypeEntrails::Token) {
                let arg_name = symbol_table.name(arg_inst.my_id());
                code.push(format!("let {} = {}.token()?.clone();", arg_name, arg_name))
            } else if let TypeEntrails::UserDefinedType(MetaSymbolKind::Token, _) =
                arg_type.entrails()
            {
                let arg_name = symbol_table.name(arg_inst.my_id());
                code.push(format!(
                    "let {} = {}.token()?.try_into().map_err(parol_runtime::ParolError::UserError)?;",
                    arg_name, arg_name,
                ))
            }
        }
        Ok(())
    }

    fn generate_stack_pops(
        grammar_type: GrammarType,
        code: &mut StrVec,
        action_id: SymbolId,
        type_info: &GrammarTypeInfo,
    ) -> Result<()> {
        let symbol_table = &type_info.symbol_table;
        let function = symbol_table.symbol_as_function(action_id)?;

        let member_count = symbol_table.members(action_id)?.len();

        for (i, member_id) in symbol_table.members(action_id)?.iter().rev().enumerate() {
            let arg_inst = symbol_table.symbol_as_instance(*member_id);
            let arg_type = symbol_table.symbol_as_type(arg_inst.type_id());
            if matches!(
                *arg_type.entrails(),
                TypeEntrails::Clipped(MetaSymbolKind::NonTerminal(_))
            ) {
                // let arg_name = symbol_table.name(arg_inst.my_id());
                // code.push(format!("// Ignore clipped member '{}'", arg_name));
                code.push("self.pop(context);".to_string());
            } else if !matches!(*arg_type.entrails(), TypeEntrails::Token)
                && !matches!(
                    *arg_type.entrails(),
                    TypeEntrails::UserDefinedType(MetaSymbolKind::Token, _)
                )
                && arg_inst.sem() != SymbolAttribute::Clipped
            {
                let arg_name = symbol_table.name(arg_inst.my_id());
                let stack_pop_data = UserTraitFunctionStackPopDataBuilder::default()
                    .arg_name(arg_name.to_string())
                    .arg_type(arg_type.inner_name())
                    .vec_anchor(
                        arg_inst.sem() == SymbolAttribute::RepetitionAnchor
                            && grammar_type == GrammarType::LLK,
                    )
                    .popped_item_is_mutable(
                        function.sem == ProductionAttribute::AddToCollection
                            && match grammar_type {
                                GrammarType::LLK => i == 0,
                                GrammarType::LALR1 => i == member_count - 1,
                            },
                    )
                    .build()
                    .unwrap();
                // code.push(format!("// Type of popped value is {}", arg_type.my_id()));
                code.push(format!("{}", stack_pop_data));
            }
        }
        Ok(())
    }

    fn generate_push_semantic(
        &self,
        code: &mut StrVec,
        action_id: SymbolId,
        symbol_table: &SymbolTable,
    ) -> Result<()> {
        let function = symbol_table.symbol_as_function(action_id)?;
        let fn_type = symbol_table.symbol_as_type(action_id);
        let fn_name = symbol_table.name(fn_type.my_id()).to_string();

        if function.sem == ProductionAttribute::AddToCollection {
            match self.grammar_config.grammar_type {
                GrammarType::LLK => {
                    let last_arg = symbol_table
                        .members(action_id)?
                        .iter()
                        .last()
                        .ok_or_else(|| anyhow!("There should be at least one argument!"))?;
                    let arg_inst = symbol_table.symbol_as_instance(*last_arg);
                    let arg_name = symbol_table.name(arg_inst.my_id());
                    code.push("// Add an element to the vector".to_string());
                    code.push(format!(" {}.push({}_built);", arg_name, fn_name,));
                }
                GrammarType::LALR1 => {
                    let first_arg = symbol_table
                        .members(action_id)?
                        .iter()
                        .next()
                        .ok_or_else(|| anyhow!("There should be at least one argument!"))?;
                    let arg_inst = symbol_table.symbol_as_instance(*first_arg);
                    let arg_name = symbol_table.name(arg_inst.my_id());
                    code.push("// Add an element to the vector".to_string());
                    code.push(format!(" {}.push({}_built);", arg_name, fn_name,));
                }
            }
        }
        Ok(())
    }

    fn format_builder_call(
        symbol_table: &SymbolTable,
        member_id: &SymbolId,
        sem: ProductionAttribute,
        code: &mut StrVec,
    ) -> Result<()> {
        let arg_inst = symbol_table.symbol_as_instance(*member_id);
        if arg_inst.sem() == SymbolAttribute::Clipped {
            return Ok(());
        }
        let arg_type = symbol_table.symbol_as_type(arg_inst.type_id());
        if !matches!(*arg_type.entrails(), TypeEntrails::Clipped(_)) {
            let arg_name = symbol_table.name(arg_inst.my_id());
            let setter_name = &arg_name;
            let arg_name = if matches!(*arg_type.entrails(), TypeEntrails::Box(_)) &&
                // If the production is AddToCollection then instance semantic must not be RepetitionAnchor
                (sem != ProductionAttribute::AddToCollection || arg_inst.sem() != SymbolAttribute::RepetitionAnchor)
            {
                format!("Box::new({})", arg_name)
            } else if matches!(
                *arg_type.entrails(),
                TypeEntrails::UserDefinedType(MetaSymbolKind::NonTerminal(_), _)
            ) {
                format!(
                    r#"(&{}).try_into().map_err(parol_runtime::ParolError::UserError)?"#,
                    arg_name
                )
            } else if let TypeEntrails::Option(t) = arg_type.entrails() {
                let inner_type = symbol_table.symbol_as_type(*t);
                if let TypeEntrails::Box(_) = inner_type.entrails() {
                    format!("{}.map(Box::new)", arg_name)
                } else {
                    arg_name.to_string()
                }
            } else {
                arg_name.to_string()
            };
            if *setter_name == arg_name {
                // Avoid clippy warning "Redundant field names in struct initialization"
                code.push(format!("    {},", setter_name));
            } else {
                code.push(format!("    {}: {},", setter_name, arg_name));
            }
        }
        Ok(())
    }

    fn generate_result_builder(
        &self,
        code: &mut StrVec,
        action_id: SymbolId,
        type_info: &GrammarTypeInfo,
    ) -> Result<()> {
        let symbol_table = &type_info.symbol_table;
        let function = symbol_table.symbol_as_function(action_id)?;
        let fn_type = symbol_table.symbol_as_type(action_id);
        let fn_name = symbol_table.name(fn_type.my_id()).to_string();
        let fn_out_type = symbol_table.symbol_as_type(
            *type_info
                .production_types
                .get(&function.prod_num)
                .ok_or_else(|| anyhow!("Production output type not accessible!"))?,
        );
        let nt_type = symbol_table.symbol_as_type(
            *type_info
                .non_terminal_types
                .get(&function.non_terminal)
                .ok_or_else(|| anyhow!("Non-terminal type not accessible!"))?,
        );

        if function.sem == ProductionAttribute::CollectionStart {
            code.push(format!("let {}_built = Vec::new();", fn_name));
        } else if function.sem == ProductionAttribute::AddToCollection {
            match self.grammar_config.grammar_type {
                GrammarType::LLK => {
                    code.push(format!("let {}_built = {} {{", fn_name, nt_type.name()));
                    for member_id in symbol_table.members(action_id)?.iter().rev().skip(1) {
                        Self::format_builder_call(symbol_table, member_id, function.sem, code)?;
                    }
                    code.push(r#"};"#.to_string());
                }
                GrammarType::LALR1 => {
                    code.push(format!("let {}_built = {} {{", fn_name, nt_type.name()));
                    for member_id in symbol_table.members(action_id)?.iter().skip(1).rev() {
                        Self::format_builder_call(symbol_table, member_id, function.sem, code)?;
                    }
                    code.push(r#"};"#.to_string());
                }
            }
        } else if function.sem == ProductionAttribute::OptionalSome {
            code.push(format!("let {}_built = {} {{", fn_name, nt_type.name()));
            for member_id in symbol_table.members(action_id)? {
                Self::format_builder_call(symbol_table, member_id, function.sem, code)?;
            }
            code.push(r#"};"#.to_string());
        } else if function.sem == ProductionAttribute::OptionalNone {
            // Don't generate a builder!
        } else {
            let (builder_prefix, fn_type) = if function.alts == 1 {
                (nt_type.name(), &nt_type)
            } else {
                (fn_out_type.name(), &fn_out_type)
            };
            code.push(format!("let {}_built = {} {{", fn_name, builder_prefix));
            for member_id in fn_type.members() {
                Self::format_builder_call(symbol_table, member_id, function.sem, code)?;
            }
            code.push("};".to_string());
            if function.alts > 1 {
                // Type adjustment to the non-terminal enum
                // let list_0 = List::List0(list_0);
                trace!(
                    "members: {:?} of production {} (type {})",
                    symbol_table.members(nt_type.my_id())?,
                    fn_name,
                    fn_out_type.my_id()
                );
                let (enum_variant_name, is_box) = symbol_table
                    .members(nt_type.my_id())?
                    .iter()
                    .find(|variant| {
                        let enum_variant = symbol_table.symbol_as_type(**variant);
                        if let TypeEntrails::EnumVariant(inner_type) = enum_variant.entrails() {
                            *inner_type == fn_out_type.my_id() || {
                                // Check if the enum variant is a Box
                                let inner_type_symbol = symbol_table.symbol_as_type(*inner_type);
                                // trace!("inner_type: {:?}", inner_type_symbol.entrails());
                                if let TypeEntrails::Box(inner_type) = inner_type_symbol.entrails()
                                {
                                    // trace!("boxed type: {:?}", inner_type);
                                    *inner_type == fn_out_type.my_id()
                                } else {
                                    false
                                }
                            }
                        } else {
                            false
                        }
                    })
                    .map(|enum_variant_id| {
                        let enum_variant = symbol_table.symbol_as_type(*enum_variant_id);
                        let is_box = if let TypeEntrails::EnumVariant(inner_type) =
                            enum_variant.entrails()
                        {
                            let inner_type_symbol = symbol_table.symbol_as_type(*inner_type);
                            matches!(inner_type_symbol.entrails(), TypeEntrails::Box(_))
                        } else {
                            false
                        };
                        (
                            symbol_table
                                .symbol(symbol_table.symbol(*enum_variant_id).my_id())
                                .name(),
                            is_box,
                        )
                    })
                    .ok_or_else(|| anyhow!("Enum variant not found {}", fn_name))?;
                if is_box {
                    code.push(format!(
                        "let {}_built = {}::{}(Box::new({}_built));",
                        fn_name,
                        nt_type.name(),
                        enum_variant_name,
                        fn_name
                    ));
                } else {
                    code.push(format!(
                        "let {}_built = {}::{}({}_built);",
                        fn_name,
                        nt_type.name(),
                        enum_variant_name,
                        fn_name
                    ));
                }
            }
        }
        Ok(())
    }

    fn generate_user_action_call(
        &self,
        code: &mut StrVec,
        action_id: SymbolId,
        type_info: &GrammarTypeInfo,
    ) -> Result<()> {
        let symbol_table = &type_info.symbol_table;
        let function = symbol_table.symbol_as_function(action_id)?;
        let fn_type = symbol_table.symbol_as_type(action_id);
        let fn_name = symbol_table.name(fn_type.my_id()).to_string();
        if let Ok(user_action_id) = type_info.get_user_action(&function.non_terminal) {
            let user_action_name = symbol_table.type_name(user_action_id)?;
            code.push("// Calling user action here".to_string());
            code.push(format!(
                "self.user_grammar.{}(&{}_built)?;",
                user_action_name, fn_name
            ));
        }
        Ok(())
    }

    /// Generates the code that pushes the result of the adapter function for a given production as
    /// ASTType onto the stack. The enum variant of the ASTType is the type generated for the
    /// non-terminal on the left-hand side of the production.
    fn generate_stack_push(
        &self,
        code: &mut StrVec,
        action_id: SymbolId,
        type_info: &GrammarTypeInfo,
    ) -> Result<()> {
        let symbol_table = &type_info.symbol_table;
        let function = symbol_table.symbol_as_function(action_id)?;
        let fn_type = symbol_table.symbol_as_type(action_id);
        let fn_name = symbol_table.name(fn_type.my_id()).to_string();
        let fn_out_type = symbol_table.symbol_as_type(
            *type_info
                .production_types
                .get(&function.prod_num)
                .ok_or_else(|| anyhow!("Production output type not accessible!"))?,
        );

        if function.sem == ProductionAttribute::AddToCollection {
            // The output type of the action is the type generated for the action's non-terminal
            // filled with type of the action's last argument (the vector)
            match self.grammar_config.grammar_type {
                GrammarType::LLK => {
                    let last_arg = symbol_table
                        .members(action_id)?
                        .iter()
                        .last()
                        .ok_or_else(|| anyhow!("There should be at least one argument!"))?;
                    let arg_inst = symbol_table.symbol_as_instance(*last_arg);
                    let arg_name = symbol_table.name(arg_inst.my_id());

                    code.push(format!(
                        "self.push(ASTType::{}({}), context);",
                        NmHlp::to_upper_camel_case(&function.non_terminal),
                        arg_name
                    ));
                }
                GrammarType::LALR1 => {
                    let first_arg = symbol_table
                        .members(action_id)?
                        .iter()
                        .next()
                        .ok_or_else(|| anyhow!("There should be at least one argument!"))?;
                    let arg_inst = symbol_table.symbol_as_instance(*first_arg);
                    let arg_name = symbol_table.name(arg_inst.my_id());

                    code.push(format!(
                        "self.push(ASTType::{}({}), context);",
                        NmHlp::to_upper_camel_case(&function.non_terminal),
                        arg_name
                    ));
                }
            }
        } else if function.sem == ProductionAttribute::OptionalNone {
            code.push(format!(
                "self.push(ASTType::{}(None), context);",
                NmHlp::to_upper_camel_case(&function.non_terminal),
            ));
        } else if function.sem == ProductionAttribute::OptionalSome {
            match fn_out_type.entrails() {
                TypeEntrails::Box(inner_type) => {
                    let inner_type_symbol = symbol_table.symbol_as_type(*inner_type);
                    let inner_type_name = symbol_table.name(inner_type_symbol.my_id());
                    code.push(format!(
                        "self.push(ASTType::{}(Some(Box::new({}_built))), context);",
                        NmHlp::to_upper_camel_case(&function.non_terminal),
                        inner_type_name,
                    ));
                }
                _ => {
                    code.push(format!(
                        "self.push(ASTType::{}(Some({}_built)), context);",
                        NmHlp::to_upper_camel_case(&function.non_terminal),
                        fn_name,
                    ));
                }
            }
        } else {
            // The output type of the action is the type generated for the action's non-terminal
            // filled with type kind of the action
            code.push(format!(
                "self.push(ASTType::{}({}_built), context);",
                NmHlp::to_upper_camel_case(&function.non_terminal),
                fn_name,
            ));
        }
        Ok(())
    }

    fn generate_user_action_args(
        non_terminal: &str,
        type_info: &GrammarTypeInfo,
    ) -> Result<String> {
        let user_action = type_info
            .symbol_table
            .symbol_as_type(type_info.get_user_action(non_terminal)?);

        Ok(user_action
            .members()
            .iter()
            .map(|s| {
                let arg_inst = type_info.symbol_table.symbol_as_instance(*s);
                let arg_type = type_info.symbol_table.symbol_as_type(arg_inst.type_id());
                format!(
                    "{}: {}",
                    NmHlp::add_unused_indicator(arg_inst.used(), &arg_inst.name()),
                    arg_type.to_rust(),
                )
            })
            .collect::<Vec<String>>()
            .join(", "))
    }

    fn generate_caller_argument_list(pr: &Pr) -> String {
        let arguments = pr
            .get_r()
            .iter()
            .filter(|s| !s.is_switch())
            .enumerate()
            .map(|(i, _)| format!("&children[{}]", i))
            .collect::<Vec<String>>();
        arguments.join(", ")
    }

    fn format_type(
        type_id: SymbolId,
        symbol_table: &SymbolTable,
        comment: StrVec,
    ) -> Result<Option<String>> {
        let type_symbol = symbol_table.symbol_as_type(type_id);
        let type_name = symbol_table.name(type_symbol.my_id()).to_string();
        let lifetime = symbol_table.lifetime(type_symbol.my_id());
        let default_members = Vec::default();
        let members = symbol_table
            .members(type_symbol.my_id())
            .unwrap_or(&default_members);

        match type_symbol.entrails() {
            TypeEntrails::Struct => {
                let struct_data = NonTerminalTypeStruct {
                    comment,
                    type_name,
                    lifetime,
                    members: members.iter().fold(StrVec::new(4), |mut acc, m| {
                        if symbol_table.symbol_as_instance(*m).sem() != SymbolAttribute::Clipped {
                            acc.push(symbol_table.symbol(*m).to_rust());
                        }
                        acc
                    }),
                };
                Ok(Some(format!("{}", struct_data)))
            }
            TypeEntrails::Enum => {
                let struct_data = NonTerminalTypeEnum {
                    comment,
                    type_name,
                    lifetime,
                    members: members.iter().fold(StrVec::new(4), |mut acc, m| {
                        acc.push(symbol_table.symbol(*m).to_rust());
                        acc
                    }),
                };
                Ok(Some(format!("{}", struct_data)))
            }
            _ => bail!("Unexpected type {:?}!", type_symbol.entrails()),
        }
    }

    fn generate_range_calculation(t: SymbolId, symbol_table: &SymbolTable) -> Result<String> {
        let type_symbol = symbol_table.symbol_as_type(t);
        let type_name = type_symbol.name();
        let lifetime = symbol_table.elided_lifetime(t);
        let mut range_calc = RangeCalculationBuilder::default()
            .type_name(type_name)
            .lifetime(lifetime)
            .build()
            .unwrap();
        range_calc
            .code
            .push(type_symbol.generate_range_calculation()?);
        Ok(format!("{}", range_calc))
    }

    // ---------------------------------------------------
    // Part of the Public API
    // *Changes will affect crate's version according to semver*
    // ---------------------------------------------------
    ///
    /// Generates the file with the user actions trait.
    ///
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

        type_info.symbol_table.propagate_lifetimes();

        let production_output_types = {
            type_info
                .production_types
                .iter()
                .map(|(prod_num, type_id)| {
                    (
                        type_id,
                        type_info
                            .symbol_table
                            .symbol_as_function(*type_info.adapter_actions.get(prod_num).unwrap()),
                    )
                })
                .filter_map(|(t, f)| {
                    if let Ok(f) = f {
                        if f.alts > 1 && f.sem == ProductionAttribute::None {
                            Some((t, f))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .try_fold(StrVec::new(0), |acc, (t, f)| {
                    Self::generate_single_production_output_type(f, t, type_info, acc, config)
                })?
        };

        let non_terminal_types = {
            type_info
                .non_terminal_types
                .iter()
                .try_fold(StrVec::new(0), |acc, (s, t)| {
                    Self::generate_single_non_terminal_type(s, t, type_info, acc, config)
                })?
        };

        let mut ast_type_decl = {
            let mut comment = StrVec::new(0);
            comment.push(String::default());
            comment.push("Deduced ASTType of expanded grammar".to_string());
            comment.push(String::default());
            Self::format_type(type_info.ast_enum_type, &type_info.symbol_table, comment)?.unwrap()
        };

        if config.range() {
            ast_type_decl += &Self::generate_range_calculation(
                type_info.ast_enum_type,
                &type_info.symbol_table,
            )?;
        }

        let trait_functions = type_info
            .adapter_actions
            .iter()
            .try_fold(StrVec::new(0).first_line_no_indent(), |acc, a| {
                self.generate_single_adapter_function(a, type_info, grammar_type, acc)
            })?;

        let user_trait_functions = type_info
            .get_user_actions()
            .iter()
            .try_fold(StrVec::new(0).first_line_no_indent(), |acc, fn_id| {
                Self::generate_single_user_trait_function(type_info, fn_id, acc)
            })?;

        trace!("user_trait_functions:\n{}", user_trait_functions);

        let trait_caller = self
            .grammar_config
            .cfg
            .pr
            .iter()
            .enumerate()
            .try_fold(StrVec::new(12), |acc, (i, p)| {
                Self::generate_single_user_action_call(type_info, i, p, acc)
            })?;
        // $env:RUST_LOG="parol::generators::user_trait_generator=trace"
        trace!("// Type information:");
        trace!("{}", type_info);

        let ast_type_has_lifetime = type_info.symbol_table.has_lifetime(type_info.ast_enum_type);
        let user_trait_data = UserTraitDataBuilder::default()
            .user_type_name(config.user_type_name())
            .range(config.range())
            .user_provided_attributes(config.inner_attributes().iter().fold(
                StrVec::new(0),
                |mut acc, e| {
                    acc.push(e.to_string());
                    acc
                },
            ))
            .production_output_types(production_output_types)
            .non_terminal_types(non_terminal_types)
            .ast_type_decl(ast_type_decl)
            .ast_type_has_lifetime(ast_type_has_lifetime)
            .trait_functions(trait_functions)
            .trait_caller(trait_caller)
            .user_trait_functions(user_trait_functions)
            .build()
            .unwrap();

        Ok(format!("{}", user_trait_data))
    }

    fn generate_single_adapter_function(
        &self,
        a: (&usize, &SymbolId),
        type_info: &GrammarTypeInfo,
        grammar_type: GrammarType,
        mut acc: StrVec,
    ) -> std::result::Result<StrVec, anyhow::Error> {
        let action_id = *a.1;
        let fn_type = type_info.symbol_table.symbol_as_type(action_id);
        let fn_name = type_info.symbol_table.name(fn_type.my_id()).to_string();
        let function = type_info.symbol_table.symbol_as_function(action_id)?;
        let prod_num = function.prod_num;
        let prod_string = function.prod_string;
        let fn_arguments =
            self.generate_adapter_function_args(action_id, &type_info.symbol_table)?;
        let mut code = StrVec::new(8);
        self.generate_context(&mut code);
        self.generate_token_assignments(&mut code, action_id, &type_info.symbol_table)?;
        Self::generate_stack_pops(grammar_type, &mut code, action_id, type_info)?;
        self.generate_result_builder(&mut code, action_id, type_info)?;
        self.generate_push_semantic(&mut code, action_id, &type_info.symbol_table)?;
        self.generate_user_action_call(&mut code, action_id, type_info)?;
        self.generate_stack_push(&mut code, action_id, type_info)?;
        let user_trait_function_data = UserTraitFunctionDataBuilder::default()
            .fn_name(fn_name)
            .prod_num(prod_num)
            .fn_arguments(fn_arguments)
            .prod_string(prod_string)
            .named(true)
            .code(code)
            .inner(true)
            .build()
            .unwrap();
        acc.push(format!("{}", user_trait_function_data));
        Ok(acc)
    }

    // ---------------------------------------------------
    // Part of the Public API
    // *Changes will affect crate's version according to semver*
    // ---------------------------------------------------
    /// Creates a new item
    ///
    #[deprecated(since = "0.26.0", note = "Please use `new` instead")]
    pub fn try_new(grammar_config: &'a GrammarConfig) -> Result<Self> {
        Ok(UserTraitGenerator::new(grammar_config))
    }

    fn generate_single_non_terminal_type<C: CommonGeneratorConfig + UserTraitGeneratorConfig>(
        s: &String,
        t: &SymbolId,
        type_info: &GrammarTypeInfo,
        mut acc: StrVec,
        config: &C,
    ) -> std::result::Result<StrVec, anyhow::Error> {
        let mut comment = StrVec::new(0);
        comment.push(String::default());
        comment.push(format!("Type derived for non-terminal {}", s));
        comment.push(String::default());
        Self::format_type(*t, &type_info.symbol_table, comment)?
            .into_iter()
            .for_each(|s| acc.push(s));
        if config.range() {
            acc.push(Self::generate_range_calculation(
                *t,
                &type_info.symbol_table,
            )?);
        }
        Ok(acc)
    }

    fn generate_single_production_output_type<
        C: CommonGeneratorConfig + UserTraitGeneratorConfig,
    >(
        f: super::symbol_table::Function,
        t: &SymbolId,
        type_info: &GrammarTypeInfo,
        mut acc: StrVec,
        config: &C,
    ) -> std::result::Result<StrVec, anyhow::Error> {
        let mut comment = StrVec::new(0);
        comment.push(String::default());
        comment.push(format!("Type derived for production {}", f.prod_num));
        comment.push(String::default());
        comment.push(format!("`{}`", f.prod_string));
        comment.push(String::default());
        Self::format_type(*t, &type_info.symbol_table, comment)?
            .into_iter()
            .for_each(|s| acc.push(s));
        if config.range() {
            acc.push(Self::generate_range_calculation(
                *t,
                &type_info.symbol_table,
            )?);
        }
        Ok(acc)
    }

    fn generate_single_user_trait_function(
        type_info: &GrammarTypeInfo,
        fn_id: &SymbolId,
        mut acc: StrVec,
    ) -> std::result::Result<StrVec, anyhow::Error> {
        let fn_name = type_info.symbol_table.type_name(*fn_id)?;
        let nt = type_info
            .symbol_table
            .symbol_as_function(*fn_id)?
            .non_terminal;
        let fn_arguments = Self::generate_user_action_args(&nt, type_info)
            .map_err(|e| anyhow!("{e} in {fn_name}"))?;
        let user_trait_function_data = UserTraitFunctionDataBuilder::default()
            .fn_name(fn_name)
            .non_terminal(nt)
            .fn_arguments(fn_arguments)
            .build()
            .unwrap();
        acc.push(format!("{}", user_trait_function_data));
        Ok(acc)
    }

    fn generate_single_user_action_call(
        type_info: &GrammarTypeInfo,
        i: usize,
        p: &Pr,
        mut acc: StrVec,
    ) -> std::result::Result<StrVec, anyhow::Error> {
        let fn_type_id = type_info.adapter_actions.get(&i).unwrap();
        let fn_type = type_info.symbol_table.symbol_as_type(*fn_type_id);
        let fn_name = type_info.symbol_table.name(fn_type.my_id()).to_string();
        let fn_arguments = Self::generate_caller_argument_list(p);
        let user_trait_function_data = UserTraitCallerFunctionDataBuilder::default()
            .fn_name(fn_name)
            .prod_num(i)
            .fn_arguments(fn_arguments)
            .build()
            .unwrap();
        acc.push(format!("{}", user_trait_function_data));
        Ok(acc)
    }
}
