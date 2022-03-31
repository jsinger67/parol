use std::collections::HashSet;
use std::convert::TryInto;

use super::grammar_type_generator::{ASTType, Action, GrammarTypeInfo};
use super::template_data::{
    NonTerminalTypeEnum, NonTerminalTypeStruct, NonTerminalTypeVec,
    UserTraitCallerFunctionDataBuilder, UserTraitDataBuilder, UserTraitFunctionDataBuilder,
    UserTraitFunctionStackPopDataBuilder,
};
use crate::generators::naming_helper::NamingHelper as NmHlp;
use crate::generators::GrammarConfig;
use crate::grammar::{ProductionAttribute, SymbolAttribute};
use crate::parser::{ParolGrammarItem, Production};
use crate::{ParolGrammar, Pr, StrVec};
use log::trace;
use miette::{IntoDiagnostic, Result};

/// Generator for user trait code
#[derive(Builder, Debug, Default)]
pub struct UserTraitGenerator<'a> {
    /// User type that implements the language processing
    user_type_name: String,
    /// User type's module name
    module_name: &'a str,
    /// Enable feature auto-generation for expanded grammar's semantic actions
    auto_generate: bool,
    /// Parsed original user grammar
    parol_grammar: &'a ParolGrammar,
    /// Compiled grammar configuration
    grammar_config: &'a GrammarConfig,
}

impl<'a> UserTraitGenerator<'a> {
    fn generate_inner_action_args(&self, action: &Action) -> String {
        // We reference the parse_tree argument only if a token is in the argument list
        let lifetime = if self.auto_generate { "<'t>" } else { "" };
        let mut parse_tree_argument_used = false;
        let mut arguments = action
            .args
            .iter()
            .map(|a| {
                if matches!(a.arg_type, ASTType::Token(_)) {
                    parse_tree_argument_used = true;
                }
                format!("{}: &ParseTreeStackEntry{}", a.name(), lifetime)
            })
            .collect::<Vec<String>>();
        arguments.push(format!(
            "{}parse_tree: &Tree<ParseTreeType{}>",
            NmHlp::item_unused_indicator(self.auto_generate && parse_tree_argument_used),
            lifetime
        ));
        arguments.join(", ")
    }

    fn generate_context(&self, code: &mut StrVec, action: &Action) {
        if self.auto_generate {
            code.push(format!("let context = \"{}\";", action.fn_name));
            code.push("trace!(\"{}\", self.trace_item_stack(context));".to_string());
        }
    }

    fn generate_token_assignments(&self, code: &mut StrVec, action: &Action) {
        if self.auto_generate {
            action
                .args
                .iter()
                .filter(|a| matches!(a.arg_type, ASTType::Token(_)))
                .for_each(|arg| {
                    let arg_name = arg.name();
                    code.push(format!(
                        "let {} = *{}.token(parse_tree)?;",
                        arg_name, arg_name
                    ))
                });
        }
    }

    fn generate_stack_pops(&self, code: &mut StrVec, action: &Action) -> Result<()> {
        if self.auto_generate {
            action
                .args
                .iter()
                .rev()
                .enumerate()
                .filter(|(_, a)| !matches!(a.arg_type, ASTType::Token(_)))
                .fold(Ok(()), |res: Result<()>, (i, arg)| {
                    res?;
                    let stack_pop_data = UserTraitFunctionStackPopDataBuilder::default()
                        .arg_name(arg.name.clone())
                        .arg_type(arg.arg_type.inner_type_name())
                        .vec_anchor(arg.sem == SymbolAttribute::RepetitionAnchor)
                        .vec_push_semantic(
                            action.sem == ProductionAttribute::AddToCollection && i == 0,
                        )
                        .build()
                        .into_diagnostic()?;
                    code.push(format!("{}", stack_pop_data));
                    Ok(())
                })
        } else {
            Ok(())
        }
    }

    fn generate_push_semantic(&self, code: &mut StrVec, action: &Action) {
        if self.auto_generate && action.sem == ProductionAttribute::AddToCollection {
            code.push("// Add an element to the vector".to_string());
            code.push(format!(
                " {}.push({}_built);",
                action.args.iter().last().unwrap().name,
                &action.fn_name,
            ));
        }
    }

    fn generate_result_builder(&self, code: &mut StrVec, action: &Action) {
        if self.auto_generate {
            if action.sem == ProductionAttribute::CollectionStart {
                code.push(format!("let {}_built = Vec::new();", action.fn_name));
            } else if action.sem == ProductionAttribute::AddToCollection {
                code.push(format!(
                    "let {}_built = {}Builder::default()",
                    action.fn_name,
                    NmHlp::to_upper_camel_case(&action.non_terminal)
                ));
                action.args.iter().rev().skip(1).for_each(|arg| {
                    let setter_name = &arg.name;
                    let arg_name = if matches!(arg.arg_type, ASTType::TypeRef(_))
                        && arg.sem == SymbolAttribute::None
                    {
                        format!("Box::new({})", &arg.name)
                    } else {
                        arg.name.clone()
                    };
                    code.push(format!("    .{}({})", setter_name, arg_name));
                });
                code.push("    .build()".to_string());
                code.push("    .into_diagnostic()?;".to_string());
            } else {
                let builder_prefix = if action.alts == 1 {
                    &action.non_terminal
                } else {
                    &action.fn_name
                };
                code.push(format!(
                    "let {}_built = {}Builder::default()",
                    action.fn_name,
                    NmHlp::to_upper_camel_case(builder_prefix)
                ));
                action.args.iter().for_each(|arg| {
                    let setter_name = &arg.name;
                    let arg_name = if matches!(arg.arg_type, ASTType::TypeRef(_)) {
                        format!("Box::new({})", &arg.name)
                    } else {
                        arg.name.clone()
                    };
                    code.push(format!("    .{}({})", setter_name, arg_name));
                });
                code.push("    .build()".to_string());
                code.push("    .into_diagnostic()?;".to_string());
                if action.alts > 1 {
                    // Type adjustment to the non-terminal enum
                    // let list_0 = List::List0(list_0);
                    code.push(format!(
                        "let {}_built = {}::{}({}_built);",
                        action.fn_name,
                        NmHlp::to_upper_camel_case(&action.non_terminal),
                        NmHlp::to_upper_camel_case(builder_prefix),
                        action.fn_name
                    ));
                }
            }
        }
    }

    fn generate_user_action_call(
        &self,
        code: &mut StrVec,
        action: &Action,
        parol_grammar: &'a ParolGrammar,
    ) {
        if self.auto_generate
            && parol_grammar
                .item_stack
                .iter()
                .filter_map(|item| match item {
                    ParolGrammarItem::Prod(Production { lhs, .. }) => Some(lhs),
                    _ => None,
                })
                .any(|lhs| &action.non_terminal == lhs)
        {
            code.push("// Calling user action here".to_string());
            code.push(format!(
                "self.user_grammar.{}(&{}_built)?;",
                NmHlp::to_lower_snake_case(&action.non_terminal),
                action.fn_name
            ));
        }
    }

    fn generate_stack_push(&self, code: &mut StrVec, action: &Action) {
        if self.auto_generate {
            if action.sem == ProductionAttribute::AddToCollection {
                // The output type of the action is the type generated for the action's non-terminal
                // filled with type of the action's last argument (the vector)
                code.push(format!(
                    "self.push(ASTType::{}({}), context);",
                    NmHlp::to_upper_camel_case(&action.non_terminal),
                    action.args.iter().last().unwrap().name
                ));
            } else {
                // The output type of the action is the type generated for the action's non-terminal
                // filled with type kind of the action
                code.push(format!(
                    "self.push(ASTType::{}({}_built), context);",
                    NmHlp::to_upper_camel_case(&action.non_terminal),
                    action.fn_name
                ));
            }
        }
    }

    fn generate_user_action_args(non_terminal: &str) -> String {
        format!("_arg: &{}<'t>", NmHlp::to_upper_camel_case(non_terminal))
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
        ast_type: &ASTType,
        non_terminal: &str,
        prod_num: Option<usize>,
        comment: StrVec,
    ) -> Option<String> {
        let non_terminal = if let Some(prod_num) = prod_num {
            NmHlp::to_upper_camel_case(&format!("{}_{}", non_terminal, prod_num))
        } else {
            NmHlp::to_upper_camel_case(non_terminal)
        };

        match ast_type {
            ASTType::Struct(_n, m) => {
                let struct_data = NonTerminalTypeStruct {
                    comment,
                    non_terminal,
                    lifetime: ast_type.lifetime(),
                    members: m.iter().fold(StrVec::new(4), |mut acc, (n, t)| {
                        acc.push(format!("{}: {},", n, t));
                        acc
                    }),
                };
                Some(format!("{}", struct_data))
            }
            ASTType::Enum(n, m) => {
                let struct_data = NonTerminalTypeEnum {
                    comment,
                    non_terminal: n.to_string(),
                    lifetime: ast_type.lifetime(),
                    members: m.iter().fold(StrVec::new(4), |mut acc, (c, t)| {
                        acc.push(NmHlp::to_upper_camel_case(&format!(
                            "{}({}),",
                            c,
                            t.type_name(),
                        )));
                        acc
                    }),
                };
                Some(format!("{}", struct_data))
            }
            ASTType::Repeat(r) => {
                let struct_data = NonTerminalTypeVec {
                    comment,
                    non_terminal,
                    lifetime: ast_type.lifetime(),
                    type_ref: r.clone(),
                };
                Some(format!("{}", struct_data))
            }
            ASTType::Unit => {
                let struct_data = NonTerminalTypeStruct {
                    comment,
                    non_terminal,
                    lifetime: ast_type.lifetime(),
                    members: StrVec::new(0),
                };
                Some(format!("{}", struct_data))
            }
            _ => None,
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
        let mut type_info: GrammarTypeInfo = (self.grammar_config).try_into()?;
        type_info.set_auto_generate(self.auto_generate);

        let production_output_types = if self.auto_generate {
            type_info
                .actions
                .iter()
                .filter(|a| a.alts > 1 && a.sem == ProductionAttribute::None)
                .fold(StrVec::new(0), |mut acc, a| {
                    let mut comment = StrVec::new(0);
                    comment.push(String::default());
                    comment.push(format!("Type derived for production {}", a.prod_num));
                    comment.push(String::default());
                    comment.push(a.prod_string.clone());
                    comment.push(String::default());
                    Self::format_type(&a.out_type, &a.non_terminal, Some(a.prod_num), comment)
                        .into_iter()
                        .for_each(|s| acc.push(s));
                    acc
                })
        } else {
            StrVec::new(0)
        };

        let non_terminal_types = if self.auto_generate {
            type_info
                .non_terminal_types
                .iter()
                .fold(StrVec::new(0), |mut acc, (s, t)| {
                    let mut comment = StrVec::new(0);
                    comment.push(String::default());
                    comment.push(format!("Type derived for non-terminal {}", s));
                    comment.push(String::default());
                    Self::format_type(t, s, None, comment)
                        .into_iter()
                        .for_each(|s| acc.push(s));
                    acc
                })
        } else {
            StrVec::new(0)
        };

        let ast_type_decl = if self.auto_generate {
            let mut comment = StrVec::new(0);
            comment.push(String::default());
            comment.push("Deduced ASTType of expanded grammar".to_string());
            comment.push(String::default());
            Self::format_type(&type_info.ast_enum_type, "ASTType", None, comment).unwrap()
        } else {
            String::default()
        };

        let trait_functions = type_info.actions.iter().fold(
            Ok(StrVec::new(0).first_line_no_indent()),
            |acc: Result<StrVec>, a| {
                if let Ok(mut acc) = acc {
                    let fn_name = &a.fn_name;
                    let prod_string = a.prod_string.clone();
                    let fn_arguments = self.generate_inner_action_args(a);
                    let mut code = StrVec::new(8);
                    self.generate_context(&mut code, a);
                    self.generate_token_assignments(&mut code, a);
                    self.generate_stack_pops(&mut code, a)?;
                    self.generate_result_builder(&mut code, a);
                    self.generate_push_semantic(&mut code, a);
                    self.generate_user_action_call(&mut code, a, self.parol_grammar);
                    self.generate_stack_push(&mut code, a);
                    let user_trait_function_data = UserTraitFunctionDataBuilder::default()
                        .fn_name(fn_name)
                        .prod_num(a.prod_num)
                        .fn_arguments(fn_arguments)
                        .prod_string(prod_string)
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
            trace!(
                "parol_grammar.item_stack:\n{:?}",
                self.parol_grammar.item_stack
            );

            let mut processed_non_terminals: HashSet<String> = HashSet::new();
            self.parol_grammar
                .item_stack
                .iter()
                .fold(
                    Ok((StrVec::new(0).first_line_no_indent(), 0)),
                    |acc: Result<(StrVec, usize)>, p| {
                        if let Ok((mut acc, mut i)) = acc {
                            if let ParolGrammarItem::Prod(Production { lhs, rhs: _ }) = p {
                                if !processed_non_terminals.contains(lhs) {
                                    let fn_name =
                                        NmHlp::escape_rust_keyword(NmHlp::to_lower_snake_case(lhs));
                                    let prod_string = p.to_par();
                                    let fn_arguments = Self::generate_user_action_args(lhs);
                                    let code = StrVec::default();
                                    let user_trait_function_data =
                                        UserTraitFunctionDataBuilder::default()
                                            .fn_name(&fn_name)
                                            .prod_num(i)
                                            .fn_arguments(fn_arguments)
                                            .prod_string(prod_string)
                                            .code(code)
                                            .inner(false)
                                            .build()
                                            .into_diagnostic()?;

                                    acc.push(format!("{}", user_trait_function_data));
                                    processed_non_terminals.insert(lhs.to_string());
                                }
                                i += 1;
                            }
                            Ok((acc, i))
                        } else {
                            acc
                        }
                    },
                )?
                .0
        } else {
            StrVec::default()
        };

        trace!("user_trait_functions:\n{}", user_trait_functions);

        let trait_caller = self.grammar_config.cfg.pr.iter().enumerate().fold(
            Ok(StrVec::new(12)),
            |acc: Result<StrVec>, (i, p)| {
                if let Ok(mut acc) = acc {
                    let fn_name = NmHlp::to_lower_snake_case(&format!("{}_{}", p.get_n_str(), i));
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
        let user_trait_data = UserTraitDataBuilder::default()
            .user_type_name(&self.user_type_name)
            .auto_generate(self.auto_generate)
            .production_output_types(production_output_types)
            .non_terminal_types(non_terminal_types)
            .ast_type_decl(&ast_type_decl)
            .trait_functions(trait_functions)
            .trait_caller(trait_caller)
            .module_name(self.module_name)
            .user_trait_functions(user_trait_functions)
            .build()
            .into_diagnostic()?;

        Ok(format!("{}", user_trait_data))
    }

    /// Creates a new item
    pub fn try_new(
        user_type_name: &'a str,
        module_name: &'a str,
        auto_generate: bool,
        parol_grammar: &'a ParolGrammar,
        grammar_config: &'a GrammarConfig,
    ) -> Result<Self> {
        let user_type_name = NmHlp::to_upper_camel_case(user_type_name);
        UserTraitGeneratorBuilder::default()
            .user_type_name(user_type_name)
            .module_name(module_name)
            .auto_generate(auto_generate)
            .grammar_config(grammar_config)
            .parol_grammar(parol_grammar)
            .build()
            .into_diagnostic()
    }
}
