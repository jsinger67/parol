use crate::LRParseTable;
use crate::analysis::LookaheadDFA;
use crate::analysis::compiled_la_dfa::CompiledDFA;
use crate::analysis::lookahead_dfa::CompiledProductionIndex;
use crate::generators::GrammarConfig;
use crate::generators::grammar_type_generator::GrammarTypeInfo;
use crate::generators::symbol_table::TypeEntrails;
use crate::generators::symbol_table_facade::{InstanceFacade, SymbolFacade, TypeFacade};
use crate::grammar::{ProductionAttribute, Symbol, SymbolAttribute, Terminal, TerminalKind};
use crate::parser::parol_grammar::{LookaheadExpression, ScannerStateSwitch};
use anyhow::{Result, anyhow};
use parol_runtime::TerminalIndex;
use parol_runtime::lexer::FIRST_USER_TOKEN;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Version of the language-agnostic parser export model schema.
pub const PARSER_EXPORT_MODEL_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Discriminator of the exported parser algorithm family.
pub enum ParserAlgorithmKindModel {
    /// LL(k) parser model with lookahead automata.
    Llk,
    /// LALR(1) parser model with parse table.
    Lalr1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Language-agnostic LALR parser action.
pub enum LalrActionModel {
    /// Shift to parser state.
    Shift(usize),
    /// Reduce by `(non_terminal_index, production_index)`.
    Reduce(usize, usize),
    /// Accept the input.
    Accept,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Portable production symbol representation.
pub enum ProductionSymbolExportModel {
    /// Reference to non-terminal by index.
    NonTerminal(usize),
    /// Reference to terminal by token index with clipped marker.
    Terminal { index: TerminalIndex, clipped: bool },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductionExportModel {
    pub production_index: usize,
    pub lhs_index: usize,
    pub rhs: Vec<ProductionSymbolExportModel>,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LalrStateExportModel {
    pub actions: Vec<(TerminalIndex, usize)>,
    pub gotos: Vec<(usize, usize)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LalrParseTableExportModel {
    pub actions: Vec<LalrActionModel>,
    pub states: Vec<LalrStateExportModel>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LookaheadTransitionExportModel {
    pub from_state: usize,
    pub term: TerminalIndex,
    pub to_state: usize,
    pub prod_num: CompiledProductionIndex,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LookaheadAutomatonExportModel {
    pub non_terminal_index: usize,
    pub non_terminal_name: String,
    pub prod0: CompiledProductionIndex,
    pub k: usize,
    pub transitions: Vec<LookaheadTransitionExportModel>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerminalKindExportModel {
    Legacy,
    Regex,
    Raw,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LookaheadExpressionExportModel {
    pub is_positive: bool,
    pub pattern: String,
    pub expanded_pattern: String,
    pub kind: TerminalKindExportModel,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScannerTerminalExportModel {
    pub index: TerminalIndex,
    pub pattern: String,
    pub expanded_pattern: String,
    pub kind: TerminalKindExportModel,
    pub lookahead: Option<LookaheadExpressionExportModel>,
    pub scanner_states: Vec<usize>,
    pub scanner_state_names: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScannerTransitionKindExportModel {
    Enter,
    Push,
    Pop,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScannerTransitionExportModel {
    pub terminal_index: TerminalIndex,
    pub kind: ScannerTransitionKindExportModel,
    pub target_scanner_state: Option<usize>,
    pub target_scanner_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScannerStateExportModel {
    pub scanner_state: usize,
    pub scanner_name: String,
    pub line_comments: Vec<String>,
    pub block_comments: Vec<(String, String)>,
    pub auto_newline: bool,
    pub auto_ws: bool,
    pub allow_unmatched: bool,
    pub transitions: Vec<ScannerTransitionExportModel>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScannerExportModel {
    pub terminals: Vec<ScannerTerminalExportModel>,
    pub scanner_states: Vec<ScannerStateExportModel>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProductionAttributeExportModel {
    None,
    CollectionStart,
    AddToCollection,
    OptionalSome,
    OptionalNone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymbolAttributeExportModel {
    None,
    RepetitionAnchor,
    Option,
    Clipped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportTypeKindModel {
    None,
    Token,
    Box,
    Ref,
    Surrogate,
    Struct,
    Enum,
    EnumVariant,
    Vec,
    Trait,
    Function,
    Option,
    Clipped,
    UserDefinedType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductionTypeMemberExportModel {
    pub member_name: String,
    pub symbol_attribute: SymbolAttributeExportModel,
    pub description: String,
    pub used: bool,
    pub type_name: String,
    pub type_kind: ExportTypeKindModel,
    pub rust_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductionDatatypeExportModel {
    pub production_index: usize,
    pub non_terminal_name: String,
    pub production_attribute: ProductionAttributeExportModel,
    pub type_name: String,
    pub type_kind: ExportTypeKindModel,
    pub rust_type: String,
    pub members: Vec<ProductionTypeMemberExportModel>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Language-agnostic parser export model intended for external code generators.
pub struct ParserExportModel {
    /// Schema version of this export model.
    pub version: u32,
    /// Parser algorithm variant represented by this model.
    pub algorithm: ParserAlgorithmKindModel,
    /// Ordered non-terminal names.
    pub non_terminal_names: Vec<String>,
    /// Index of the start symbol in `non_terminal_names`.
    pub start_symbol_index: usize,
    /// Ordered production metadata.
    pub productions: Vec<ProductionExportModel>,
    /// LL(k) lookahead automata. Empty for LALR(1).
    pub lookahead_automata: Vec<LookaheadAutomatonExportModel>,
    /// LALR(1) parse table data. `None` for LL(k).
    pub lalr_parse_table: Option<LalrParseTableExportModel>,
    /// Scanner model used to generate lexers/scanners for this grammar.
    pub scanner: ScannerExportModel,
    /// Datatype model for production outputs, derived from GrammarTypeInfo's symbol table.
    pub production_datatypes: Vec<ProductionDatatypeExportModel>,
}

pub(crate) enum ProductionSymbolModel {
    NonTerminal(usize),
    Terminal { index: TerminalIndex, clipped: bool },
}

pub(crate) struct ProductionModel {
    pub(crate) production_index: usize,
    pub(crate) lhs_index: usize,
    pub(crate) rhs: Vec<ProductionSymbolModel>,
    pub(crate) text: String,
}

pub(crate) struct LalrStateModel {
    pub(crate) actions: Vec<(TerminalIndex, usize)>,
    pub(crate) gotos: Vec<(usize, usize)>,
}

pub(crate) struct LalrParseTableModel {
    pub(crate) actions: Vec<crate::LRAction>,
    pub(crate) states: Vec<LalrStateModel>,
}

pub(crate) struct LookaheadTransitionModel {
    pub(crate) from_state: usize,
    pub(crate) term: TerminalIndex,
    pub(crate) to_state: usize,
    pub(crate) prod_num: CompiledProductionIndex,
}

pub(crate) struct LookaheadAutomatonModel {
    pub(crate) non_terminal_index: usize,
    pub(crate) non_terminal_name: String,
    pub(crate) prod0: CompiledProductionIndex,
    pub(crate) k: usize,
    pub(crate) transitions: Vec<LookaheadTransitionModel>,
}

pub(crate) fn ordered_non_terminal_names(grammar_config: &GrammarConfig) -> Vec<String> {
    grammar_config
        .cfg
        .get_non_terminal_set()
        .iter()
        .cloned()
        .collect::<Vec<_>>()
}

pub(crate) fn find_start_symbol_index(
    non_terminal_names: &[String],
    grammar_config: &GrammarConfig,
) -> Result<usize> {
    non_terminal_names
        .iter()
        .position(|n| n == grammar_config.cfg.get_start_symbol())
        .ok_or_else(|| {
            anyhow!(
                "Start symbol '{}' is not part of the given grammar!",
                grammar_config.cfg.get_start_symbol()
            )
        })
}

pub(crate) fn build_production_model(
    grammar_config: &GrammarConfig,
    non_terminal_names: &[String],
) -> Result<Vec<ProductionModel>> {
    let terminals = grammar_config.cfg.get_ordered_terminals();

    let get_non_terminal_index = |nt: &str| {
        non_terminal_names
            .iter()
            .position(|n| n == nt)
            .ok_or_else(|| anyhow!("Non-terminal '{}' not found", nt))
    };

    let get_terminal_index = |tr: &str, l: &Option<LookaheadExpression>| -> Result<TerminalIndex> {
        terminals
            .iter()
            .position(|(t, _, look, _)| *t == tr && look == l)
            .map(|i| i as TerminalIndex + parol_runtime::lexer::FIRST_USER_TOKEN)
            .ok_or_else(|| anyhow!("Terminal '{}' with lookahead not found", tr))
    };

    grammar_config
        .cfg
        .pr
        .iter()
        .enumerate()
        .map(|(production_index, pr)| {
            let lhs_index = get_non_terminal_index(pr.get_n_str())?;
            let rhs = pr
                .get_r()
                .iter()
                .map(|s| match s {
                    Symbol::N(n, ..) => {
                        get_non_terminal_index(n).map(ProductionSymbolModel::NonTerminal)
                    }
                    Symbol::T(Terminal::Trm(t, _, _, attr, _, _, l0)) => get_terminal_index(t, l0)
                        .map(|index| ProductionSymbolModel::Terminal {
                            index,
                            clipped: *attr == SymbolAttribute::Clipped,
                        }),
                    _ => Err(anyhow!("Unexpected symbol type in production")),
                })
                .collect::<Result<Vec<_>>>()?;

            Ok(ProductionModel {
                production_index,
                lhs_index,
                rhs,
                text: format!("{pr}"),
            })
        })
        .collect::<Result<Vec<_>>>()
}

pub(crate) fn build_lalr_parse_table_model(parse_table: &LRParseTable) -> LalrParseTableModel {
    let actions = parse_table
        .states
        .iter()
        .fold(BTreeSet::<crate::LRAction>::new(), |mut acc, s| {
            s.actions.iter().for_each(|(_, a)| {
                acc.insert(a.clone());
            });
            acc
        })
        .into_iter()
        .collect::<Vec<_>>();

    let action_index = actions
        .iter()
        .cloned()
        .enumerate()
        .map(|(i, action)| (action, i))
        .collect::<BTreeMap<_, _>>();

    let states = parse_table
        .states
        .iter()
        .map(|state| LalrStateModel {
            actions: state
                .actions
                .iter()
                .map(|(terminal, action)| (*terminal, *action_index.get(action).unwrap()))
                .collect::<Vec<_>>(),
            gotos: state
                .gotos
                .iter()
                .map(|(non_terminal, goto_state)| (*non_terminal, *goto_state))
                .collect::<Vec<_>>(),
        })
        .collect::<Vec<_>>();

    LalrParseTableModel { actions, states }
}

pub(crate) fn build_lookahead_automata_model(
    la_dfa: &BTreeMap<String, LookaheadDFA>,
    non_terminal_names: &[String],
) -> Vec<LookaheadAutomatonModel> {
    non_terminal_names
        .iter()
        .enumerate()
        .filter_map(|(non_terminal_index, non_terminal_name)| {
            la_dfa.get(non_terminal_name).map(|dfa| {
                let compiled = CompiledDFA::from_lookahead_dfa(dfa);
                LookaheadAutomatonModel {
                    non_terminal_index,
                    non_terminal_name: non_terminal_name.clone(),
                    prod0: compiled.prod0,
                    k: compiled.k,
                    transitions: compiled
                        .transitions
                        .iter()
                        .map(|t| LookaheadTransitionModel {
                            from_state: t.from_state,
                            term: t.term,
                            to_state: t.to_state,
                            prod_num: t.prod_num,
                        })
                        .collect::<Vec<_>>(),
                }
            })
        })
        .collect::<Vec<_>>()
}

fn to_lalr_action_model(action: &crate::LRAction) -> LalrActionModel {
    match action {
        crate::LRAction::Shift(state) => LalrActionModel::Shift(*state),
        crate::LRAction::Reduce(non_terminal, production) => {
            LalrActionModel::Reduce(*non_terminal, *production)
        }
        crate::LRAction::Accept => LalrActionModel::Accept,
    }
}

fn to_terminal_kind_export_model(kind: TerminalKind) -> TerminalKindExportModel {
    match kind {
        TerminalKind::Legacy => TerminalKindExportModel::Legacy,
        TerminalKind::Regex => TerminalKindExportModel::Regex,
        TerminalKind::Raw => TerminalKindExportModel::Raw,
    }
}

fn to_production_attribute_export_model(
    attribute: ProductionAttribute,
) -> ProductionAttributeExportModel {
    match attribute {
        ProductionAttribute::None => ProductionAttributeExportModel::None,
        ProductionAttribute::CollectionStart => ProductionAttributeExportModel::CollectionStart,
        ProductionAttribute::AddToCollection => ProductionAttributeExportModel::AddToCollection,
        ProductionAttribute::OptionalSome => ProductionAttributeExportModel::OptionalSome,
        ProductionAttribute::OptionalNone => ProductionAttributeExportModel::OptionalNone,
    }
}

fn to_symbol_attribute_export_model(attribute: SymbolAttribute) -> SymbolAttributeExportModel {
    match attribute {
        SymbolAttribute::None => SymbolAttributeExportModel::None,
        SymbolAttribute::RepetitionAnchor => SymbolAttributeExportModel::RepetitionAnchor,
        SymbolAttribute::Option => SymbolAttributeExportModel::Option,
        SymbolAttribute::Clipped => SymbolAttributeExportModel::Clipped,
    }
}

fn to_export_type_kind_model(entrails: &TypeEntrails) -> ExportTypeKindModel {
    match entrails {
        TypeEntrails::None => ExportTypeKindModel::None,
        TypeEntrails::Token => ExportTypeKindModel::Token,
        TypeEntrails::Box(_) => ExportTypeKindModel::Box,
        TypeEntrails::Ref(_) => ExportTypeKindModel::Ref,
        TypeEntrails::Surrogate(_) => ExportTypeKindModel::Surrogate,
        TypeEntrails::Struct => ExportTypeKindModel::Struct,
        TypeEntrails::Enum => ExportTypeKindModel::Enum,
        TypeEntrails::EnumVariant(_) => ExportTypeKindModel::EnumVariant,
        TypeEntrails::Vec(_) => ExportTypeKindModel::Vec,
        TypeEntrails::Trait => ExportTypeKindModel::Trait,
        TypeEntrails::Function(_) => ExportTypeKindModel::Function,
        TypeEntrails::Option(_) => ExportTypeKindModel::Option,
        TypeEntrails::Clipped(_) => ExportTypeKindModel::Clipped,
        TypeEntrails::UserDefinedType(_, _) => ExportTypeKindModel::UserDefinedType,
    }
}

fn build_production_datatypes_export_model(
    grammar_config: &GrammarConfig,
) -> Result<Vec<ProductionDatatypeExportModel>> {
    let mut type_info = GrammarTypeInfo::try_new("ParserExportModel")?;
    type_info.set_grammar_type(grammar_config.grammar_type);
    type_info.build(grammar_config)?;

    grammar_config
        .cfg
        .pr
        .iter()
        .enumerate()
        .map(|(production_index, production)| {
            let type_id = *type_info
                .production_types
                .get(&production_index)
                .ok_or_else(|| {
                    anyhow!("Missing production type for production {production_index}")
                })?;

            let type_symbol = type_info.symbol_table.symbol(type_id);
            let type_name = type_symbol.name();
            let rust_type = type_symbol.to_rust();
            let type_facade = type_info.symbol_table.symbol_as_type(type_id);
            let type_kind = to_export_type_kind_model(type_facade.entrails());

            let members = type_info
                .symbol_table
                .members(type_id)?
                .iter()
                .map(|member_id| {
                    let member = type_info.symbol_table.symbol_as_instance(*member_id);
                    let member_type = type_info.symbol_table.symbol_as_type(member.type_id());
                    let member_type_symbol = type_info.symbol_table.symbol(member.type_id());
                    ProductionTypeMemberExportModel {
                        member_name: member.name(),
                        symbol_attribute: to_symbol_attribute_export_model(member.sem()),
                        description: member.description().to_string(),
                        used: member.used(),
                        type_name: member_type_symbol.name(),
                        type_kind: to_export_type_kind_model(member_type.entrails()),
                        rust_type: member_type_symbol.to_rust(),
                    }
                })
                .collect::<Vec<_>>();

            Ok(ProductionDatatypeExportModel {
                production_index,
                non_terminal_name: production.get_n(),
                production_attribute: to_production_attribute_export_model(
                    production.get_attribute(),
                ),
                type_name,
                type_kind,
                rust_type,
                members,
            })
        })
        .collect::<Result<Vec<_>>>()
}

fn build_scanner_export_model(grammar_config: &GrammarConfig) -> ScannerExportModel {
    let scanner_name_by_state = grammar_config
        .scanner_configurations
        .iter()
        .map(|s| (s.scanner_state, s.scanner_name.clone()))
        .collect::<BTreeMap<_, _>>();
    let scanner_state_by_name = grammar_config
        .scanner_configurations
        .iter()
        .map(|s| (s.scanner_name.clone(), s.scanner_state))
        .collect::<BTreeMap<_, _>>();

    let terminals = grammar_config
        .cfg
        .get_ordered_terminals()
        .into_iter()
        .enumerate()
        .map(|(i, (pattern, kind, lookahead, scanner_states))| {
            let scanner_state_names = scanner_states
                .iter()
                .map(|state| {
                    scanner_name_by_state
                        .get(state)
                        .cloned()
                        .unwrap_or_else(|| "<?>".to_string())
                })
                .collect::<Vec<_>>();
            ScannerTerminalExportModel {
                index: i as TerminalIndex + FIRST_USER_TOKEN,
                pattern: pattern.to_string(),
                expanded_pattern: kind.expand(pattern),
                kind: to_terminal_kind_export_model(kind),
                lookahead: lookahead
                    .as_ref()
                    .map(|lookahead| LookaheadExpressionExportModel {
                        is_positive: lookahead.is_positive,
                        pattern: lookahead.pattern.clone(),
                        expanded_pattern: lookahead.kind.expand(&lookahead.pattern),
                        kind: to_terminal_kind_export_model(lookahead.kind),
                    }),
                scanner_states,
                scanner_state_names,
            }
        })
        .collect::<Vec<_>>();

    let scanner_states = grammar_config
        .scanner_configurations
        .iter()
        .map(|scanner| {
            let transitions = scanner
                .transitions
                .iter()
                .map(|(terminal_index, transition)| match transition {
                    ScannerStateSwitch::Switch(target_name, _) => ScannerTransitionExportModel {
                        terminal_index: *terminal_index,
                        kind: ScannerTransitionKindExportModel::Enter,
                        target_scanner_state: scanner_state_by_name.get(target_name).copied(),
                        target_scanner_name: Some(target_name.clone()),
                    },
                    ScannerStateSwitch::SwitchPush(target_name, _) => {
                        ScannerTransitionExportModel {
                            terminal_index: *terminal_index,
                            kind: ScannerTransitionKindExportModel::Push,
                            target_scanner_state: scanner_state_by_name.get(target_name).copied(),
                            target_scanner_name: Some(target_name.clone()),
                        }
                    }
                    ScannerStateSwitch::SwitchPop(_) => ScannerTransitionExportModel {
                        terminal_index: *terminal_index,
                        kind: ScannerTransitionKindExportModel::Pop,
                        target_scanner_state: None,
                        target_scanner_name: None,
                    },
                })
                .collect::<Vec<_>>();
            ScannerStateExportModel {
                scanner_state: scanner.scanner_state,
                scanner_name: scanner.scanner_name.clone(),
                line_comments: scanner.line_comments.clone(),
                block_comments: scanner.block_comments.clone(),
                auto_newline: scanner.auto_newline,
                auto_ws: scanner.auto_ws,
                allow_unmatched: scanner.allow_unmatched,
                transitions,
            }
        })
        .collect::<Vec<_>>();

    ScannerExportModel {
        terminals,
        scanner_states,
    }
}

fn to_production_export_model(production: &ProductionModel) -> ProductionExportModel {
    ProductionExportModel {
        production_index: production.production_index,
        lhs_index: production.lhs_index,
        rhs: production
            .rhs
            .iter()
            .map(|symbol| match symbol {
                ProductionSymbolModel::NonTerminal(index) => {
                    ProductionSymbolExportModel::NonTerminal(*index)
                }
                ProductionSymbolModel::Terminal { index, clipped } => {
                    ProductionSymbolExportModel::Terminal {
                        index: *index,
                        clipped: *clipped,
                    }
                }
            })
            .collect::<Vec<_>>(),
        text: production.text.clone(),
    }
}

fn to_lookahead_automaton_export_model(
    automaton: &LookaheadAutomatonModel,
) -> LookaheadAutomatonExportModel {
    LookaheadAutomatonExportModel {
        non_terminal_index: automaton.non_terminal_index,
        non_terminal_name: automaton.non_terminal_name.clone(),
        prod0: automaton.prod0,
        k: automaton.k,
        transitions: automaton
            .transitions
            .iter()
            .map(|t| LookaheadTransitionExportModel {
                from_state: t.from_state,
                term: t.term,
                to_state: t.to_state,
                prod_num: t.prod_num,
            })
            .collect::<Vec<_>>(),
    }
}

fn to_lalr_parse_table_export_model(
    parse_table: &LalrParseTableModel,
) -> LalrParseTableExportModel {
    LalrParseTableExportModel {
        actions: parse_table
            .actions
            .iter()
            .map(to_lalr_action_model)
            .collect::<Vec<_>>(),
        states: parse_table
            .states
            .iter()
            .map(|state| LalrStateExportModel {
                actions: state.actions.clone(),
                gotos: state.gotos.clone(),
            })
            .collect::<Vec<_>>(),
    }
}

pub(crate) fn build_export_model_for_llk(
    grammar_config: &GrammarConfig,
    la_dfa: &BTreeMap<String, LookaheadDFA>,
) -> Result<ParserExportModel> {
    let non_terminal_names = ordered_non_terminal_names(grammar_config);
    let start_symbol_index = find_start_symbol_index(&non_terminal_names, grammar_config)?;
    let productions = build_production_model(grammar_config, &non_terminal_names)?;
    let lookahead_automata = build_lookahead_automata_model(la_dfa, &non_terminal_names);
    let scanner = build_scanner_export_model(grammar_config);
    let production_datatypes = build_production_datatypes_export_model(grammar_config)?;

    Ok(ParserExportModel {
        version: PARSER_EXPORT_MODEL_VERSION,
        algorithm: ParserAlgorithmKindModel::Llk,
        non_terminal_names,
        start_symbol_index,
        productions: productions
            .iter()
            .map(to_production_export_model)
            .collect::<Vec<_>>(),
        lookahead_automata: lookahead_automata
            .iter()
            .map(to_lookahead_automaton_export_model)
            .collect::<Vec<_>>(),
        lalr_parse_table: None,
        scanner,
        production_datatypes,
    })
}

pub(crate) fn build_export_model_for_lalr(
    grammar_config: &GrammarConfig,
    parse_table: &LRParseTable,
) -> Result<ParserExportModel> {
    let non_terminal_names = ordered_non_terminal_names(grammar_config);
    let start_symbol_index = find_start_symbol_index(&non_terminal_names, grammar_config)?;
    let productions = build_production_model(grammar_config, &non_terminal_names)?;
    let parse_table_model = build_lalr_parse_table_model(parse_table);
    let scanner = build_scanner_export_model(grammar_config);
    let production_datatypes = build_production_datatypes_export_model(grammar_config)?;

    Ok(ParserExportModel {
        version: PARSER_EXPORT_MODEL_VERSION,
        algorithm: ParserAlgorithmKindModel::Lalr1,
        non_terminal_names,
        start_symbol_index,
        productions: productions
            .iter()
            .map(to_production_export_model)
            .collect::<Vec<_>>(),
        lookahead_automata: Vec::new(),
        lalr_parse_table: Some(to_lalr_parse_table_export_model(&parse_table_model)),
        scanner,
        production_datatypes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::obtain_grammar_config;
    use anyhow::anyhow;
    use parol_runtime::lexer::FIRST_USER_TOKEN;
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn grammar_path(file_name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/valid")
            .join(file_name)
    }

    #[test]
    fn llk_export_model_is_versioned_and_serializable() {
        let grammar_path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/arg_tests/generate.par");
        let grammar_config = obtain_grammar_config(grammar_path, false).unwrap();
        let la_dfa = crate::calculate_lookahead_dfas(&grammar_config, 5).unwrap();

        let model = build_export_model_for_llk(&grammar_config, &la_dfa).unwrap();
        let json = serde_json::to_string(&model).unwrap();

        assert!(json.contains("\"version\":1"));
        assert!(json.contains("\"algorithm\":\"Llk\""));
        assert!(model.lalr_parse_table.is_none());
        assert!(!model.lookahead_automata.is_empty());
        assert!(!model.scanner.terminals.is_empty());
        assert!(!model.scanner.scanner_states.is_empty());
        assert_eq!(model.production_datatypes.len(), model.productions.len());
    }

    #[test]
    fn lalr_export_model_contains_parse_table() {
        let grammar_path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/arg_tests/generate_lr.par");
        let grammar_config = obtain_grammar_config(grammar_path, false).unwrap();
        let parse_table = crate::calculate_lalr1_parse_table(&grammar_config)
            .unwrap()
            .0;

        let model = build_export_model_for_lalr(&grammar_config, &parse_table).unwrap();
        let json = serde_json::to_string(&model).unwrap();

        assert!(json.contains("\"algorithm\":\"Lalr1\""));
        assert!(model.lalr_parse_table.is_some());
        assert!(model.lookahead_automata.is_empty());
        assert!(!model.scanner.terminals.is_empty());
        assert!(!model.scanner.scanner_states.is_empty());
        assert_eq!(model.production_datatypes.len(), model.productions.len());
    }

    #[test]
    fn production_model_marks_clipped_terminals() {
        let grammar_config = obtain_grammar_config(grammar_path("clipped1.par"), false).unwrap();
        let non_terminal_names = ordered_non_terminal_names(&grammar_config);

        let production_model =
            build_production_model(&grammar_config, &non_terminal_names).unwrap();
        let rhs = &production_model[0].rhs;

        assert_eq!(rhs.len(), 2);
        assert!(matches!(
            rhs[0],
            ProductionSymbolModel::Terminal {
                index: FIRST_USER_TOKEN,
                clipped: false
            }
        ));
        assert!(matches!(
            rhs[1],
            ProductionSymbolModel::Terminal {
                index,
                clipped: true
            } if index == FIRST_USER_TOKEN + 1
        ));
    }

    #[test]
    fn production_model_errors_on_unknown_non_terminal() {
        let grammar_config = obtain_grammar_config(grammar_path("clipped1.par"), false).unwrap();
        let invalid_non_terminals = vec!["Unknown".to_string()];

        let error = match build_production_model(&grammar_config, &invalid_non_terminals) {
            Ok(_) => panic!("Expected build_production_model to fail for unknown non-terminal"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("Non-terminal 'Start' not found"));
    }

    #[test]
    fn production_model_distinguishes_same_terminal_with_lookahead() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_grammar_file = std::env::temp_dir().join(format!(
            "parol_parser_model_lookahead_{}_{}.par",
            std::process::id(),
            now
        ));
        let grammar = r#"%start S
%%
S: "a" ?= "b" | "a";
"#;
        fs::write(&temp_grammar_file, grammar).unwrap();

        let test_result = (|| {
            let grammar_config = obtain_grammar_config(temp_grammar_file.clone(), false)?;
            let non_terminal_names = ordered_non_terminal_names(&grammar_config);
            let production_model = build_production_model(&grammar_config, &non_terminal_names)?;

            anyhow::ensure!(
                production_model.len() == 2,
                "Expected exactly two productions for test grammar"
            );

            let lookahead_terminal_index = grammar_config
                .cfg
                .get_ordered_terminals()
                .iter()
                .position(|(t, _, l, _)| *t == "a" && l.is_some())
                .map(|i| i as TerminalIndex + FIRST_USER_TOKEN)
                .ok_or_else(|| anyhow!("Failed to resolve lookahead terminal index"))?;
            let plain_terminal_index = grammar_config
                .cfg
                .get_ordered_terminals()
                .iter()
                .position(|(t, _, l, _)| *t == "a" && l.is_none())
                .map(|i| i as TerminalIndex + FIRST_USER_TOKEN)
                .ok_or_else(|| anyhow!("Failed to resolve plain terminal index"))?;

            anyhow::ensure!(
                lookahead_terminal_index != plain_terminal_index,
                "Lookahead and plain terminals must resolve to different indices"
            );

            let first_index = match production_model[0].rhs.as_slice() {
                [ProductionSymbolModel::Terminal { index, .. }] => *index,
                _ => anyhow::bail!("Unexpected RHS for first production"),
            };
            let second_index = match production_model[1].rhs.as_slice() {
                [ProductionSymbolModel::Terminal { index, .. }] => *index,
                _ => anyhow::bail!("Unexpected RHS for second production"),
            };

            anyhow::ensure!(
                first_index == lookahead_terminal_index,
                "First production should use lookahead-specific terminal index"
            );
            anyhow::ensure!(
                second_index == plain_terminal_index,
                "Second production should use plain terminal index"
            );
            Ok::<(), anyhow::Error>(())
        })();

        let _ = fs::remove_file(&temp_grammar_file);
        test_result.unwrap();
    }

    #[test]
    fn lalr_parse_table_model_action_refs_are_consistent() {
        let grammar_path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/arg_tests/generate_lr.par");
        let grammar_config = obtain_grammar_config(grammar_path, false).unwrap();
        let parse_table = crate::calculate_lalr1_parse_table(&grammar_config)
            .unwrap()
            .0;

        let parse_table_model = build_lalr_parse_table_model(&parse_table);

        assert_eq!(parse_table.states.len(), parse_table_model.states.len());
        assert_eq!(
            parse_table_model.actions.len(),
            parse_table_model
                .actions
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>()
                .len()
        );

        for (state, state_model) in parse_table
            .states
            .iter()
            .zip(parse_table_model.states.iter())
        {
            assert_eq!(state.actions.len(), state_model.actions.len());
            assert_eq!(state.gotos.len(), state_model.gotos.len());

            for ((terminal, action), (model_terminal, model_action_index)) in
                state.actions.iter().zip(state_model.actions.iter())
            {
                assert_eq!(terminal, model_terminal);
                assert_eq!(action, &parse_table_model.actions[*model_action_index]);
            }

            for ((non_terminal, goto_state), (model_non_terminal, model_goto_state)) in
                state.gotos.iter().zip(state_model.gotos.iter())
            {
                assert_eq!(non_terminal, model_non_terminal);
                assert_eq!(goto_state, model_goto_state);
            }
        }
    }

    #[test]
    fn lookahead_automata_model_follows_non_terminal_order() {
        let grammar_path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/arg_tests/generate.par");
        let grammar_config = obtain_grammar_config(grammar_path, false).unwrap();
        let la_dfa = crate::calculate_lookahead_dfas(&grammar_config, 5).unwrap();
        let non_terminal_names = ordered_non_terminal_names(&grammar_config);

        let automata_model = build_lookahead_automata_model(&la_dfa, &non_terminal_names);

        assert_eq!(automata_model.len(), la_dfa.len());
        for automaton in &automata_model {
            assert_eq!(
                automaton.non_terminal_name,
                non_terminal_names[automaton.non_terminal_index]
            );
            assert!(la_dfa.contains_key(&automaton.non_terminal_name));
        }
    }
}
