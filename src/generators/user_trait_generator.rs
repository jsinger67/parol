use std::convert::TryInto;

use super::grammar_type_generator::{ASTType, Argument, GrammarTypeInfo};
use super::template_data::{
    NonTerminalTypeEnum, NonTerminalTypeStruct, NonTerminalTypeVec,
    UserTraitCallerFunctionDataBuilder, UserTraitDataBuilder, UserTraitFunctionDataBuilder,
};
use crate::generators::naming_helper::NamingHelper as NmHlp;
use crate::generators::GrammarConfig;
use crate::{ParolGrammar, Pr, StrVec};
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
    fn generate_argument_list(&self, args: &[Argument]) -> String {
        let mut arguments = args
            .iter()
            .map(|a| format!("{}: &ParseTreeStackEntry", a.name(),))
            .collect::<Vec<String>>();
        arguments.push("_parse_tree: &Tree<ParseTreeType>".to_string());
        arguments.join(", ")
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
        comment: String,
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
        type_info.adjust_arguments_used(self.auto_generate);

        let production_output_types = if self.auto_generate {
            type_info
                .actions
                .iter()
                .filter(|a| a.alts > 1)
                .fold(StrVec::new(0), |mut acc, a| {
                    Self::format_type(
                        &a.out_type,
                        &a.non_terminal,
                        Some(a.prod_num),
                        format!(
                            "Type derived for production {}: {}",
                            a.prod_num, a.prod_string
                        ),
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
                    Self::format_type(t, s, None, format!("Type derived for non-terminal {}", s))
                        .into_iter()
                        .for_each(|s| acc.push(s));
                    acc
                })
        } else {
            StrVec::new(0)
        };

        let ast_type_decl = if self.auto_generate {
            Self::format_type(
                &type_info.ast_enum_type,
                "ASTType",
                None,
                "Deduced ASTType of expanded grammar".to_string(),
            )
            .unwrap()
        } else {
            String::default()
        };

        let trait_functions = type_info.actions.iter().fold(
            Ok(StrVec::new(0).first_line_no_indent()),
            |acc: Result<StrVec>, a| {
                if let Ok(mut acc) = acc {
                    let fn_name = a.fn_name.clone();
                    let prod_string = a.prod_string.clone();
                    let fn_arguments = self.generate_argument_list(&a.args);
                    let user_trait_function_data = UserTraitFunctionDataBuilder::default()
                        .fn_name(fn_name)
                        .prod_num(a.prod_num)
                        .fn_arguments(fn_arguments)
                        .prod_string(prod_string)
                        .build()
                        .into_diagnostic()?;
                    acc.push(format!("{}", user_trait_function_data));
                    Ok(acc)
                } else {
                    acc
                }
            },
        )?;

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
            .user_trait_functions(StrVec::new(0))
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
