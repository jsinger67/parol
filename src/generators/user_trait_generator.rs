use crate::errors::*;
use crate::generators::{generate_terminal_name, GrammarConfig};
use crate::{Pr, StrVec, Symbol, Terminal};

#[derive(BartDisplay, Debug, Default)]
#[template = "templates/user_trait_caller_function_template.rs"]
struct UserTraitCallerFunctionData {
    fn_name: String,
    prod_num: usize,
    fn_arguments: String,
}

#[derive(BartDisplay, Debug, Default)]
#[template = "templates/user_trait_function_template.rs"]
struct UserTraitFunctionData {
    fn_name: String,
    prod_num: usize,
    fn_arguments: String,
    prod_string: String,
}

#[derive(BartDisplay, Debug, Default)]
#[template = "templates/user_trait_template.rs"]
struct UserTraitData<'a> {
    user_type_name: &'a str,
    trait_functions: StrVec,
    trait_caller: StrVec,
    user_trait_module_name: &'a str,
}

fn to_camel_case(name: &str) -> String {
    name.chars().fold(String::new(), |mut acc, c| {
        if acc.is_empty() {
            acc.push(c.to_lowercase().next().unwrap())
        } else if c.is_uppercase() {
            acc.push('_');
            acc.push(c.to_lowercase().next().unwrap())
        } else {
            acc.push(c);
        }
        acc
    })
}

fn generate_argument_list(pr: &Pr, terminals: &[&str], terminal_names: &[String]) -> String {
    let get_terminal_index = |tr: &str| terminals.iter().position(|t| *t == tr).unwrap();
    let mut arguments = pr
        .get_r()
        .iter()
        .enumerate()
        .map(|(i, a)| match a {
            Symbol::N(n) => {
                format!("_{}_{}: &ParseTreeStackEntry", to_camel_case(n), i)
            }
            Symbol::T(Terminal::Trm(t, _)) => {
                let terminal_name = &terminal_names[get_terminal_index(t)];
                format!(
                    "_{}_{}: &ParseTreeStackEntry",
                    to_camel_case(terminal_name),
                    i
                )
            }
            _ => panic!("Invalid symbol type in production!"),
        })
        .collect::<Vec<String>>();
    arguments.push("_parse_tree: &Tree<ParseTreeType>".to_string());
    arguments.push("mut _scanner_access: RefMut<dyn ScannerAccess>".to_string());
    arguments.join(", ")
}

fn generate_caller_argument_list(pr: &Pr) -> String {
    let mut arguments = (0..pr.get_r().len())
        .map(|i| format!("&children[{}]", i))
        .collect::<Vec<String>>();
    arguments.push("parse_tree".to_string());
    arguments.push("scanner_access".to_string());
    arguments.join(", ")
}

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
    let terminal_names = terminals
        .iter()
        .fold(Vec::new(), |mut acc, e| {
            let n = generate_terminal_name(e, usize::MAX, &grammar_config.cfg);
            acc.push(n);
            acc
        });

    let trait_functions = grammar_config.cfg.pr.iter().enumerate().fold(
        StrVec::new(0).first_line_no_indent(),
        |mut acc, (i, p)| {
            let fn_name = to_camel_case(p.get_n_str());
            let prod_string = format!("{}", p);
            let fn_arguments = generate_argument_list(p, &terminals, &terminal_names);
            let user_trait_function_data = UserTraitFunctionData {
                fn_name,
                prod_num: i,
                fn_arguments,
                prod_string,
            };
            acc.push(format!("{}", user_trait_function_data));
            acc
        },
    );
    let trait_caller =
        grammar_config
            .cfg
            .pr
            .iter()
            .enumerate()
            .fold(StrVec::new(12), |mut acc, (i, p)| {
                let fn_name = to_camel_case(p.get_n_str());
                let fn_arguments = generate_caller_argument_list(p);
                let user_trait_function_data = UserTraitCallerFunctionData {
                    fn_name,
                    prod_num: i,
                    fn_arguments,
                };
                acc.push(format!("{}", user_trait_function_data));
                acc
            });
    let user_trait_data = UserTraitData {
        user_type_name,
        trait_functions,
        trait_caller,
        user_trait_module_name,
    };
    //Ok(formatted_or_unchanged(format!("{}", user_trait_data)))
    Ok(format!("{}", user_trait_data))
}
