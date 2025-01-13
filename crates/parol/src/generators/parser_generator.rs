use crate::analysis::compiled_la_dfa::CompiledDFA;
use crate::analysis::lalr1_parse_table::LR1State;
use crate::analysis::lookahead_dfa::CompiledProductionIndex;
use crate::analysis::LookaheadDFA;
use crate::config::{CommonGeneratorConfig, ParserGeneratorConfig};
use crate::conversions::dot::render_dfa_dot_string;
use crate::generators::GrammarConfig;
use crate::{LRAction, LRParseTable, Pr, Symbol, Terminal};
use anyhow::{anyhow, Result};
use parol_runtime::lexer::{
    BLOCK_COMMENT, EOI, FIRST_USER_TOKEN, LINE_COMMENT, NEW_LINE, WHITESPACE,
};
use parol_runtime::log::trace;
use parol_runtime::{NonTerminalIndex, TerminalIndex};
use std::collections::{BTreeMap, BTreeSet};

use crate::StrVec;
use std::fmt::Debug;

#[derive(Debug, Default)]
pub(crate) struct Dfa {
    prod0: CompiledProductionIndex,
    transitions: StrVec,
    k: usize,
    nt_index: usize,
    nt_name: String,
}

impl Dfa {
    fn from_la_dfa(la_dfa: &LookaheadDFA, nt_index: usize, nt_name: String) -> Self {
        let compiled_dfa = CompiledDFA::from_lookahead_dfa(la_dfa);
        Dfa::from_compiled_dfa(compiled_dfa, nt_index, nt_name)
    }

    pub(crate) fn from_compiled_dfa(
        compiled_dfa: CompiledDFA,
        nt_index: usize,
        nt_name: String,
    ) -> Dfa {
        let prod0 = compiled_dfa.prod0;
        let transitions = compiled_dfa.transitions.iter().fold(
            StrVec::new(4).first_line_no_indent(),
            |mut acc, t| {
                acc.push(format!(
                    "Trans({}, {}, {}, {}),",
                    t.from_state, t.term, t.to_state, t.prod_num
                ));
                acc
            },
        );
        let k = compiled_dfa.k;

        Self {
            prod0,
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
            prod0,
            transitions,
            k,
            nt_index,
            nt_name,
        } = self;
        writeln!(f, r#"/* {nt_index} - "{nt_name}" */"#)?;
        f.write_fmt(ume::ume! {
            LookaheadDFA {
                prod0: #prod0,
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
        let get_terminal_index = |tr: &str| {
            terminals.iter().position(|t| *t == tr).unwrap() as TerminalIndex + FIRST_USER_TOKEN
        };
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
struct LRProduction {
    lhs: usize,
    len: usize,
    prod_num: usize,
    prod_string: String,
}

impl LRProduction {
    fn from_cfg_production(pr: &Pr, prod_num: usize, non_terminals: &[&str]) -> Self {
        let get_non_terminal_index =
            |nt: &str| non_terminals.iter().position(|n| *n == nt).unwrap();
        let lhs = get_non_terminal_index(pr.get_n_str());
        let len = pr.get_r().iter().count();
        let prod_string = format!("{}", pr);
        Self {
            lhs,
            len,
            prod_num,
            prod_string,
        }
    }
}

impl std::fmt::Display for LRProduction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let LRProduction {
            lhs,
            len,
            prod_num,
            prod_string,
        } = self;
        writeln!(f, "// {prod_num} - {prod_string}")?;
        f.write_fmt(ume::ume! {
            LRProduction {
                lhs: #lhs,
                len: #len,
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
struct LRProductions {
    production_count: usize,
    productions: String,
}

impl std::fmt::Display for LRProductions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let LRProductions {
            production_count,
            productions,
        } = self;
        f.write_fmt(ume::ume! {
            pub const PRODUCTIONS: &[LRProduction; #production_count] = &[
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
    user_type_name: &'a str,
    user_type_life_time: &'static str,
    module_name: &'a str,
    trim_parse_tree: bool,
    disable_recovery: bool,
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
            user_type_name,
            user_type_life_time,
            module_name,
            trim_parse_tree,
            disable_recovery,
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

        f.write_fmt(ume::ume! {
            use parol_runtime::{ScannerConfig, TokenStream, Tokenizer};
            use parol_runtime::once_cell::sync::Lazy;
            use parol_runtime::{ParolError, ParseTree, TerminalIndex};
            use parol_runtime::parser::parse_tree_type::SynTreeNode;
            #[allow(unused_imports)]
            use parol_runtime::parser::{
                Trans, LLKParser, LookaheadDFA, ParseType, Production
            };
            use std::path::Path;
        })?;

        writeln!(f, "\n")?;
        let auto_name = format!("{}Auto", user_type_name);
        let trait_module_name = format!("{}_trait", module_name);
        f.write_fmt(ume::ume! {
            use crate::#module_name::#user_type_name;
            use crate::#trait_module_name::#auto_name;
        })?;
        writeln!(f, "\n")?;

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
            static SCANNERS: Lazy<Vec<ScannerConfig>> = Lazy::new(|| vec![
                #scanner_builds
            ]);
        })?;

        writeln!(f, "\n")?;

        let user_actions = ume::ume!(&mut #user_type_name #user_type_life_time).to_string();
        let lifetime_on_parse = if *user_type_life_time == "<'t>" {
            "'t,"
        } else {
            ""
        };
        let lifetime_on_input = if *user_type_life_time == "<'t>" {
            "'t"
        } else {
            ""
        };
        let auto_wrapper = format!(
            "\n// Initialize wrapper\n{}",
            ume::ume! {
                let mut user_actions = #auto_name::new(user_actions);
            }
        );
        let mut_ref_user_actions = ume::ume!(&mut user_actions);
        let enable_trimming = if *trim_parse_tree {
            "llk_parser.trim_parse_tree();\n"
        } else {
            ""
        };
        let recovery = if *disable_recovery {
            "llk_parser.disable_recovery();\n"
        } else {
            ""
        };
        f.write_fmt(ume::ume! {
            pub fn parse<#lifetime_on_parse T>(
                input: &#lifetime_on_input str,
                file_name: T,
                user_actions: #user_actions,
            ) -> Result<ParseTree, ParolError> where T: AsRef<Path> {
                let mut llk_parser = LLKParser::new(
                    #start_symbol_index,
                    LOOKAHEAD_AUTOMATA,
                    PRODUCTIONS,
                    TERMINAL_NAMES,
                    NON_TERMINALS,
                );
                #enable_trimming
                #recovery
                #auto_wrapper
                llk_parser.parse(TokenStream::new(input, file_name, &SCANNERS, MAX_K).unwrap(),
                    #mut_ref_user_actions)
            }
        })?;
        f.write_fmt(ume::ume! {
            #[allow(dead_code)]
            pub fn parse2<'t, T: SynTreeNode<'t>>(
                input: &'t str,
                file_name: impl AsRef<Path>,
                user_actions: #user_actions,
            ) -> Result<ParseTree<T>, ParolError> {
                let mut llk_parser = LLKParser::new(
                    #start_symbol_index,
                    LOOKAHEAD_AUTOMATA,
                    PRODUCTIONS,
                    TERMINAL_NAMES,
                    NON_TERMINALS,
                );
                #enable_trimming
                #recovery
                #auto_wrapper
                llk_parser.parse::<T>(TokenStream::new(input, file_name, &SCANNERS, MAX_K).unwrap(),
                    #mut_ref_user_actions)
            }
        })
    }
}

#[derive(Debug, Default)]
struct LRParserData<'a> {
    start_symbol_index: usize,
    lexer_source: &'a str,
    non_terminals: StrVec,
    non_terminal_count: usize,
    productions: String,
    scanner_builds: StrVec,
    user_type_name: &'a str,
    user_type_life_time: &'static str,
    module_name: &'a str,
    trim_parse_tree: bool,
    parse_table_source: String,
}

impl std::fmt::Display for LRParserData<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let LRParserData {
            start_symbol_index,
            lexer_source,
            non_terminals,
            non_terminal_count,
            productions,
            scanner_builds,
            user_type_name,
            user_type_life_time,
            module_name,
            trim_parse_tree,
            parse_table_source,
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

        f.write_fmt(ume::ume! {
            use parol_runtime::{ScannerConfig, TokenStream, Tokenizer};
            use parol_runtime::once_cell::sync::Lazy;
            use parol_runtime::{ParolError, ParseTree, TerminalIndex};
            use parol_runtime::parser::parse_tree_type::SynTreeNode;
            #[allow(unused_imports)]
            use parol_runtime::parser::{Trans, ParseType, Production};
            use parol_runtime::lr_parser::{LRParseTable, LRParser, LRProduction, LR1State, LRAction};
            use std::path::Path;
        })?;

        writeln!(f, "\n")?;
        let auto_name = format!("{}Auto", user_type_name);
        let trait_module_name = format!("{}_trait", module_name);
        f.write_fmt(ume::ume! {
            use crate::#module_name::#user_type_name;
            use crate::#trait_module_name::#auto_name;
        })?;
        writeln!(f, "\n")?;

        writeln!(f, "{}\n", lexer_source)?;

        writeln!(f, "\n\n")?;
        f.write_fmt(ume::ume! {
            pub const NON_TERMINALS: &[&str; #non_terminal_count] = &[#non_terminals];
        })?;

        writeln!(
            f,
            "\n\nstatic PARSE_TABLE: LRParseTable  = {};\n",
            parse_table_source
        )?;
        writeln!(f, "\n{}\n", productions)?;

        f.write_fmt(ume::ume! {
            static SCANNERS: Lazy<Vec<ScannerConfig>> = Lazy::new(|| vec![
                #scanner_builds
            ]);
        })?;

        writeln!(f, "\n")?;

        let user_actions = ume::ume!(&mut #user_type_name #user_type_life_time).to_string();
        let lifetime_on_parse = if *user_type_life_time == "<'t>" {
            "'t,"
        } else {
            ""
        };
        let lifetime_on_input = if *user_type_life_time == "<'t>" {
            "'t"
        } else {
            ""
        };
        let auto_wrapper = format!(
            "\n// Initialize wrapper\n{}",
            ume::ume! {
                let mut user_actions = #auto_name::new(user_actions);
            }
        );
        let mut_ref_user_actions = ume::ume!(&mut user_actions);
        let enable_trimming = if *trim_parse_tree {
            "lr_parser.trim_parse_tree();\n"
        } else {
            ""
        };

        f.write_fmt(ume::ume! {
            pub fn parse<#lifetime_on_parse T>(
                input: &#lifetime_on_input str,
                file_name: T,
                user_actions: #user_actions,
            ) -> Result<ParseTree, ParolError> where T: AsRef<Path> {
                let mut lr_parser = LRParser::new(
                    #start_symbol_index,
                    &PARSE_TABLE,
                    PRODUCTIONS,
                    TERMINAL_NAMES,
                    NON_TERMINALS,
                );
                #enable_trimming
                #auto_wrapper
                lr_parser.parse(TokenStream::new(input, file_name, &SCANNERS, 1).unwrap(),
                    #mut_ref_user_actions)
            }
        })?;
        f.write_fmt(ume::ume! {
            #[allow(dead_code)]
            pub fn parse2<'t, T: SynTreeNode<'t>>(
                input: &'t str,
                file_name: impl AsRef<Path>,
                user_actions: #user_actions,
            ) -> Result<ParseTree<T>, ParolError> {
                let mut lr_parser = LRParser::new(
                    #start_symbol_index,
                    &PARSE_TABLE,
                    PRODUCTIONS,
                    TERMINAL_NAMES,
                    NON_TERMINALS,
                );
                #enable_trimming
                #auto_wrapper
                lr_parser.parse::<T>(TokenStream::new(input, file_name, &SCANNERS, 1).unwrap(),
                    #mut_ref_user_actions)
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
    let terminals = get_terminals(grammar_config);
    let original_non_terminals = grammar_config.cfg.get_non_terminal_set();
    let non_terminal_count = original_non_terminals.len();
    let width = (non_terminal_count as f32).log10() as usize + 1;

    let non_terminals = original_non_terminals.iter().collect::<Vec<_>>();
    let start_symbol_index: usize =
        find_start_symbol_index(non_terminals.as_slice(), grammar_config)?;

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

    let scanner_builds = generate_scanner_builds(grammar_config);

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
        user_type_name: config.user_type_name(),
        user_type_life_time,
        module_name: config.module_name(),
        trim_parse_tree: config.trim_parse_tree(),
        disable_recovery: config.recovery_disabled(),
    };

    Ok(format!("{}", parser_data))
}

fn generate_scanner_builds(grammar_config: &GrammarConfig) -> StrVec {
    let primary_non_terminal_finder = grammar_config.cfg.get_primary_non_terminal_finder();
    let scanner_state_resolver = grammar_config.get_scanner_state_resolver();
    grammar_config
        .scanner_configurations
        .iter()
        .enumerate()
        .fold(StrVec::new(0), |mut acc, (i, e)| {
            let transitions = e.transitions.iter().fold(StrVec::new(4), |mut acc, t| {
                acc.push(format!(r#"({} /* {} */, {} /* {} */),"#, t.0, primary_non_terminal_finder(t.0).unwrap_or("".to_string()), t.1, scanner_state_resolver(&[t.1])));
                acc
            });
            acc.push(format!(
                r#"ScannerConfig::new("{}", Tokenizer::build(TERMINALS, SCANNER_{}.0, SCANNER_{}.1).unwrap(), &[{}]),"#,
                e.scanner_name, i, i, transitions
            ));
            acc
        })
}

fn get_terminals(grammar_config: &GrammarConfig) -> Vec<&str> {
    grammar_config
        .cfg
        .get_ordered_terminals()
        .iter()
        .map(|(t, _, _, _)| *t)
        .collect::<Vec<&str>>()
}

fn find_start_symbol_index(
    non_terminals: &[&String],
    grammar_config: &GrammarConfig,
) -> Result<usize, anyhow::Error> {
    non_terminals
        .iter()
        .position(|n| *n == grammar_config.cfg.get_start_symbol())
        .ok_or_else(|| {
            anyhow!(
                "Start symbol '{}' is not part of the given grammar!",
                grammar_config.cfg.get_start_symbol()
            )
        })
}

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
///
/// Generates the parser part of the parser output file in case of LALR(1) parser.
///
pub fn generate_lalr1_parser_source<C: CommonGeneratorConfig + ParserGeneratorConfig>(
    grammar_config: &GrammarConfig,
    lexer_source: &str,
    config: &C,
    parse_table: &LRParseTable,
    ast_type_has_lifetime: bool,
) -> Result<String> {
    let terminals = get_terminals(grammar_config);
    let original_non_terminals = grammar_config.cfg.get_non_terminal_set();
    let non_terminal_count = original_non_terminals.len();
    let width = (non_terminal_count as f32).log10() as usize + 1;

    let non_terminals = original_non_terminals.iter().collect::<Vec<_>>();
    let start_symbol_index: usize = find_start_symbol_index(&non_terminals, grammar_config)?;

    let non_terminals_with_index_comment =
        non_terminals
            .iter()
            .enumerate()
            .fold(StrVec::new(4), |mut acc, (i, n)| {
                acc.push(format!(r#"/* {:w$} */ "{}","#, i, n, w = width));
                acc
            });
    let productions = generate_lr_productions(grammar_config, &original_non_terminals);
    let scanner_builds = generate_scanner_builds(grammar_config);

    let user_type_life_time = if ast_type_has_lifetime { "<'t>" } else { "" };

    let parse_table_source = generate_parse_table_source(parse_table, &terminals, &non_terminals);

    let parser_data = LRParserData {
        start_symbol_index,
        lexer_source,
        non_terminals: non_terminals_with_index_comment,
        non_terminal_count,
        productions,
        scanner_builds,
        user_type_name: config.user_type_name(),
        user_type_life_time,
        module_name: config.module_name(),
        trim_parse_tree: config.trim_parse_tree(),
        parse_table_source,
    };

    Ok(format!("{}", parser_data))
}

fn generate_parse_table_source(
    parse_table: &LRParseTable,
    terminals: &[&str],
    non_terminals: &[&String],
) -> String {
    // Create a terminal resolver function
    let tr = |ti: TerminalIndex| {
        if ti >= FIRST_USER_TOKEN {
            terminals[(ti - FIRST_USER_TOKEN) as usize]
        } else {
            match ti {
                EOI => "<$>",
                NEW_LINE => "<NL>",
                WHITESPACE => "<WS>",
                LINE_COMMENT => "<LC>",
                BLOCK_COMMENT => "<BC>",
                _ => unreachable!(),
            }
        }
    };

    // Create a non-terminal resolver function
    let nr = |ni: usize| non_terminals[ni].as_str();

    let actions = parse_table
        .states
        .iter()
        .fold(BTreeSet::<LRAction>::new(), |mut acc, s| {
            s.actions.iter().for_each(|(_, a)| {
                acc.insert(a.clone());
            });
            acc
        });

    // Sorted array of actions
    let actions_array = actions.iter().cloned().collect::<Vec<_>>();

    let actions = actions_array
        .iter()
        .enumerate()
        .fold(String::new(), |mut acc, (i, a)| {
            acc.push_str(format!("/* {} */ {}, ", i, generate_source_for_action(a, nr)).as_str());
            acc
        });

    let states = parse_table
        .states
        .iter()
        .enumerate()
        .fold(String::new(), |mut acc, (i, s)| {
            acc.push_str(&generate_source_for_lrstate(s, i, &actions_array, &tr, &nr));
            acc.push(',');
            acc
        });

    format!(
        "LRParseTable {{ actions: &[{}], states: &[{}] }}",
        actions, states,
    )
}

fn generate_source_for_lrstate<'a>(
    state: &'a LR1State,
    state_num: usize,
    actions_array: &[LRAction],
    tr: &impl Fn(TerminalIndex) -> &'a str,
    nr: &impl Fn(NonTerminalIndex) -> &'a str,
) -> String {
    format!(
        r#"
        // State {}
        LR1State {{
            actions: {},
            gotos: {} }}"#,
        state_num,
        generate_source_for_actions(state, actions_array, tr, nr),
        generate_source_for_gotos(state, nr)
    )
}

fn generate_source_for_actions<'a>(
    state: &LR1State,
    actions_array: &[LRAction],
    tr: &impl Fn(TerminalIndex) -> &'a str,
    nr: &impl Fn(NonTerminalIndex) -> &'a str,
) -> String {
    format!(
        r#"&[{}]"#,
        state
            .actions
            .iter()
            .map(|(t, a)| {
                format!(
                    r#"
        ({}, {}) /* '{}' => {} */"#,
                    t,
                    generate_source_for_action_ref(a, actions_array),
                    tr(*t),
                    generate_action_comment(a, nr)
                )
            })
            .collect::<Vec<String>>()
            .join(", ")
    )
}

fn generate_source_for_gotos<'a>(
    state: &LR1State,
    nr: &impl Fn(NonTerminalIndex) -> &'a str,
) -> String {
    if state.gotos.is_empty() {
        return "&[]".to_string();
    }
    format!(
        r#"&[{}]"#,
        state
            .gotos
            .iter()
            .map(|(n, s)| {
                format!(
                    r#"
                ({}, {}) /* {} => {} */"#,
                    n,
                    s,
                    nr(*n),
                    s,
                )
            })
            .collect::<Vec<String>>()
            .join(", ")
    )
}

fn generate_source_for_action<'a>(
    action: &LRAction,
    nr: impl Fn(NonTerminalIndex) -> &'a str,
) -> String {
    match action {
        LRAction::Shift(s) => format!("LRAction::Shift({})", s),
        LRAction::Reduce(n, p) => format!("LRAction::Reduce({} /* {} */, {})", n, nr(*n), p),
        LRAction::Accept => "LRAction::Accept".to_string(),
    }
}

fn generate_source_for_action_ref(action: &LRAction, actions_array: &[LRAction]) -> String {
    let index = actions_array.iter().position(|a| a == action).unwrap();
    format!("{}", index)
}

fn generate_action_comment<'a>(
    action: &LRAction,
    nr: impl Fn(NonTerminalIndex) -> &'a str,
) -> String {
    match action {
        LRAction::Shift(s) => format!("LRAction::Shift({})", s),
        LRAction::Reduce(n, p) => format!("LRAction::Reduce({}, {})", nr(*n), p),
        LRAction::Accept => "LRAction::Accept".to_string(),
    }
}

fn generate_dfa_source(la_dfa: &BTreeMap<String, LookaheadDFA>) -> String {
    let lookahead_dfa_s = la_dfa
        .iter()
        .enumerate()
        .fold(StrVec::new(0), |mut acc, (i, (n, d))| {
            trace!("{}", d);
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

fn generate_lr_productions(
    grammar_config: &GrammarConfig,
    non_terminals: &BTreeSet<String>,
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
                let production = LRProduction::from_cfg_production(p, i, &non_terminals);
                acc.push_str(format!("{}", production).as_str());
                acc
            });

    let productions = LRProductions {
        production_count,
        productions,
    };

    format!("{}", productions)
}
