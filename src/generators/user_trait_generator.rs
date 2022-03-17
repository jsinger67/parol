use std::collections::HashSet;
use std::convert::TryInto;

use super::grammar_type_generator::{ASTType, Argument, GrammarTypeInfo, Action};
use super::template_data::{
    NonTerminalTypeEnum, NonTerminalTypeStruct, NonTerminalTypeVec,
    UserTraitCallerFunctionDataBuilder, UserTraitDataBuilder, UserTraitFunctionDataBuilder,
};
use crate::generators::naming_helper::NamingHelper as NmHlp;
use crate::generators::GrammarConfig;
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
    fn generate_argument_list(&self, action: &Action) -> String {
        // We reference the parse_tree argument only if a token is in the argument list
        let mut parse_tree_argument_used = false;
        let mut arguments = action
            .args
            .iter()
            .map(|a| {
                if matches!(a.arg_type, ASTType::Token(_)) {
                    parse_tree_argument_used = true;
                }
                format!(
                    "{}{}: &ParseTreeStackEntry",
                    NmHlp::item_unused_indicator(self.auto_generate && a.used),
                    a.name,
                )
            })
            .collect::<Vec<String>>();
        arguments.push(format!(
            "{}parse_tree: &Tree<ParseTreeType>",
            NmHlp::item_unused_indicator(self.auto_generate && parse_tree_argument_used)
        ));
        arguments.join(", ")
    }

    fn generate_token_assignments(str_vec: &mut StrVec, action: &Action) {
        action.args.iter().filter(|a| matches!(a.arg_type, ASTType::Token(_))).for_each(|arg| {
            let arg_name = arg.name();
            // let num_0 = num_0.token(parse_tree)?.to_owned();
            str_vec.push(format!("let {} = {}.token(parse_tree)?.to_owned();", arg_name, arg_name))
        });
    }

    fn generate_user_action_args(non_terminal: &str) -> String {
        format!("_arg: {}", NmHlp::to_upper_camel_case(non_terminal))
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
                    members: m.iter().fold(StrVec::new(4), |mut acc, (c, t)| {
                        acc.push(NmHlp::to_upper_camel_case(&format!(
                            "{}({}),",
                            c,
                            t.type_name()
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
                    type_ref: r.clone(),
                };
                Some(format!("{}", struct_data))
            }
            ASTType::Unit => {
                let struct_data = NonTerminalTypeStruct {
                    comment,
                    non_terminal,
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
                .filter(|a| a.alts > 1)
                .fold(StrVec::new(0), |mut acc, a| {
                    let mut comment = StrVec::new(0);
                    comment.push(String::default());
                    comment.push(format!("Type derived for production {}", a.prod_num));
                    comment.push(String::default());
                    comment.push(a.prod_string.clone());
                    comment.push(String::default());
                    Self::format_type(
                        &a.out_type,
                        &a.non_terminal,
                        Some(a.prod_num),
                        comment,
                    )
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
            Self::format_type(
                &type_info.ast_enum_type,
                "ASTType",
                None,
                comment,
            )
            .unwrap()
        } else {
            String::default()
        };

        let trait_functions = type_info.actions.iter().fold(
            Ok(StrVec::new(0).first_line_no_indent()),
            |acc: Result<StrVec>, a| {
                if let Ok(mut acc) = acc {
                    let fn_name = &a.fn_name;
                    let prod_string = a.prod_string.clone();
                    let fn_arguments = self.generate_argument_list(a);
                    let mut code = StrVec::new(8);
                    Self::generate_token_assignments(&mut code, a);
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
            trace!("parol_grammar.item_stack:\n{:?}", self.parol_grammar.item_stack);

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
            .module_name(&self.module_name)
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
