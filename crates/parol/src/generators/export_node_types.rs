use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::SymbolAttribute;

use super::GrammarConfig;
use super::grammar_type_generator::{GrammarTypeInfo, NonTerminalEnumType};
use std::collections::HashMap;

use crate::generators::{NamingHelper, generate_terminal_name};
use crate::grammar::{ProductionAttribute, Symbol as GrammarSymbol};

/// Syntree node types generator.
pub struct NodeTypesExporter<'a> {
    grammar_config: &'a GrammarConfig,
    grammar_type_info: &'a GrammarTypeInfo,
    terminals: Vec<(usize, String)>,
}

impl<'a> NodeTypesExporter<'a> {
    /// Create a new node types exporter.
    ///
    /// Arguments must be properly prepared before calling this function.
    pub fn new(grammar_config: &'a GrammarConfig, grammar_type_info: &'a GrammarTypeInfo) -> Self {
        Self {
            grammar_config,
            grammar_type_info,
            terminals: grammar_config.generate_terminal_names(),
        }
    }
}

impl NodeTypesExporter<'_> {
    /// Generate the node type information for the grammar.
    ///
    /// This method extracts detailed information about the grammar's terminal and non-terminal nodes,
    /// including their structures, attributes, and relationships. The resulting `NodeInfo` can be
    /// used by parol users to write custom code generation with the same capabilities as parol itself.
    pub fn generate(&self) -> NodeTypesInfo {
        let mut user_terminal_patterns: HashMap<usize, String> = HashMap::new();
        for (i, (pattern, _, _, _)) in self
            .grammar_config
            .cfg
            .get_ordered_terminals_owned()
            .iter()
            .enumerate()
        {
            user_terminal_patterns.insert(i + 5, pattern.clone()); // User terminals start at index 5
        }

        let terminal_infos = self
            .terminals
            .iter()
            .map(|(idx, variant_name)| TerminalInfo {
                name: variant_name.clone(),
                variant: variant_name.clone(),
                index: *idx,
            })
            .collect();

        let nt_original_to_variant: HashMap<String, String> = self
            .grammar_type_info
            .generate_non_terminal_enum_type()
            .into_iter()
            .map(
                |NonTerminalEnumType {
                     from_non_terminal_name,
                     name,
                     ..
                 }| (from_non_terminal_name.to_string(), name),
            )
            .collect();

        let start_symbol = self.grammar_config.cfg.get_start_symbol();

        let non_terminal_infos = self
            .grammar_config
            .cfg
            .get_non_terminal_set()
            .iter()
            .map(|nt_original_name| {
                // Get child kind information for this non-terminal
                let (children_type, children) =
                    self.generate_child_kinds_info(nt_original_name, &nt_original_to_variant);

                let variant_name = nt_original_to_variant
                    .get(nt_original_name)
                    .cloned()
                    .unwrap_or_else(|| NamingHelper::to_upper_camel_case(nt_original_name));

                NonTerminalInfo {
                    name: nt_original_name.clone(),
                    variant: variant_name,
                    children,
                    kind: children_type,
                }
            })
            .chain(std::iter::once(NonTerminalInfo {
                name: "Root".to_string(),
                variant: "Root".to_string(),
                children: vec![Child {
                    kind: ChildAttribute::Normal,
                    name: NodeName::NonTerminal(NonTerminalName(start_symbol.to_string())),
                }],
                kind: ChildrenType::Sequence,
            }))
            .collect::<Vec<_>>();

        NodeTypesInfo {
            terminals: terminal_infos,
            non_terminals: non_terminal_infos,
        }
    }

    /// Generate child kinds information for a non-terminal.
    fn generate_child_kinds_info(
        &self,
        pr: &str,
        nt_original_to_variant: &HashMap<String, String>,
    ) -> (ChildrenType, Vec<Child>) {
        let alts = self.grammar_config.cfg.matching_productions(pr);
        if alts.is_empty() {
            panic!("Not supported: no productions for {}", pr);
        }

        if alts.len() == 2 {
            match (alts[0].1.get_attribute(), alts[1].1.get_attribute()) {
                (ProductionAttribute::CollectionStart, ProductionAttribute::AddToCollection) => (
                    ChildrenType::Recursion,
                    alts[1]
                        .1
                        .get_r()
                        .iter()
                        .map(|s| self.child_kind(s, nt_original_to_variant).unwrap())
                        .collect(),
                ),
                (ProductionAttribute::AddToCollection, ProductionAttribute::CollectionStart) => (
                    ChildrenType::Recursion,
                    alts[0]
                        .1
                        .get_r()
                        .iter()
                        .map(|s| self.child_kind(s, nt_original_to_variant).unwrap())
                        .collect(),
                ),
                (ProductionAttribute::OptionalNone, ProductionAttribute::OptionalSome) => (
                    ChildrenType::Option,
                    alts[1]
                        .1
                        .get_r()
                        .iter()
                        .map(|s| self.child_kind(s, nt_original_to_variant).unwrap())
                        .collect(),
                ),
                (ProductionAttribute::OptionalSome, ProductionAttribute::OptionalNone) => (
                    ChildrenType::Option,
                    alts[0]
                        .1
                        .get_r()
                        .iter()
                        .map(|s| self.child_kind(s, nt_original_to_variant).unwrap())
                        .collect(),
                ),
                _ => (
                    ChildrenType::OneOf,
                    alts.iter()
                        .map(|(_, p)| {
                            p.get_r()
                                .first()
                                .map(|s| self.child_kind(s, nt_original_to_variant).unwrap())
                                .expect("Expected a single child for each variant")
                        })
                        .collect(),
                ),
            }
        } else if alts.len() == 1 {
            (
                ChildrenType::Sequence,
                alts[0]
                    .1
                    .get_r()
                    .iter()
                    .map(|s| self.child_kind(s, nt_original_to_variant).unwrap())
                    .collect(),
            )
        } else {
            (
                ChildrenType::OneOf,
                alts.iter()
                    .map(|(_, p)| {
                        p.get_r()
                            .first()
                            .map(|s| self.child_kind(s, nt_original_to_variant).unwrap())
                            .expect("Expected a single child for each variant")
                    })
                    .collect(),
            )
        }
    }

    fn child_kind(
        &self,
        symbol: &GrammarSymbol,
        nt_original_to_variant: &HashMap<String, String>,
    ) -> Option<Child> {
        match symbol {
            GrammarSymbol::N(s, attrs, _, _) => {
                let attribute = match attrs {
                    SymbolAttribute::Option => ChildAttribute::Optional,
                    SymbolAttribute::RepetitionAnchor => ChildAttribute::Vec,
                    SymbolAttribute::Clipped => ChildAttribute::Clipped,
                    SymbolAttribute::None => ChildAttribute::Normal,
                };

                let variant_name = nt_original_to_variant
                    .get(s.as_str())
                    .cloned()
                    .unwrap_or_else(|| NamingHelper::to_upper_camel_case(s));

                Some(Child {
                    kind: attribute,
                    name: NodeName::NonTerminal(NonTerminalName(variant_name)),
                })
            }
            GrammarSymbol::T(crate::grammar::Terminal::Trm(
                terminal,
                _,
                _scanner_index,
                attrs,
                _,
                _,
                _,
            )) => {
                let attribute = match attrs {
                    SymbolAttribute::Option => ChildAttribute::Optional,
                    SymbolAttribute::RepetitionAnchor => ChildAttribute::Vec,
                    SymbolAttribute::Clipped => ChildAttribute::Clipped,
                    SymbolAttribute::None => ChildAttribute::Normal,
                };

                let terminal_name =
                    generate_terminal_name(terminal, None, &self.grammar_config.cfg);

                Some(Child {
                    kind: attribute,
                    name: NodeName::Terminal(TerminalName(terminal_name)),
                })
            }
            GrammarSymbol::T(crate::grammar::Terminal::Eps) => None,
            GrammarSymbol::T(crate::grammar::Terminal::End) => None,
            GrammarSymbol::S(_) => None,
            GrammarSymbol::Push(_) => None,
            GrammarSymbol::Pop => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
/// The name of a node
pub enum NodeName {
    /// A terminal
    Terminal(TerminalName),
    /// A non-terminal
    NonTerminal(NonTerminalName),
}

/// The name of a terminal
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TerminalName(pub String);

/// The name of a non-terminal
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct NonTerminalName(pub String);

/// Information about the node types
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct NodeTypesInfo {
    /// The terminals
    pub terminals: Vec<TerminalInfo>,
    /// The non-terminals
    pub non_terminals: Vec<NonTerminalInfo>,
}

/// Information about the terminals
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TerminalInfo {
    /// The name of the terminal
    pub name: String,
    /// The enum variant name for this terminal in the generated TerminalKind enum
    pub variant: String,
    /// The index of this terminal in the grammar
    pub index: usize,
}

/// Information about the non-terminals
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct NonTerminalInfo {
    /// The name of the non-terminal
    pub name: String,
    /// The enum variant name for this non-terminal in the generated NonTerminalKind enum
    pub variant: String,
    /// The children of the non-terminal
    pub children: Vec<Child>,
    /// The kind of the non-terminal
    pub kind: ChildrenType,
}

/// A child of a non-terminal
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Child {
    /// The attribute of the child
    pub kind: ChildAttribute,
    /// The name of the child
    pub name: NodeName,
}

/// The children of the non-terminal
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum ChildrenType {
    /// The children are a sequence
    Sequence,
    /// The children are one of the alternatives
    OneOf,
    /// After the children, the same non-terminal is exists. The children are optional. And the last child is the same non-terminal as self.
    Recursion,
    /// The children are optional
    Option,
}

/// The attribute of a child
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum ChildAttribute {
    /// The child is clipped
    Clipped,
    /// The child is normal
    Normal,
    /// The child is optional
    Optional,
    /// The child is a vector
    Vec,
}
