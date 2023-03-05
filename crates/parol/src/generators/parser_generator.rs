use crate::analysis::compiled_la_dfa::CompiledDFA;
use crate::analysis::LookaheadDFA;
use crate::config::{CommonGeneratorConfig, ParserGeneratorConfig};
use crate::conversions::dot::render_dfa_dot_string;
use crate::generators::GrammarConfig;
use crate::{Pr, Symbol, Terminal};
use anyhow::{anyhow, Result};
use parol_runtime::lexer::FIRST_USER_TOKEN;
use parol_runtime::log::trace;
use std::collections::{BTreeMap, BTreeSet};

use crate::StrVec;
use std::fmt::Debug;

#[derive(Debug, Default)]
struct Dfa {
    states: StrVec,
    transitions: StrVec,
    k: usize,
    nt_index: usize,
    nt_name: String,
}

impl Dfa {
    fn from_la_dfa(la_dfa: &LookaheadDFA, nt_index: usize, nt_name: String) -> Self {
        let compiled_dfa = CompiledDFA::from_lookahead_dfa(la_dfa);
        let states =
            compiled_dfa
                .states
                .iter()
                .fold(StrVec::new(4).first_line_no_indent(), |mut acc, s| {
                    acc.push(format!("{:?},", s));
                    acc
                });
        let transitions = compiled_dfa.transitions.iter().fold(
            StrVec::new(4).first_line_no_indent(),
            |mut acc, t| {
                acc.push(format!("DFATransition{:?},", t));
                acc
            },
        );
        let k = compiled_dfa.k;

        Self {
            states,
            transitions,
            k,
            nt_index,
            nt_name,
        }
    }
}

impl std::fmt::Display for Dfa {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Dfa {
            states,
            transitions,
            k,
            nt_index,
            nt_name,
        } = self;
        writeln!(f, r#"/* {nt_index} - "{nt_name}" */"#)?;
        f.write_fmt(ume::ume! {
            LookaheadDFA {
                states: &[#states],
                transitions: &[#transitions],
                k: #k,
            },
        })
    }
}

#[derive(Debug, Default)]
struct Dfas {
    dfa_count: usize,
    lookahead_dfa_s: String,
}

impl std::fmt::Display for Dfas {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Dfas {
            dfa_count,
            lookahead_dfa_s,
        } = self;
        f.write_fmt(ume::ume! {
            pub const LOOKAHEAD_AUTOMATA: &[LookaheadDFA; #dfa_count] = &[
            #lookahead_dfa_s];
        })
    }
}

#[derive(Debug, Default)]
struct Production {
    lhs: usize,
    production: StrVec,
    prod_num: usize,
    prod_string: String,
}

impl Production {
    fn from_cfg_production(
        pr: &Pr,
        prod_num: usize,
        non_terminals: &[&str],
        terminals: &[&str],
    ) -> Self {
        let get_non_terminal_index =
            |nt: &str| non_terminals.iter().position(|n| *n == nt).unwrap();
        let get_terminal_index =
            |tr: &str| terminals.iter().position(|t| *t == tr).unwrap() + FIRST_USER_TOKEN;
        let lhs = get_non_terminal_index(pr.get_n_str());
        let production =
            pr.get_r()
                .iter()
                .rev()
                .fold(StrVec::new(4).first_line_no_indent(), |mut acc, s| {
                    match s {
                        Symbol::N(n, ..) => {
                            acc.push(format!("ParseType::N({}),", get_non_terminal_index(n)))
                        }
                        Symbol::T(Terminal::Trm(t, ..)) => {
                            acc.push(format!("ParseType::T({}),", get_terminal_index(t)))
                        }
                        Symbol::S(s) => acc.push(format!("ParseType::S({}),", s)),
                        Symbol::Push(s) => acc.push(format!("ParseType::Push({}),", s)),
                        Symbol::Pop => acc.push("ParseType::Pop,".to_string()),
                        _ => panic!("Unexpected symbol type in production!"),
                    }
                    acc
                });
        let prod_string = format!("{}", pr);
        Self {
            lhs,
            production,
            prod_num,
            prod_string,
        }
    }
}

impl std::fmt::Display for Production {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Production {
            lhs,
            production,
            prod_num,
            prod_string,
        } = self;
        writeln!(f, "// {prod_num} - {prod_string}")?;
        f.write_fmt(ume::ume! {
            Production {
                lhs: #lhs,
                production: &[#production],
            },
        })?;
        writeln!(f)
    }
}

#[derive(Debug, Default)]
struct Productions {
    production_count: usize,
    productions: String,
}

impl std::fmt::Display for Productions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Productions {
            production_count,
            productions,
        } = self;
        f.write_fmt(ume::ume! {
            pub const PRODUCTIONS: &[Production; #production_count] = &[
            #productions];
        })
    }
}

#[derive(Debug, Default)]
struct ParserData<'a> {
    start_symbol_index: usize,
    lexer_source: &'a str,
    non_terminals: StrVec,
    non_terminal_count: usize,
    dfa_source: String,
    productions: String,
    max_k: usize,
    scanner_builds: StrVec,
    auto_generate: bool,
    user_type_name: &'a str,
    user_type_life_time: &'static str,
    module_name: &'a str,
    trim_parse_tree: bool,
}

impl std::fmt::Display for ParserData<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ParserData {
            start_symbol_index,
            lexer_source,
            non_terminals,
            non_terminal_count,
            dfa_source,
            productions,
            max_k,
            scanner_builds,
            auto_generate,
            user_type_name,
            user_type_life_time,
            module_name,
            trim_parse_tree,
        } = self;

        writeln!(
            f,
            "
            // ---------------------------------------------------------
            // This file was generated by parol.
            // It is not intended for manual editing and changes will be
            // lost after next build.
            // ---------------------------------------------------------
            "
        )?;

        let user_action_trait = if *auto_generate {
            "".into()
        } else {
            ume::ume!(UserActionsTrait,).to_string()
        };
        f.write_fmt(ume::ume! {
            use parol_runtime::{TokenStream, Tokenizer};
            use parol_runtime::once_cell::sync::Lazy;
            use parol_runtime::{ParolError, ParseTree};
            #[allow(unused_imports)]
            use parol_runtime::parser::{
                ParseTreeType, DFATransition, LLKParser, LookaheadDFA, ParseType, Production, #user_action_trait
            };
            use std::cell::RefCell;
            use std::path::Path;
        })?;

        writeln!(f, "\n")?;
        let auto_name = format!("{}Auto", user_type_name);
        if *auto_generate {
            let trait_module_name = format!("{}_trait", module_name);
            f.write_fmt(ume::ume! {
                use crate::#module_name::#user_type_name;
                use crate::#trait_module_name::#auto_name;
            })?;
            writeln!(f, "\n")?;
        }

        writeln!(f, "{}\n", lexer_source)?;

        f.write_fmt(ume::ume! {
            const MAX_K: usize = #max_k;
        })?;
        writeln!(f, "\n\n")?;
        f.write_fmt(ume::ume! {
            pub const NON_TERMINALS: &[&str; #non_terminal_count] = &[#non_terminals];
        })?;

        writeln!(f, "\n\n{}", dfa_source)?;
        writeln!(f, "\n{}\n", productions)?;

        f.write_fmt(ume::ume! {
            static TOKENIZERS: Lazy<Vec<(&'static str, Tokenizer)>> = Lazy::new(|| vec![
                ("INITIAL", Tokenizer::build(TERMINALS, SCANNER_0.0, SCANNER_0.1).unwrap()),
                #scanner_builds
            ]);
        })?;

        writeln!(f, "\n")?;

        let user_actions = if *auto_generate {
            ume::ume!(&mut #user_type_name #user_type_life_time).to_string()
        } else {
            ume::ume!(&mut dyn UserActionsTrait<'t>).to_string()
        };
        let auto_wrapper = if *auto_generate {
            format!(
                "// Initialize wrapper\n{}\n",
                ume::ume! {
                    let mut user_actions = #auto_name::new(user_actions);
                }
            )
        } else {
            "".into()
        };
        let mut_ref_user_actions = if *auto_generate {
            ume::ume!(&mut user_actions)
        } else {
            ume::ume!(user_actions)
        };
        let enable_trimming = if *trim_parse_tree {
            "llk_parser.trim_parse_tree();\n"
        } else {
            ""
        };
        f.write_fmt(ume::ume! {
            pub fn parse<'t, T>(
                input: &'t str,
                file_name: T,
                user_actions: #user_actions,
            ) -> Result<ParseTree<'t>, ParolError> where T: AsRef<Path> {
                let mut llk_parser = LLKParser::new(
                    #start_symbol_index,
                    LOOKAHEAD_AUTOMATA,
                    PRODUCTIONS,
                    TERMINAL_NAMES,
                    NON_TERMINALS,
                );
                #enable_trimming
                let token_stream = RefCell::new(
                    TokenStream::new(input, file_name, &TOKENIZERS, MAX_K).unwrap(),
                );
                #auto_wrapper
                llk_parser.parse(token_stream, #mut_ref_user_actions)
            }
        })
    }
}

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
///
/// Generates the parser part of the parser output file.
///
pub fn generate_parser_source<C: CommonGeneratorConfig + ParserGeneratorConfig>(
    grammar_config: &GrammarConfig,
    lexer_source: &str,
    config: &C,
    la_dfa: &BTreeMap<String, LookaheadDFA>,
    ast_type_has_lifetime: bool,
) -> Result<String> {
    let terminals = grammar_config
        .cfg
        .get_ordered_terminals()
        .iter()
        .map(|(t, _, _)| *t)
        .collect::<Vec<&str>>();
    let original_non_terminals = grammar_config.cfg.get_non_terminal_set();
    let non_terminal_count = original_non_terminals.len();
    let width = (non_terminal_count as f32).log10() as usize + 1;

    let non_terminals = original_non_terminals.iter().collect::<Vec<_>>();
    let start_symbol_index: usize = non_terminals
        .iter()
        .position(|n| *n == grammar_config.cfg.get_start_symbol())
        .ok_or_else(|| {
            anyhow!(
                "Start symbol '{}' is not part of the given grammar!",
                grammar_config.cfg.get_start_symbol()
            )
        })?;

    let non_terminals = non_terminals
        .iter()
        .enumerate()
        .fold(StrVec::new(4), |mut acc, (i, n)| {
            acc.push(format!(r#"/* {:w$} */ "{}","#, i, n, w = width));
            acc
        });

    let dfa_source = generate_dfa_source(la_dfa);

    let productions = generate_productions(grammar_config, &original_non_terminals, &terminals);

    let max_k = grammar_config.lookahead_size;

    let scanner_builds = grammar_config
        .scanner_configurations
        .iter()
        .enumerate()
        .skip(1)
        .fold(StrVec::new(8), |mut acc, (i, e)| {
            acc.push(format!(
                r#"("{}", Tokenizer::build(TERMINALS, SCANNER_{}.0, SCANNER_{}.1).unwrap()),"#,
                e.scanner_name, i, i
            ));
            acc
        });

    let user_type_life_time = if ast_type_has_lifetime { "<'t>" } else { "" };

    let parser_data = ParserData {
        start_symbol_index,
        lexer_source,
        non_terminals,
        non_terminal_count,
        dfa_source,
        productions,
        max_k,
        scanner_builds,
        auto_generate: config.auto_generate(),
        user_type_name: config.user_type_name(),
        user_type_life_time,
        module_name: config.module_name(),
        trim_parse_tree: config.trim_parse_tree(),
    };

    Ok(format!("{}", parser_data))
}

fn generate_dfa_source(la_dfa: &BTreeMap<String, LookaheadDFA>) -> String {
    let lookahead_dfa_s = la_dfa
        .iter()
        .enumerate()
        .fold(StrVec::new(0), |mut acc, (i, (n, d))| {
            trace!("{}", render_dfa_dot_string(d, n));
            let dfa = Dfa::from_la_dfa(d, i, n.clone());
            acc.push(format!("{}", dfa));
            acc
        });
    let dfa_count = la_dfa.len();

    let dfas = Dfas {
        dfa_count,
        lookahead_dfa_s: format!("{}", lookahead_dfa_s),
    };

    format!("{}", dfas)
}

fn generate_productions(
    grammar_config: &GrammarConfig,
    non_terminals: &BTreeSet<String>,
    terminals: &[&str],
) -> String {
    let non_terminals = non_terminals
        .iter()
        .map(|n| n.as_str())
        .collect::<Vec<&str>>();
    let production_count = grammar_config.cfg.pr.len();
    let productions =
        grammar_config
            .cfg
            .pr
            .iter()
            .enumerate()
            .fold(String::new(), |mut acc, (i, p)| {
                let production = Production::from_cfg_production(p, i, &non_terminals, terminals);
                acc.push_str(format!("{}", production).as_str());
                acc
            });

    let productions = Productions {
        production_count,
        productions,
    };

    format!("{}", productions)
}
