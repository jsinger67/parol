use super::template_data::{
    UserTraitCallerFunctionDataBuilder, UserTraitDataBuilder, UserTraitFunctionDataBuilder,
};
use crate::generators::naming_helper::NamingHelper as NmHlp;
use crate::generators::{generate_terminal_name, GrammarConfig};
use crate::{Pr, StrVec, Symbol, Terminal};
use miette::{IntoDiagnostic, Result};

fn generate_argument_list(pr: &Pr, terminals: &[&str], terminal_names: &[String]) -> String {
    let get_terminal_index = |tr: &str| terminals.iter().position(|t| *t == tr).unwrap();
    let mut arguments = pr
        .get_r()
        .iter()
        .enumerate()
        .filter(|(_, s)| !s.is_switch())
        .map(|(i, a)| {
            let n = match a {
                Symbol::N(n) => n,
                Symbol::T(Terminal::Trm(t, _)) => &terminal_names[get_terminal_index(t)],
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
pub fn generate_user_trait_source(
    user_type_name: &str,
    user_trait_module_name: &str,
    grammar_config: &GrammarConfig,
) -> Result<String> {
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

    let scanner_state_resolver = |s: &[usize]| {
        s.iter()
            .map(|s| {
                grammar_config.scanner_configurations[*s]
                    .scanner_name
                    .clone()
            })
            .collect::<Vec<String>>()
            .join(", ")
    };

    let trait_functions = grammar_config.cfg.pr.iter().enumerate().fold(
        Ok(StrVec::new(0).first_line_no_indent()),
        |acc: Result<StrVec>, (i, p)| {
            if let Ok(mut acc) = acc {
                let fn_name = NmHlp::to_lower_snake_case(&format!("{}_{}", p.get_n_str(), i));
                let prod_string = p.format(&scanner_state_resolver);
                let fn_arguments = generate_argument_list(p, &terminals, &terminal_names);
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
    let trait_caller = grammar_config.cfg.pr.iter().enumerate().fold(
        Ok(StrVec::new(12)),
        |acc: Result<StrVec>, (i, p)| {
            if let Ok(mut acc) = acc {
                let fn_name = NmHlp::to_lower_snake_case(&format!("{}_{}", p.get_n_str(), i));
                let fn_arguments = generate_caller_argument_list(p);
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
        .user_type_name(user_type_name)
        .trait_functions(trait_functions)
        .trait_caller(trait_caller)
        .user_trait_module_name(user_trait_module_name)
        .build()
        .into_diagnostic()?;

    Ok(format!("{}", user_trait_data))
}
