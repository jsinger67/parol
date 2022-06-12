use std::collections::HashSet;

use super::grammar_type_generator::GrammarTypeInfo;
use super::symbol_table::{MetaSymbolKind, SymbolId, SymbolTable, TypeEntrails};
use super::symbol_table_facade::{InstanceFacade, SymbolFacade, TypeFacade};
use super::template_data::{
    NonTerminalTypeEnum, NonTerminalTypeStruct, UserTraitCallerFunctionDataBuilder,
    UserTraitDataBuilder, UserTraitFunctionDataBuilder, UserTraitFunctionStackPopDataBuilder,
};
use crate::generators::naming_helper::NamingHelper as NmHlp;
use crate::generators::GrammarConfig;
use crate::grammar::{ProductionAttribute, SymbolAttribute};
use crate::parser::Production;
use crate::{Pr, StrVec};
use log::trace;
use miette::{bail, miette, IntoDiagnostic, Result};

/// Generator for user trait code
/// The lifetime parameter `'a` refers to the lifetime of the contained references.
/// The lifetime parameter `'t` refers to the lifetime of the scanned text.
#[derive(Builder, Debug, Default)]
pub struct UserTraitGenerator<'a> {
    /// User type that implements the language processing
    user_type_name: String,
    /// User type's module name
    module_name: &'a str,
    /// Enable feature auto-generation for expanded grammar's semantic actions
    auto_generate: bool,
    /// Parsed original user grammar
    productions: Vec<Production>,
    /// Compiled grammar configuration
    grammar_config: &'a GrammarConfig,
}

impl<'a> UserTraitGenerator<'a> {
    fn generate_inner_action_args(
        &self,
        action_id: SymbolId,
        symbol_table: &SymbolTable,
    ) -> Result<String> {
        // We reference the parse_tree argument only if a token is in the argument list
        let lifetime = if self.auto_generate { "<'t>" } else { "" };
        let mut parse_tree_argument_used = false;
        let mut arguments = Vec::new();

        for member_id in symbol_table.members(action_id)? {
            let arg_inst = symbol_table.symbol_as_instance(*member_id);
            let arg_type = symbol_table.symbol_as_type(arg_inst.type_id());
            if matches!(*arg_type.entrails(), TypeEntrails::Token)
                && arg_inst.sem() != SymbolAttribute::Clipped
            {
                parse_tree_argument_used = true;
            }
            arguments.push(format!(
                "{}: &ParseTreeStackEntry{}",
                NmHlp::add_unused_indicator(arg_inst.used(), symbol_table.name(arg_inst.name_id())),
                lifetime
            ));
        }

        arguments.push(format!(
            "{}parse_tree: &Tree<ParseTreeType{}>",
            NmHlp::item_unused_indicator(self.auto_generate && parse_tree_argument_used),
            lifetime
        ));
        Ok(arguments.join(", "))
    }

    fn generate_context(&self, code: &mut StrVec) {
        if self.auto_generate {
            code.push("let context = function_name!();".to_string());
            code.push("trace!(\"{}\", self.trace_item_stack(context));".to_string());
        }
    }

    fn generate_token_assignments(
        &self,
        code: &mut StrVec,
        action_id: SymbolId,
        symbol_table: &SymbolTable,
    ) -> Result<()> {
        if !self.auto_generate {
            return Ok(());
        }

        for member_id in symbol_table.members(action_id)? {
            let arg_inst = symbol_table.symbol_as_instance(*member_id);
            let arg_type = symbol_table.symbol_as_type(arg_inst.type_id());
            if matches!(arg_type.entrails(), TypeEntrails::Token) {
                let arg_name = symbol_table.name(arg_inst.name_id());
                code.push(format!(
                    "let {} = *{}.token(parse_tree)?;",
                    arg_name, arg_name
                ))
            }
        }
        Ok(())
    }

    fn generate_stack_pops(
        &self,
        code: &mut StrVec,
        action_id: SymbolId,
        symbol_table: &SymbolTable,
    ) -> Result<()> {
        if !self.auto_generate {
            return Ok(());
        }

        let function = symbol_table.symbol_as_function(action_id)?;

        for (i, member_id) in symbol_table.members(action_id)?.iter().rev().enumerate() {
            let arg_inst = symbol_table.symbol_as_instance(*member_id);
            let arg_type = symbol_table.symbol_as_type(arg_inst.type_id());
            if matches!(
                *arg_type.entrails(),
                TypeEntrails::Clipped(MetaSymbolKind::NonTerminal)
            ) {
                let arg_name = symbol_table.name(arg_inst.name_id());
                code.push(format!("// Ignore clipped member '{}'", arg_name));
                code.push("self.pop(context);".to_string());
            } else if !matches!(*arg_type.entrails(), TypeEntrails::Token)
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
                    .into_diagnostic()?;
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
        let fn_name = symbol_table.name(fn_type.name_id()).to_string();

        if self.auto_generate && function.sem == ProductionAttribute::AddToCollection {
            let last_arg = symbol_table
                .members(action_id)?
                .iter()
                .last()
                .ok_or_else(|| miette!("There should be at least one argument!"))?;
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
            } else {
                arg_name.to_string()
            };
            code.push(format!("    .{}({})", setter_name, arg_name));
        }
        Ok(())
    }

    fn generate_result_builder(
        &self,
        code: &mut StrVec,
        action_id: SymbolId,
        type_info: &GrammarTypeInfo,
    ) -> Result<()> {
        if !self.auto_generate {
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
                .ok_or_else(|| miette!("Production output type not accessible!"))?,
        );
        let nt_type = symbol_table.symbol_as_type(
            *type_info
                .non_terminal_types
                .get(&function.non_terminal)
                .ok_or_else(|| miette!("Non-terminal type not accessible!"))?,
        );

        if function.sem == ProductionAttribute::CollectionStart {
            code.push(format!("let {}_built = Vec::new();", fn_name));
        } else if function.sem == ProductionAttribute::AddToCollection {
            code.push(format!(
                "let {}_built = {}Builder::default()",
                fn_name,
                nt_type.name()
            ));
            for member_id in symbol_table.members(action_id)?.iter().rev().skip(1) {
                Self::format_builder_call(symbol_table, member_id, function.sem, code)?;
            }
            code.push("    .build()".to_string());
            code.push("    .into_diagnostic()?;".to_string());
        } else if function.sem == ProductionAttribute::OptionalSome {
            code.push(format!(
                "let {}_built = {}Builder::default()",
                fn_name,
                nt_type.name()
            ));
            for member_id in symbol_table.members(action_id)? {
                Self::format_builder_call(symbol_table, member_id, function.sem, code)?;
            }
            code.push("    .build()".to_string());
            code.push("    .into_diagnostic()?;".to_string());
        } else if function.sem == ProductionAttribute::OptionalNone {
            // Don't generate a builder!
        } else {
            let builder_prefix = if function.alts == 1 {
                nt_type.name()
            } else {
                fn_out_type.name()
            };
            code.push(format!(
                "let {}_built = {}Builder::default()",
                fn_name, builder_prefix
            ));
            for member_id in symbol_table.members(action_id)? {
                Self::format_builder_call(symbol_table, member_id, function.sem, code)?;
            }
            code.push("    .build()".to_string());
            code.push("    .into_diagnostic()?;".to_string());
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
                    .ok_or_else(|| miette!("Enum variant not found"))?;
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

    fn generate_user_action_call(
        &self,
        code: &mut StrVec,
        action_id: SymbolId,
        type_info: &GrammarTypeInfo,
    ) -> Result<()> {
        if !self.auto_generate {
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

    fn generate_stack_push(
        &self,
        code: &mut StrVec,
        action_id: SymbolId,
        symbol_table: &SymbolTable,
    ) -> Result<()> {
        if self.auto_generate {
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
                    .ok_or_else(|| miette!("There should be at least one argument!"))?;
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
                    fn_name
                ));
            } else {
                // The output type of the action is the type generated for the action's non-terminal
                // filled with type kind of the action
                code.push(format!(
                    "self.push(ASTType::{}({}_built), context);",
                    NmHlp::to_upper_camel_case(&function.non_terminal),
                    fn_name
                ));
            }
        }
        Ok(())
    }

    pub(crate) fn add_user_actions(&self, type_info: &mut GrammarTypeInfo) -> Result<()> {
        let mut processed_non_terminals: HashSet<String> = HashSet::new();
        self.productions.iter().fold(Ok(()), |acc: Result<()>, p| {
            acc?;
            if !processed_non_terminals.contains(&p.lhs) {
                type_info.add_user_action(&p.lhs)?;
                processed_non_terminals.insert(p.lhs.to_string());
            }
            Ok(())
        })
    }

    fn generate_user_action_args(
        non_terminal: &str,
        type_info: &GrammarTypeInfo,
    ) -> Result<String> {
        let type_name = NmHlp::to_upper_camel_case(non_terminal);
        if let Some(symbol_id) = type_info.symbol_table.get_global_type(&type_name) {
            Ok(format!(
                "_arg: &{}{}",
                type_name,
                type_info.symbol_table.lifetime(symbol_id)
            ))
        } else {
            Err(miette!("Can't find type of argument {}", non_terminal))
        }
    }

    fn generate_caller_argument_list(pr: &Pr) -> String {
        let mut arguments = pr
            .get_r()
            .iter()
            .filter(|s| !s.is_switch())
            .enumerate()
            .map(|(i, _)| format!("&children[{}]", i))
            .collect::<Vec<String>>();
        arguments.push("parse_tree".to_string());
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
            _ => bail!("Unexpected type!"),
        }
    }

    // ---------------------------------------------------
    // Part of the Public API
    // *Changes will affect crate's version according to semver*
    // ---------------------------------------------------
    ///
    /// Generates the file with the user actions trait.
    ///
    pub fn generate_user_trait_source(&self) -> Result<String> {
        let mut type_info: GrammarTypeInfo = GrammarTypeInfo::try_new(&self.user_type_name)?;
        type_info.build(self.grammar_config)?;
        type_info.set_auto_generate(self.auto_generate)?;

        self.add_user_actions(&mut type_info)?;
        type_info.symbol_table.propagate_lifetimes();

        let production_output_types = if self.auto_generate {
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
                .fold(Ok(StrVec::new(0)), |acc: Result<StrVec>, (t, f)| {
                    if let Ok(mut acc) = acc {
                        let mut comment = StrVec::new(0);
                        comment.push(String::default());
                        comment.push(format!("Type derived for production {}", f.prod_num));
                        comment.push(String::default());
                        comment.push(f.prod_string);
                        comment.push(String::default());
                        Self::format_type(*t, &type_info.symbol_table, comment)?
                            .into_iter()
                            .for_each(|s| acc.push(s));
                        Ok(acc)
                    } else {
                        acc
                    }
                })?
        } else {
            StrVec::new(0)
        };

        let non_terminal_types = if self.auto_generate {
            type_info.non_terminal_types.iter().fold(
                Ok(StrVec::new(0)),
                |acc: Result<StrVec>, (s, t)| {
                    if let Ok(mut acc) = acc {
                        let mut comment = StrVec::new(0);
                        comment.push(String::default());
                        comment.push(format!("Type derived for non-terminal {}", s));
                        comment.push(String::default());
                        Self::format_type(*t, &type_info.symbol_table, comment)?
                            .into_iter()
                            .for_each(|s| acc.push(s));
                        Ok(acc)
                    } else {
                        acc
                    }
                },
            )?
        } else {
            StrVec::new(0)
        };

        let ast_type_decl = if self.auto_generate {
            let mut comment = StrVec::new(0);
            comment.push(String::default());
            comment.push("Deduced ASTType of expanded grammar".to_string());
            comment.push(String::default());
            Self::format_type(type_info.ast_enum_type, &type_info.symbol_table, comment)?.unwrap()
        } else {
            String::default()
        };

        let trait_functions = type_info.adapter_actions.iter().fold(
            Ok(StrVec::new(0).first_line_no_indent()),
            |acc: Result<StrVec>, a| {
                if let Ok(mut acc) = acc {
                    let action_id = *a.1;
                    let fn_type = type_info.symbol_table.symbol_as_type(action_id);
                    let fn_name = type_info.symbol_table.name(fn_type.name_id()).to_string();
                    let function = type_info.symbol_table.symbol_as_function(action_id)?;
                    let prod_num = function.prod_num;
                    let prod_string = function.prod_string;
                    let fn_arguments =
                        self.generate_inner_action_args(action_id, &type_info.symbol_table)?;
                    let mut code = StrVec::new(8);
                    self.generate_context(&mut code);
                    self.generate_token_assignments(&mut code, action_id, &type_info.symbol_table)?;
                    self.generate_stack_pops(&mut code, action_id, &type_info.symbol_table)?;
                    self.generate_result_builder(&mut code, action_id, &type_info)?;
                    self.generate_push_semantic(&mut code, action_id, &type_info.symbol_table)?;
                    self.generate_user_action_call(&mut code, action_id, &type_info)?;
                    self.generate_stack_push(&mut code, action_id, &type_info.symbol_table)?;
                    let user_trait_function_data = UserTraitFunctionDataBuilder::default()
                        .fn_name(fn_name)
                        .prod_num(prod_num)
                        .fn_arguments(fn_arguments)
                        .prod_string(prod_string)
                        .named(self.auto_generate)
                        .code(code)
                        .inner(true)
                        .build()
                        .into_diagnostic()?;
                    acc.push(format!("{}", user_trait_function_data));
                    Ok(acc)
                } else {
                    acc
                }
            },
        )?;

        let user_trait_functions = if self.auto_generate {
            trace!("parol_grammar.item_stack:\n{:?}", self.productions);

            type_info.user_actions.iter().fold(
                Ok(StrVec::new(0).first_line_no_indent()),
                |acc: Result<StrVec>, (nt, fn_id)| {
                    if let Ok(mut acc) = acc {
                        let fn_name = type_info.symbol_table.type_name(*fn_id)?;
                        let fn_arguments = Self::generate_user_action_args(nt, &type_info)?;
                        let user_trait_function_data = UserTraitFunctionDataBuilder::default()
                            .fn_name(fn_name)
                            .non_terminal(nt.to_string())
                            .fn_arguments(fn_arguments)
                            .build()
                            .into_diagnostic()?;
                        acc.push(format!("{}", user_trait_function_data));
                        Ok(acc)
                    } else {
                        acc
                    }
                },
            )?
        } else {
            StrVec::default()
        };

        trace!("user_trait_functions:\n{}", user_trait_functions);

        let trait_caller = self.grammar_config.cfg.pr.iter().enumerate().fold(
            Ok(StrVec::new(12)),
            |acc: Result<StrVec>, (i, p)| {
                if let Ok(mut acc) = acc {
                    let fn_type_id = type_info.adapter_actions.get(&i).unwrap();
                    let fn_type = type_info.symbol_table.symbol_as_type(*fn_type_id);
                    let fn_name = type_info.symbol_table.name(fn_type.name_id()).to_string();
                    let fn_arguments = Self::generate_caller_argument_list(p);
                    let user_trait_function_data = UserTraitCallerFunctionDataBuilder::default()
                        .fn_name(fn_name)
                        .prod_num(i)
                        .fn_arguments(fn_arguments)
                        .build()
                        .into_diagnostic()?;
                    acc.push(format!("{}", user_trait_function_data));
                    Ok(acc)
                } else {
                    acc
                }
            },
        )?;
        // $env:RUST_LOG="parol::generators::user_trait_generator=trace"
        trace!("// Type information:");
        trace!("{}", type_info);

        let user_trait_data = UserTraitDataBuilder::default()
            .user_type_name(&self.user_type_name)
            .auto_generate(self.auto_generate)
            .production_output_types(production_output_types)
            .non_terminal_types(non_terminal_types)
            .ast_type_decl(ast_type_decl)
            .trait_functions(trait_functions)
            .trait_caller(trait_caller)
            .module_name(self.module_name)
            .user_trait_functions(user_trait_functions)
            .build()
            .into_diagnostic()?;

        Ok(format!("{}", user_trait_data))
    }

    // ---------------------------------------------------
    // Part of the Public API
    // *Changes will affect crate's version according to semver*
    // ---------------------------------------------------
    /// Creates a new item
    pub fn try_new(
        user_type_name: &'a str,
        module_name: &'a str,
        auto_generate: bool,
        productions: Vec<Production>,
        grammar_config: &'a GrammarConfig,
    ) -> Result<Self> {
        let user_type_name = NmHlp::to_upper_camel_case(user_type_name);
        UserTraitGeneratorBuilder::default()
            .user_type_name(user_type_name)
            .module_name(module_name)
            .auto_generate(auto_generate)
            .grammar_config(grammar_config)
            .productions(productions)
            .build()
            .into_diagnostic()
    }
}
