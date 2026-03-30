use crate::LRParseTable;
use crate::analysis::LookaheadDFA;
use crate::analysis::compiled_la_dfa::CompiledDFA;
use crate::analysis::lookahead_dfa::CompiledProductionIndex;
use crate::config::{CommonGeneratorConfig, ParserGeneratorConfig};
use crate::generators::parser_model::{
    LookaheadAutomatonModel as LookaheadAutomatonIR, ProductionModel as ProductionIR,
    ProductionSymbolModel as ProductionSymbolIR, build_export_model_for_lalr,
    build_export_model_for_llk, build_lookahead_automata_model, build_production_model,
    find_start_symbol_index as parser_model_find_start_symbol_index,
};
use crate::generators::parser_render_ir::{
    LalrProductionRenderIR, build_lalr_production_render_ir, build_non_terminal_metadata_ir,
    build_rust_lalr_parse_table_render_ir, build_terminal_label_map,
};
use crate::generators::{GrammarConfig, NamingHelper};
use crate::parser::GrammarType;
use crate::parser::parol_grammar::LookaheadExpression;
use anyhow::Result;
use std::collections::BTreeMap;

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
    #[allow(dead_code)]
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

    fn from_ir(automaton_ir: &LookaheadAutomatonIR) -> Self {
        let prod0 = automaton_ir.prod0;
        let transitions = automaton_ir.transitions.iter().fold(
            StrVec::new(4).first_line_no_indent(),
            |mut acc, t| {
                acc.push(format!(
                    "Trans({}, {}, {}, {}),",
                    t.from_state, t.term, t.to_state, t.prod_num
                ));
                acc
            },
        );
        let k = automaton_ir.k;

        Self {
            prod0,
            transitions,
            k,
            nt_index: automaton_ir.non_terminal_index,
            nt_name: automaton_ir.non_terminal_name.clone(),
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
    fn from_ir(production_ir: &ProductionIR) -> Self {
        let lhs = production_ir.lhs_index;
        let production = production_ir.rhs.iter().rev().fold(
            StrVec::new(4).first_line_no_indent(),
            |mut acc, s| {
                match s {
                    ProductionSymbolIR::NonTerminal(index) => {
                        acc.push(format!("ParseType::N({index}),"))
                    }
                    ProductionSymbolIR::Terminal { index, .. } => {
                        acc.push(format!("ParseType::T({index}),"))
                    }
                }
                acc
            },
        );
        Self {
            lhs,
            production,
            prod_num: production_ir.production_index,
            prod_string: production_ir.text.clone(),
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
    fn from_render_ir(production_render_ir: &LalrProductionRenderIR) -> Self {
        let lhs = production_render_ir.lhs_index;
        let len = production_render_ir.rhs_len;
        Self {
            lhs,
            len,
            prod_num: production_render_ir.production_index,
            prod_string: production_render_ir.text.clone(),
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
    user_type_name: &'a str,
    user_type_life_time: &'static str,
    scanner_type_name: String,
    scanner_module_name: String,
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
            user_type_name,
            scanner_type_name,
            scanner_module_name,
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
            // Do not edit this file manually.
            // Changes will be overwritten on the next build.
            // ---------------------------------------------------------
            "
        )?;

        f.write_fmt(ume::ume! {
            use parol_runtime::{
                parser::{
                    parse_tree_type::TreeConstruct, LLKParser, LookaheadDFA, ParseType, Production, Trans,
                },
                ParolError, ParseTree, TokenStream,
            };
            use scnr2::scanner;
            use std::path::Path;
        })?;

        writeln!(f, "\n")?;
        let auto_name = format!("{user_type_name}Auto");
        let trait_module_name = format!("{module_name}_trait");
        f.write_fmt(ume::ume! {
            use crate::#module_name::#user_type_name;
            use crate::#trait_module_name::#auto_name;
        })?;
        writeln!(f, "\n")?;

        writeln!(f, "{lexer_source}\n")?;

        f.write_fmt(ume::ume! {
            const MAX_K: usize = #max_k;
        })?;
        writeln!(f, "\n\n")?;
        f.write_fmt(ume::ume! {
            pub const NON_TERMINALS: &[&str; #non_terminal_count] = &[#non_terminals];
        })?;

        writeln!(f, "\n\n{dfa_source}")?;
        writeln!(f, "\n{productions}\n")?;

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
        let use_scanner_type = ume::ume! {
            use #scanner_module_name::#scanner_type_name;
        }
        .to_string();
        let scanner_instance = ume::ume! {
            let scanner = #scanner_type_name::new();
        }
        .to_string();
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
                use parol_runtime::{
                    parser::{parse_tree_type::SynTree, parser_types::SynTreeFlavor},
                    syntree::Builder,
                };
                let mut builder = Builder::<SynTree, SynTreeFlavor>::new_with();
                parse_into(input, &mut builder, file_name, user_actions)?;
                Ok(builder.build()?)
            }
        })?;
        f.write_fmt(ume::ume! {
            #[allow(dead_code)]
            pub fn parse_into<'t, T: TreeConstruct<'t>>(
                input: &'t str,
                tree_builder: &mut T,
                file_name: impl AsRef<Path>,
                user_actions: #user_actions,
            ) -> Result<(), ParolError> where ParolError: From<T::Error> {
                #use_scanner_type
                let mut llk_parser = LLKParser::new(
                    #start_symbol_index,
                    LOOKAHEAD_AUTOMATA,
                    PRODUCTIONS,
                    TERMINAL_NAMES,
                    NON_TERMINALS,
                );
                #enable_trimming
                #recovery
                #scanner_instance
                #auto_wrapper

                llk_parser.parse_into(
                    tree_builder,
                    TokenStream::new(
                        input,
                        file_name,
                        scanner.scanner_impl.clone(),
                        &#scanner_type_name::match_function,
                        MAX_K,
                    )
                    .unwrap(),
                    #mut_ref_user_actions
                )
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
    user_type_name: &'a str,
    user_type_life_time: &'static str,
    scanner_type_name: String,
    scanner_module_name: String,
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
            user_type_name,
            user_type_life_time,
            scanner_type_name,
            scanner_module_name,
            module_name,
            trim_parse_tree,
            parse_table_source,
        } = self;

        writeln!(
            f,
            "
            // ---------------------------------------------------------
            // This file was generated by parol.
            // Do not edit this file manually.
            // Changes will be overwritten on the next build.
            // ---------------------------------------------------------
            "
        )?;

        f.write_fmt(ume::ume! {
            use parol_runtime::{
                ParolError, ParseTree, TokenStream,
                lr_parser::{LR1State, LRAction, LRParseTable, LRParser, LRProduction},
                parser::parse_tree_type::TreeConstruct,
            };
            use scnr2::scanner;
            use std::path::Path;
        })?;

        writeln!(f, "\n")?;
        let auto_name = format!("{user_type_name}Auto");
        let trait_module_name = format!("{module_name}_trait");
        f.write_fmt(ume::ume! {
            use crate::#module_name::#user_type_name;
            use crate::#trait_module_name::#auto_name;
        })?;
        writeln!(f, "\n")?;

        writeln!(f, "{lexer_source}\n")?;

        writeln!(f, "\n\n")?;
        f.write_fmt(ume::ume! {
            pub const NON_TERMINALS: &[&str; #non_terminal_count] = &[#non_terminals];
        })?;

        writeln!(
            f,
            "\n\nstatic PARSE_TABLE: LRParseTable  = {parse_table_source};\n"
        )?;
        writeln!(f, "\n{productions}\n")?;

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
        let use_scanner_type = ume::ume! {
            use #scanner_module_name::#scanner_type_name;
        }
        .to_string();
        let scanner_instance = ume::ume! {
            let scanner = #scanner_type_name::new();
        }
        .to_string();

        f.write_fmt(ume::ume! {
            pub fn parse<#lifetime_on_parse T>(
                input: &#lifetime_on_input str,
                file_name: T,
                user_actions: #user_actions,
            ) -> Result<ParseTree, ParolError> where T: AsRef<Path> {
                use parol_runtime::{
                    parser::{parse_tree_type::SynTree, parser_types::SynTreeFlavor},
                    syntree::Builder,
                };
                let mut builder = Builder::<SynTree, SynTreeFlavor>::new_with();
                parse_into(input, &mut builder, file_name, user_actions)?;
                Ok(builder.build()?)
            }
        })?;
        f.write_fmt(ume::ume! {
            #[allow(dead_code)]
            pub fn parse_into<'t, T: TreeConstruct<'t>>(
                input: &'t str,
                tree_builder: &mut T,
                file_name: impl AsRef<Path>,
                user_actions: #user_actions,
            ) -> Result<(), ParolError> where ParolError: From<T::Error> {
                #use_scanner_type
                let mut lr_parser = LRParser::new(
                    #start_symbol_index,
                    &PARSE_TABLE,
                    PRODUCTIONS,
                    TERMINAL_NAMES,
                    NON_TERMINALS,
                );
                #enable_trimming
                #auto_wrapper
                #scanner_instance
                lr_parser.parse_into(
                    tree_builder,
                    TokenStream::new(
                        input,
                        file_name,
                        scanner.scanner_impl.clone(),
                        &#scanner_type_name::match_function,
                        1,
                    )
                    .unwrap(),
                    #mut_ref_user_actions
                )
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
    generate_parser_source_internal(
        grammar_config,
        lexer_source,
        config,
        la_dfa,
        ast_type_has_lifetime,
    )
}

///
/// Builds a language-agnostic export model for LL(k) parser generation.
///
/// This function expects precomputed lookahead DFAs.
/// Prefer this over `generate_parser_export_model_from_grammar` when you already
/// have DFAs to avoid duplicate analysis work.
///
pub fn generate_parser_export_model(
    grammar_config: &GrammarConfig,
    la_dfa: &BTreeMap<String, LookaheadDFA>,
) -> Result<crate::generators::ParserExportModel> {
    build_export_model_for_llk(grammar_config, la_dfa)
}

fn generate_parser_source_internal<C: CommonGeneratorConfig + ParserGeneratorConfig>(
    grammar_config: &GrammarConfig,
    lexer_source: &str,
    config: &C,
    la_dfa: &BTreeMap<String, LookaheadDFA>,
    ast_type_has_lifetime: bool,
) -> Result<String> {
    let non_terminal_metadata = build_non_terminal_metadata_ir(grammar_config);
    let non_terminal_names = non_terminal_metadata.names;
    let non_terminal_count = non_terminal_names.len();
    let start_symbol_index: usize =
        parser_model_find_start_symbol_index(&non_terminal_names, grammar_config)?;

    let non_terminals =
        non_terminal_metadata
            .indexed_rows
            .into_iter()
            .fold(StrVec::new(4), |mut acc, row| {
                acc.push(row);
                acc
            });

    let lookahead_automata_ir = build_lookahead_automata_model(la_dfa, &non_terminal_names);
    let dfa_source = generate_dfa_source(&lookahead_automata_ir);

    let production_ir = build_production_model(grammar_config, &non_terminal_names)?;
    let productions = generate_productions(&production_ir);

    let max_k = grammar_config.lookahead_size;

    let user_type_life_time = if ast_type_has_lifetime { "<'t>" } else { "" };

    let parser_data = ParserData {
        start_symbol_index,
        lexer_source,
        non_terminals,
        non_terminal_count,
        dfa_source,
        productions,
        max_k,
        user_type_name: config.user_type_name(),
        user_type_life_time,
        scanner_type_name: get_scanner_type_name(config),
        scanner_module_name: get_scanner_module_name(config),
        module_name: config.module_name(),
        trim_parse_tree: config.trim_parse_tree(),
        disable_recovery: config.recovery_disabled(),
    };

    Ok(format!("{parser_data}"))
}

fn get_terminals(grammar_config: &GrammarConfig) -> Vec<(&str, Option<LookaheadExpression>)> {
    grammar_config
        .cfg
        .get_ordered_terminals()
        .iter()
        .map(|(t, _, l, _)| (*t, l.clone()))
        .collect::<Vec<(&str, Option<LookaheadExpression>)>>()
}

fn get_scanner_module_name<C: CommonGeneratorConfig>(config: &C) -> String {
    let scanner_module_name = NamingHelper::to_lower_snake_case(config.user_type_name());
    scanner_module_name + "_scanner"
}

fn get_scanner_type_name<C: CommonGeneratorConfig>(config: &C) -> String {
    let scanner_type_name = NamingHelper::to_upper_camel_case(config.user_type_name());
    scanner_type_name + "Scanner"
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
    generate_lalr1_parser_source_internal(
        grammar_config,
        lexer_source,
        config,
        parse_table,
        ast_type_has_lifetime,
    )
}

///
/// Builds a language-agnostic export model for LALR(1) parser generation.
///
/// This function expects a precomputed LALR(1) parse table.
/// Prefer this over `generate_parser_export_model_from_grammar` when you already
/// have the parse table to avoid duplicate analysis work.
///
pub fn generate_lalr1_parser_export_model(
    grammar_config: &GrammarConfig,
    parse_table: &LRParseTable,
) -> Result<crate::generators::ParserExportModel> {
    build_export_model_for_lalr(grammar_config, parse_table)
}

///
/// Builds a language-agnostic export model for the given grammar.
///
/// For LL(k) grammars this function computes lookahead DFAs using `max_lookahead`.
/// For LALR(1) grammars it computes the parse table and ignores `max_lookahead`.
///
/// Use this function when you only have a [`GrammarConfig`] and want one entry point
/// for both parser algorithms.
///
/// ```no_run
/// # use anyhow::Result;
/// # fn example() -> Result<()> {
/// let grammar_config = parol::obtain_grammar_config(
///     "crates/parol/tests/data/arg_tests/generate.par",
///     false,
/// )?;
/// let model = parol::generate_parser_export_model_from_grammar(&grammar_config, 5)?;
/// assert_eq!(model.version, parol::generators::PARSER_EXPORT_MODEL_VERSION);
/// # Ok(())
/// # }
/// ```
///
/// If you already computed lookahead DFAs or an LALR(1) parse table, prefer
/// [`generate_parser_export_model`] or [`generate_lalr1_parser_export_model`]
/// to avoid duplicate analysis work.
pub fn generate_parser_export_model_from_grammar(
    grammar_config: &GrammarConfig,
    max_lookahead: usize,
) -> Result<crate::generators::ParserExportModel> {
    match grammar_config.grammar_type {
        GrammarType::LLK => {
            let lookahead_dfas = crate::calculate_lookahead_dfas(grammar_config, max_lookahead)?;
            generate_parser_export_model(grammar_config, &lookahead_dfas)
        }
        GrammarType::LALR1 => {
            let parse_table = crate::calculate_lalr1_parse_table(grammar_config)?.0;
            generate_lalr1_parser_export_model(grammar_config, &parse_table)
        }
    }
}

fn generate_lalr1_parser_source_internal<C: CommonGeneratorConfig + ParserGeneratorConfig>(
    grammar_config: &GrammarConfig,
    lexer_source: &str,
    config: &C,
    parse_table: &LRParseTable,
    ast_type_has_lifetime: bool,
) -> Result<String> {
    let terminals = get_terminals(grammar_config);
    let non_terminal_metadata = build_non_terminal_metadata_ir(grammar_config);
    let non_terminal_names = non_terminal_metadata.names;
    let non_terminal_count = non_terminal_names.len();
    let start_symbol_index: usize =
        parser_model_find_start_symbol_index(&non_terminal_names, grammar_config)?;

    let non_terminals = non_terminal_names.iter().collect::<Vec<_>>();

    let non_terminals_with_index_comment =
        non_terminal_metadata
            .indexed_rows
            .into_iter()
            .fold(StrVec::new(4), |mut acc, row| {
                acc.push(row);
                acc
            });
    let production_ir = build_production_model(grammar_config, &non_terminal_names)?;
    let productions = generate_lr_productions(&production_ir);

    let user_type_life_time = if ast_type_has_lifetime { "<'t>" } else { "" };

    let parse_table_source = generate_parse_table_source(parse_table, &terminals, &non_terminals);

    let parser_data = LRParserData {
        start_symbol_index,
        lexer_source,
        non_terminals: non_terminals_with_index_comment,
        non_terminal_count,
        productions,
        user_type_name: config.user_type_name(),
        user_type_life_time,
        scanner_type_name: get_scanner_type_name(config),
        scanner_module_name: get_scanner_module_name(config),
        module_name: config.module_name(),
        trim_parse_tree: config.trim_parse_tree(),
        parse_table_source,
    };

    Ok(format!("{parser_data}"))
}

fn generate_parse_table_source(
    parse_table: &LRParseTable,
    terminals: &[(&str, Option<LookaheadExpression>)],
    non_terminals: &[&String],
) -> String {
    let terminal_labels = build_terminal_label_map(terminals);
    let non_terminal_names = non_terminals
        .iter()
        .map(|n| (*n).clone())
        .collect::<Vec<_>>();
    let render_ir =
        build_rust_lalr_parse_table_render_ir(parse_table, &terminal_labels, &non_terminal_names);

    let actions =
        render_ir
            .actions
            .iter()
            .enumerate()
            .fold(String::new(), |mut acc, (i, action_source)| {
                acc.push_str(format!("/* {} */ {}, ", i, action_source).as_str());
                acc
            });

    let states = render_ir
        .states
        .iter()
        .fold(String::new(), |mut acc, state| {
            let state_actions = format!(
                "&[{}]",
                state
                    .actions
                    .iter()
                    .map(|a| {
                        format!(
                            r#"
        ({}, {}) /* '{}' => {} */"#,
                            a.terminal, a.action_index, a.terminal_label, a.action_comment
                        )
                    })
                    .collect::<Vec<String>>()
                    .join(", ")
            );

            let state_gotos = if state.gotos.is_empty() {
                "&[]".to_string()
            } else {
                format!(
                    "&[{}]",
                    state
                        .gotos
                        .iter()
                        .map(|g| {
                            format!(
                                r#"
                ({}, {}) /* {} => {} */"#,
                                g.non_terminal, g.goto_state, g.non_terminal_name, g.goto_state,
                            )
                        })
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            };

            acc.push_str(
                format!(
                    r#"
        // State {}
        LR1State {{
            actions: {},
            gotos: {} }}"#,
                    state.state_index, state_actions, state_gotos
                )
                .as_str(),
            );
            acc.push(',');
            acc
        });

    format!("LRParseTable {{ actions: &[{actions}], states: &[{states}] }}",)
}

fn generate_dfa_source(lookahead_automata_ir: &[LookaheadAutomatonIR]) -> String {
    let lookahead_dfa_s =
        lookahead_automata_ir
            .iter()
            .fold(StrVec::new(0), |mut acc, automaton_ir| {
                let dfa = Dfa::from_ir(automaton_ir);
                acc.push(format!("{dfa}"));
                acc
            });
    let dfa_count = lookahead_automata_ir.len();

    let dfas = Dfas {
        dfa_count,
        lookahead_dfa_s: format!("{lookahead_dfa_s}"),
    };

    format!("{dfas}")
}

fn generate_productions(production_ir: &[ProductionIR]) -> String {
    let production_count = production_ir.len();
    let productions = production_ir.iter().fold(String::new(), |mut acc, p| {
        let production = Production::from_ir(p);
        acc.push_str(format!("{production}").as_str());
        acc
    });

    let productions = Productions {
        production_count,
        productions,
    };

    format!("{productions}")
}

fn generate_lr_productions(production_ir: &[ProductionIR]) -> String {
    let production_render_ir = build_lalr_production_render_ir(production_ir);
    let production_count = production_render_ir.len();
    let productions = production_render_ir
        .iter()
        .fold(String::new(), |mut acc, p| {
            let production = LRProduction::from_render_ir(p);
            acc.push_str(format!("{production}").as_str());
            acc
        });

    let productions = LRProductions {
        production_count,
        productions,
    };

    format!("{productions}")
}
