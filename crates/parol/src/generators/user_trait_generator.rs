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

    fn generate_inner_action_args<C: CommonGeneratorConfig + UserTraitGeneratorConfig>(
        &self,
        config: &C,
        action_id: SymbolId,
        symbol_table: &SymbolTable,
    ) -> Result<String> {
        // We reference the parse_tree argument only if a token is in the argument list
        let lifetime = if config.auto_generate() { "<'t>" } else { "" };
        let mut arguments = Vec::new();

        for member_id in symbol_table.members(action_id)? {
            let arg_inst = symbol_table.symbol_as_instance(*member_id);
            arguments.push(format!(
                "{}: &ParseTreeType{}",
                NmHlp::add_unused_indicator(arg_inst.used(), symbol_table.name(arg_inst.name_id())),
                lifetime
            ));
        }

        Ok(arguments.join(", "))
    }

    fn generate_context<C: CommonGeneratorConfig + UserTraitGeneratorConfig>(
        &self,
        config: &C,
        code: &mut StrVec,
    ) {
        if config.auto_generate() {
            code.push("let context = function_name!();".to_string());
            code.push("trace!(\"{}\", self.trace_item_stack(context));".to_string());
        }
    }

    fn generate_token_assignments<C: CommonGeneratorConfig + UserTraitGeneratorConfig>(
        &self,
        config: &C,
        code: &mut StrVec,
        action_id: SymbolId,
        symbol_table: &SymbolTable,
    ) -> Result<()> {
        if !config.auto_generate() {
            return Ok(());
        }

        for member_id in symbol_table.members(action_id)? {
            let arg_inst = symbol_table.symbol_as_instance(*member_id);
            let arg_type = symbol_table.symbol_as_type(arg_inst.type_id());
            if matches!(arg_type.entrails(), TypeEntrails::Token) {
                let arg_name = symbol_table.name(arg_inst.name_id());
                code.push(format!("let {} = {}.token()?.clone();", arg_name, arg_name))
            } else if let TypeEntrails::UserDefinedType(MetaSymbolKind::Token, _) =
                arg_type.entrails()
            {
                let arg_name = symbol_table.name(arg_inst.name_id());
                code.push(format!(
                    "let {} = {}.token()?.try_into().map_err(parol_runtime::ParolError::UserError)?;",
                    arg_name, arg_name,
                ))
            }
        }
        Ok(())
    }

    fn generate_stack_pops<C: CommonGeneratorConfig + UserTraitGeneratorConfig>(
        &self,
        config: &C,
        code: &mut StrVec,
        action_id: SymbolId,
        symbol_table: &SymbolTable,
    ) -> Result<()> {
        if !config.auto_generate() {
            return Ok(());
        }

        let function = symbol_table.symbol_as_function(action_id)?;

        for (i, member_id) in symbol_table.members(action_id)?.iter().rev().enumerate() {
            let arg_inst = symbol_table.symbol_as_instance(*member_id);
            let arg_type = symbol_table.symbol_as_type(arg_inst.type_id());
            if matches!(
                *arg_type.entrails(),
                TypeEntrails::Clipped(MetaSymbolKind::NonTerminal(_))
            ) {
                let arg_name = symbol_table.name(arg_inst.name_id());
                code.push(format!("// Ignore clipped member '{}'", arg_name));
                code.push("self.pop(context);".to_string());
            } else if !matches!(*arg_type.entrails(), TypeEntrails::Token)
                && !matches!(
                    *arg_type.entrails(),
                    TypeEntrails::UserDefinedType(MetaSymbolKind::Token, _)
                )
                && arg_inst.sem() != SymbolAttribute::Clipped
            {
                let arg_name = symbol_table.name(arg_inst.name_id());
                let stack_pop_data = UserTraitFunctionStackPopDataBuilder::default()
                    .arg_name(arg_name.to_string())
                    .arg_type(arg_type.inner_name())
                    .vec_anchor(arg_inst.sem() == SymbolAttribute::RepetitionAnchor)
                    .vec_push_semantic(
                        function.sem == ProductionAttribute::AddToCollection && i == 0,
                    )
                    .build()
                    .unwrap();
                code.push(format!("{}", stack_pop_data));
            }
        }
        Ok(())
    }

    fn generate_push_semantic<C: CommonGeneratorConfig + UserTraitGeneratorConfig>(
        &self,
        config: &C,
        code: &mut StrVec,
        action_id: SymbolId,
        symbol_table: &SymbolTable,
    ) -> Result<()> {
        let function = symbol_table.symbol_as_function(action_id)?;
        let fn_type = symbol_table.symbol_as_type(action_id);
        let fn_name = symbol_table.name(fn_type.name_id()).to_string();

        if config.auto_generate() && function.sem == ProductionAttribute::AddToCollection {
            let last_arg = symbol_table
                .members(action_id)?
                .iter()
                .last()
                .ok_or_else(|| anyhow!("There should be at least one argument!"))?;
            let arg_inst = symbol_table.symbol_as_instance(*last_arg);
            let arg_name = symbol_table.name(arg_inst.name_id());
            code.push("// Add an element to the vector".to_string());
            code.push(format!(" {}.push({}_built);", arg_name, fn_name,));
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
            // Clipped element is ignored here
            code.push(format!(
                "// Ignore clipped member '{}'",
                symbol_table.name(arg_inst.name_id())
            ));
            return Ok(());
        }
        let arg_type = symbol_table.symbol_as_type(arg_inst.type_id());
        if !matches!(*arg_type.entrails(), TypeEntrails::Clipped(_)) {
            let arg_name = symbol_table.name(arg_inst.name_id());
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

    fn generate_result_builder<C: CommonGeneratorConfig + UserTraitGeneratorConfig>(
        &self,
        config: &C,
        code: &mut StrVec,
        action_id: SymbolId,
        type_info: &GrammarTypeInfo,
    ) -> Result<()> {
        if !config.auto_generate() {
            return Ok(());
        }

        let symbol_table = &type_info.symbol_table;
        let function = symbol_table.symbol_as_function(action_id)?;
        let fn_type = symbol_table.symbol_as_type(action_id);
        let fn_name = symbol_table.name(fn_type.name_id()).to_string();
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
            code.push(format!("let {}_built = {} {{", fn_name, nt_type.name()));
            for member_id in symbol_table.members(action_id)?.iter().rev().skip(1) {
                Self::format_builder_call(symbol_table, member_id, function.sem, code)?;
            }
            code.push(r#"};"#.to_string());
        } else if function.sem == ProductionAttribute::OptionalSome {
            code.push(format!("let {}_built = {} {{", fn_name, nt_type.name()));
            for member_id in symbol_table.members(action_id)? {
                Self::format_builder_call(symbol_table, member_id, function.sem, code)?;
            }
            code.push(r#"};"#.to_string());
        } else if function.sem == ProductionAttribute::OptionalNone {
            // Don't generate a builder!
        } else {
            let builder_prefix = if function.alts == 1 {
                nt_type.name()
            } else {
                fn_out_type.name()
            };
            code.push(format!("let {}_built = {} {{", fn_name, builder_prefix));
            for member_id in symbol_table.members(action_id)? {
                Self::format_builder_call(symbol_table, member_id, function.sem, code)?;
            }
            code.push("};".to_string());
            if function.alts > 1 {
                // Type adjustment to the non-terminal enum
                // let list_0 = List::List0(list_0);
                let enum_variant_name = symbol_table
                    .members(nt_type.my_id())?
                    .iter()
                    .find(|variant| {
                        let enum_variant = symbol_table.symbol_as_type(**variant);
                        if let TypeEntrails::EnumVariant(inner_type) = enum_variant.entrails() {
                            *inner_type == fn_out_type.my_id()
                        } else {
                            false
                        }
                    })
                    .map(|enum_variant_id| {
                        symbol_table
                            .symbol(symbol_table.symbol(*enum_variant_id).my_id())
                            .name()
                    })
                    .ok_or_else(|| anyhow!("Enum variant not found"))?;
                code.push(format!(
                    "let {}_built = {}::{}({}_built);",
                    fn_name,
                    nt_type.name(),
                    enum_variant_name,
                    fn_name
                ));
            }
        }
        Ok(())
    }

    fn generate_user_action_call<C: CommonGeneratorConfig + UserTraitGeneratorConfig>(
        &self,
        config: &C,
        code: &mut StrVec,
        action_id: SymbolId,
        type_info: &GrammarTypeInfo,
    ) -> Result<()> {
        if !config.auto_generate() {
            return Ok(());
        }
        let symbol_table = &type_info.symbol_table;
        let function = symbol_table.symbol_as_function(action_id)?;
        let fn_type = symbol_table.symbol_as_type(action_id);
        let fn_name = symbol_table.name(fn_type.name_id()).to_string();
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

    fn generate_stack_push<C: CommonGeneratorConfig + UserTraitGeneratorConfig>(
        &self,
        config: &C,
        code: &mut StrVec,
        action_id: SymbolId,
        symbol_table: &SymbolTable,
    ) -> Result<()> {
        if config.auto_generate() {
            let function = symbol_table.symbol_as_function(action_id)?;
            let fn_type = symbol_table.symbol_as_type(action_id);
            let fn_name = symbol_table.name(fn_type.name_id()).to_string();

            if function.sem == ProductionAttribute::AddToCollection {
                // The output type of the action is the type generated for the action's non-terminal
                // filled with type of the action's last argument (the vector)
                let last_arg = symbol_table
                    .members(action_id)?
                    .iter()
                    .last()
                    .ok_or_else(|| anyhow!("There should be at least one argument!"))?;
                let arg_inst = symbol_table.symbol_as_instance(*last_arg);
                let arg_name = symbol_table.name(arg_inst.name_id());

                code.push(format!(
                    "self.push(ASTType::{}({}), context);",
                    NmHlp::to_upper_camel_case(&function.non_terminal),
                    arg_name
                ));
            } else if function.sem == ProductionAttribute::OptionalNone {
                code.push(format!(
                    "self.push(ASTType::{}(None), context);",
                    NmHlp::to_upper_camel_case(&function.non_terminal),
                ));
            } else if function.sem == ProductionAttribute::OptionalSome {
                code.push(format!(
                    "self.push(ASTType::{}(Some(Box::new({}_built))), context);",
                    NmHlp::to_upper_camel_case(&function.non_terminal),
                    fn_name,
                ));
            } else {
                // The output type of the action is the type generated for the action's non-terminal
                // filled with type kind of the action
                code.push(format!(
                    "self.push(ASTType::{}({}_built), context);",
                    NmHlp::to_upper_camel_case(&function.non_terminal),
                    fn_name,
                ));
            }
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
                    "{}: {}{}{}",
                    NmHlp::add_unused_indicator(arg_inst.used(), &arg_inst.name()),
                    arg_inst.reference(),
                    arg_type.inner_name(),
                    arg_type.lifetime()
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
        let type_name = symbol_table.name(type_symbol.name_id()).to_string();
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
        let lifetime = symbol_table.lifetime(t);
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
        type_info: &mut GrammarTypeInfo,
    ) -> Result<String> {
        if config.range() && !config.auto_generate() {
            bail!("Range information can only be generated in auto-generation mode!");
        }
        type_info.build(self.grammar_config)?;
        type_info.set_auto_generate(config.auto_generate())?;

        type_info.symbol_table.propagate_lifetimes();

        let production_output_types = if config.auto_generate() {
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
        } else {
            StrVec::new(0)
        };

        let non_terminal_types = if config.auto_generate() {
            type_info
                .non_terminal_types
                .iter()
                .try_fold(StrVec::new(0), |acc, (s, t)| {
                    Self::generate_single_non_terminal_type(s, t, type_info, acc, config)
                })?
        } else {
            StrVec::new(0)
        };

        let mut ast_type_decl = if config.auto_generate() {
            let mut comment = StrVec::new(0);
            comment.push(String::default());
            comment.push("Deduced ASTType of expanded grammar".to_string());
            comment.push(String::default());
            Self::format_type(type_info.ast_enum_type, &type_info.symbol_table, comment)?.unwrap()
        } else {
            String::default()
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
                self.generate_single_trait_function(a, type_info, config, acc)
            })?;

        let user_trait_functions = if config.auto_generate() {
            type_info
                .get_user_actions()
                .iter()
                .try_fold(StrVec::new(0).first_line_no_indent(), |acc, fn_id| {
                    Self::generate_single_user_trait_function(type_info, fn_id, acc)
                })?
        } else {
            StrVec::default()
        };

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
            .auto_generate(config.auto_generate())
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
            .module_name(config.module_name())
            .user_trait_functions(user_trait_functions)
            .build()
            .unwrap();

        Ok(format!("{}", user_trait_data))
    }

    fn generate_single_trait_function<C: CommonGeneratorConfig + UserTraitGeneratorConfig>(
        &self,
        a: (&usize, &SymbolId),
        type_info: &GrammarTypeInfo,
        config: &C,
        mut acc: StrVec,
    ) -> std::result::Result<StrVec, anyhow::Error> {
        let action_id = *a.1;
        let fn_type = type_info.symbol_table.symbol_as_type(action_id);
        let fn_name = type_info.symbol_table.name(fn_type.name_id()).to_string();
        let function = type_info.symbol_table.symbol_as_function(action_id)?;
        let prod_num = function.prod_num;
        let prod_string = function.prod_string;
        let fn_arguments =
            self.generate_inner_action_args(config, action_id, &type_info.symbol_table)?;
        let mut code = StrVec::new(8);
        self.generate_context(config, &mut code);
        self.generate_token_assignments(config, &mut code, action_id, &type_info.symbol_table)?;
        self.generate_stack_pops(config, &mut code, action_id, &type_info.symbol_table)?;
        self.generate_result_builder(config, &mut code, action_id, type_info)?;
        self.generate_push_semantic(config, &mut code, action_id, &type_info.symbol_table)?;
        self.generate_user_action_call(config, &mut code, action_id, type_info)?;
        self.generate_stack_push(config, &mut code, action_id, &type_info.symbol_table)?;
        let user_trait_function_data = UserTraitFunctionDataBuilder::default()
            .fn_name(fn_name)
            .prod_num(prod_num)
            .fn_arguments(fn_arguments)
            .prod_string(prod_string)
            .named(config.auto_generate())
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
        let fn_name = type_info.symbol_table.name(fn_type.name_id()).to_string();
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
