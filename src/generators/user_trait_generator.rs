use super::template_data::{
    UserTraitCallerFunctionDataBuilder, UserTraitDataBuilder, UserTraitFunctionDataBuilder,
};
use crate::generators::naming_helper::NamingHelper as NmHlp;
use crate::generators::{generate_terminal_name, GrammarConfig};
use crate::{Pr, StrVec, Symbol, Terminal};
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
    /// Compiled grammar configuration
    grammar_config: &'a GrammarConfig,
    /// Helper list for name generation
    terminals: Vec<&'a str>,
    /// Helper list for name generation
    terminal_names: Vec<String>,
}

impl<'a> UserTraitGenerator<'a> {
    fn generate_argument_list(&self, pr: &Pr) -> String {
        let get_terminal_index = |tr: &str| self.terminals.iter().position(|t| *t == tr).unwrap();
        let mut arguments = pr
            .get_r()
            .iter()
            .enumerate()
            .filter(|(_, s)| !s.is_switch())
            .map(|(i, a)| {
                let n = match a {
                    Symbol::N(n, _) => n,
                    Symbol::T(Terminal::Trm(t, _)) => &self.terminal_names[get_terminal_index(t)],
                    _ => panic!("Invalid symbol type in production!"),
                };
                format!(
                    "{}: &ParseTreeStackEntry",
                    NmHlp::to_lower_snake_case(&format!("_{}_{}", n, i)),
                )
            })
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

    // ---------------------------------------------------
    // Part of the Public API
    // *Changes will affect crate's version according to semver*
    // ---------------------------------------------------
    ///
    /// Generates the file with the user actions trait.
    ///
    pub fn generate_user_trait_source(&self) -> Result<String> {
        let scanner_state_resolver = self.grammar_config.get_scanner_state_resolver();

        let trait_functions = self.grammar_config.cfg.pr.iter().enumerate().fold(
            Ok(StrVec::new(0).first_line_no_indent()),
            |acc: Result<StrVec>, (i, p)| {
                if let Ok(mut acc) = acc {
                    let fn_name = NmHlp::to_lower_snake_case(&format!("{}_{}", p.get_n_str(), i));
                    let prod_string = p.format(&scanner_state_resolver)?;
                    let fn_arguments = self.generate_argument_list(p);
                    let user_trait_function_data = UserTraitFunctionDataBuilder::default()
                        .fn_name(fn_name)
                        .prod_num(i)
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
            .trait_functions(trait_functions)
            .trait_caller(trait_caller)
            .user_trait_module_name(self.module_name)
            .build()
            .into_diagnostic()?;

        Ok(format!("{}", user_trait_data))
    }

    /// Creates a new item
    pub fn try_new(
        user_type_name: &'a str,
        module_name: &'a str,
        auto_generate: bool,
        grammar_config: &'a GrammarConfig,
    ) -> Result<Self> {
        let user_type_name = NmHlp::to_upper_camel_case(user_type_name);
        let terminals = grammar_config
            .cfg
            .get_ordered_terminals()
            .iter()
            .map(|(t, _)| *t)
            .collect::<Vec<&str>>();
        let terminal_names = terminals.iter().fold(Vec::new(), |mut acc, e| {
            let n = generate_terminal_name(e, None, &grammar_config.cfg);
            acc.push(n);
            acc
        });
        UserTraitGeneratorBuilder::default()
            .user_type_name(user_type_name)
            .module_name(module_name)
            .auto_generate(auto_generate)
            .grammar_config(grammar_config)
            .terminals(terminals)
            .terminal_names(terminal_names)
            .build()
            .into_diagnostic()
    }
}
