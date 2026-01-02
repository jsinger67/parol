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
                // Get the structure for this non-terminal
                let structure = self.generate_structure(nt_original_name, &nt_original_to_variant);

                let variant_name = nt_original_to_variant
                    .get(nt_original_name)
                    .cloned()
                    .unwrap_or_else(|| NamingHelper::to_upper_camel_case(nt_original_name));

                NonTerminalInfo {
                    name: nt_original_name.clone(),
                    variant: variant_name,
                    structure,
                }
            })
            .chain(std::iter::once({
                NonTerminalInfo {
                    name: "Root".to_string(),
                    variant: "Root".to_string(),
                    structure: NonTerminalStructure::Sequence(vec![Child {
                        kind: ChildAttribute::Normal,
                        name: NodeName::NonTerminal(NonTerminalName(start_symbol.to_string())),
                    }]),
                }
            }))
            .collect::<Vec<_>>();

        NodeTypesInfo {
            terminals: terminal_infos,
            non_terminals: non_terminal_infos,
        }
    }

    /// Generate the structure for a non-terminal.
    fn generate_structure(
        &self,
        pr: &str,
        nt_original_to_variant: &HashMap<String, String>,
    ) -> NonTerminalStructure {
        let alts = self.grammar_config.cfg.matching_productions(pr);
        if alts.is_empty() {
            panic!("Not supported: no productions for {pr}");
        }

        // Helper to collect all children from a production
        let collect_children = |prod: &crate::Pr| -> Vec<Child> {
            prod.get_r()
                .iter()
                .map(|s| self.child_kind(s, nt_original_to_variant).unwrap())
                .collect()
        };

        // Single production = Sequence
        if alts.len() == 1 {
            return NonTerminalStructure::Sequence(collect_children(alts[0].1));
        }

        // Two productions - check for special cases (Option, Recursion)
        if alts.len() == 2 {
            match (alts[0].1.get_attribute(), alts[1].1.get_attribute()) {
                // Recursion: CollectionStart | AddToCollection
                (ProductionAttribute::CollectionStart, ProductionAttribute::AddToCollection) => {
                    return NonTerminalStructure::Recursion(collect_children(alts[1].1));
                }
                (ProductionAttribute::AddToCollection, ProductionAttribute::CollectionStart) => {
                    return NonTerminalStructure::Recursion(collect_children(alts[0].1));
                }
                // Option: OptionalNone | OptionalSome
                (ProductionAttribute::OptionalNone, ProductionAttribute::OptionalSome) => {
                    return NonTerminalStructure::Option(collect_children(alts[1].1));
                }
                (ProductionAttribute::OptionalSome, ProductionAttribute::OptionalNone) => {
                    return NonTerminalStructure::Option(collect_children(alts[0].1));
                }
                _ => {}
            }
        }

        // Default: OneOf with all alternatives and all their children
        NonTerminalStructure::OneOf(alts.iter().map(|(_, p)| collect_children(p)).collect())
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
                l,
            )) => {
                let attribute = match attrs {
                    SymbolAttribute::Option => ChildAttribute::Optional,
                    SymbolAttribute::RepetitionAnchor => ChildAttribute::Vec,
                    SymbolAttribute::Clipped => ChildAttribute::Clipped,
                    SymbolAttribute::None => ChildAttribute::Normal,
                };

                let terminal_name =
                    generate_terminal_name(terminal, None, l.as_ref(), &self.grammar_config.cfg);

                Some(Child {
                    kind: attribute,
                    name: NodeName::Terminal(TerminalName(terminal_name)),
                })
            }
            GrammarSymbol::T(crate::grammar::Terminal::Eps) => None,
            GrammarSymbol::T(crate::grammar::Terminal::End) => None,
            _ => unreachable!(
                "Scanner switching directives have been removed from the grammar syntax."
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
/// The name of a node
pub enum NodeName {
    /// A terminal
    Terminal(TerminalName),
    /// A non-terminal
    NonTerminal(NonTerminalName),
}

/// The name of a terminal
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TerminalName(pub String);

/// The name of a non-terminal
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct NonTerminalName(pub String);

/// Information about the node types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct NodeTypesInfo {
    /// The terminals
    pub terminals: Vec<TerminalInfo>,
    /// The non-terminals
    pub non_terminals: Vec<NonTerminalInfo>,
}

/// Information about the terminals
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct NonTerminalInfo {
    /// The name of the non-terminal
    pub name: String,
    /// The enum variant name for this non-terminal in the generated NonTerminalKind enum
    pub variant: String,
    /// The structure of the non-terminal as an ADT (new API)
    pub structure: NonTerminalStructure,
}

/// A child of a non-terminal
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Child {
    /// The attribute of the child
    pub kind: ChildAttribute,
    /// The name of the child
    pub name: NodeName,
}

/// The children of the non-terminal
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
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

/// The structure of a non-terminal
/// This provides full information about all children in each alternative.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum NonTerminalStructure {
    /// Single production with all its children
    Sequence(Vec<Child>),

    /// Multiple alternatives, each with all its children
    OneOf(Vec<Vec<Child>>),

    /// Optional: None | Some(children)
    Option(Vec<Child>),

    /// Recursion: Base | Recursive(children)
    Recursion(Vec<Child>),
}

impl NonTerminalStructure {
    /// Returns the ChildrenType
    pub fn kind(&self) -> ChildrenType {
        match self {
            Self::Sequence(_) => ChildrenType::Sequence,
            Self::OneOf(_) => ChildrenType::OneOf,
            Self::Option(_) => ChildrenType::Option,
            Self::Recursion(_) => ChildrenType::Recursion,
        }
    }
}

/// The attribute of a child
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generators::grammar_type_generator::GrammarTypeInfo;
    use crate::obtain_grammar_config_from_string;

    /// Generate NodeTypesInfo from a grammar string
    fn generate_node_types(input: &str) -> NodeTypesInfo {
        let grammar_config =
            obtain_grammar_config_from_string(input, false).expect("Failed to parse grammar");
        let mut type_info = GrammarTypeInfo::try_new("Test").expect("Failed to create type info");
        type_info
            .build(&grammar_config)
            .expect("Failed to build type info");
        let exporter = NodeTypesExporter::new(&grammar_config, &type_info);
        exporter.generate()
    }

    /// Find a non-terminal by name from NodeTypesInfo
    fn find_nt<'a>(node_types: &'a NodeTypesInfo, name: &str) -> &'a NonTerminalInfo {
        node_types
            .non_terminals
            .iter()
            .find(|nt| nt.name == name)
            .unwrap_or_else(|| panic!("Non-terminal '{}' not found", name))
    }

    /// Convenience: Generate and find non-terminal in one call
    fn get_nt(input: &str, name: &str) -> NonTerminalInfo {
        let node_types = generate_node_types(input);
        find_nt(&node_types, name).clone()
    }

    /// Get all terminals from a grammar string
    fn get_terminals(input: &str) -> Vec<TerminalInfo> {
        generate_node_types(input).terminals
    }

    fn nt(name: &str) -> Child {
        Child {
            kind: ChildAttribute::Normal,
            name: NodeName::NonTerminal(NonTerminalName(name.to_string())),
        }
    }

    fn t(name: &str) -> Child {
        Child {
            kind: ChildAttribute::Normal,
            name: NodeName::Terminal(TerminalName(name.to_string())),
        }
    }

    fn opt_nt(name: &str) -> Child {
        Child {
            kind: ChildAttribute::Optional,
            name: NodeName::NonTerminal(NonTerminalName(name.to_string())),
        }
    }

    fn vec_nt(name: &str) -> Child {
        Child {
            kind: ChildAttribute::Vec,
            name: NodeName::NonTerminal(NonTerminalName(name.to_string())),
        }
    }

    fn clipped_nt(name: &str) -> Child {
        Child {
            kind: ChildAttribute::Clipped,
            name: NodeName::NonTerminal(NonTerminalName(name.to_string())),
        }
    }

    fn clipped_t(name: &str) -> Child {
        Child {
            kind: ChildAttribute::Clipped,
            name: NodeName::Terminal(TerminalName(name.to_string())),
        }
    }

    fn sequence(name: &str, children: Vec<Child>) -> NonTerminalInfo {
        NonTerminalInfo {
            name: name.to_string(),
            variant: name.to_string(),
            structure: NonTerminalStructure::Sequence(children),
        }
    }

    fn one_of(name: &str, alternatives: Vec<Vec<Child>>) -> NonTerminalInfo {
        NonTerminalInfo {
            name: name.to_string(),
            variant: name.to_string(),
            structure: NonTerminalStructure::OneOf(alternatives),
        }
    }

    fn option(name: &str, children: Vec<Child>) -> NonTerminalInfo {
        NonTerminalInfo {
            name: name.to_string(),
            variant: name.to_string(),
            structure: NonTerminalStructure::Option(children),
        }
    }

    fn recursion(name: &str, children: Vec<Child>) -> NonTerminalInfo {
        NonTerminalInfo {
            name: name.to_string(),
            variant: name.to_string(),
            structure: NonTerminalStructure::Recursion(children),
        }
    }

    #[test]
    fn test_one_of_structure() {
        let s = get_nt(
            r#"
            %start S
            %%
            S: A B | C D E ;
            A: "a" ;
            B: "b" ;
            C: "c" ;
            D: "d" ;
            E: "e" ;
        "#,
            "S",
        );

        assert_eq!(
            s,
            one_of(
                "S",
                vec![vec![nt("A"), nt("B")], vec![nt("C"), nt("D"), nt("E")]]
            )
        );
    }

    #[test]
    fn test_sequence_structure() {
        let s = get_nt(
            r#"
            %start S
            %%
            S: A B C ;
            A: "a" ;
            B: "b" ;
            C: "c" ;
        "#,
            "S",
        );

        assert_eq!(s, sequence("S", vec![nt("A"), nt("B"), nt("C")]));
    }

    #[test]
    fn test_option_structure() {
        let input = r#"
            %start S
            %%
            S: [ A ] ;
            A: "a" ;
        "#;
        let node_types = generate_node_types(input);

        let s = find_nt(&node_types, "S");
        let s_opt = find_nt(&node_types, "SOpt");

        assert_eq!(s, &sequence("S", vec![opt_nt("SOpt")]));
        assert_eq!(s_opt, &option("SOpt", vec![nt("A")]));
    }

    #[test]
    fn test_recursion_structure() {
        let input = r#"
            %start S
            %%
            S: { A } ;
            A: "a" ;
        "#;
        let node_types = generate_node_types(input);

        let s = find_nt(&node_types, "S");
        let s_list = find_nt(&node_types, "SList");

        assert_eq!(s, &sequence("S", vec![vec_nt("SList")]));
        assert_eq!(s_list, &recursion("SList", vec![nt("A"), nt("SList")]));
    }

    #[test]
    fn test_terminal_children() {
        let s = get_nt(
            r#"
            %start S
            %%
            S: "hello" "world" ;
        "#,
            "S",
        );

        assert_eq!(s, sequence("S", vec![t("Hello"), t("World")]));
    }

    fn terminal(name: &str, index: usize) -> TerminalInfo {
        TerminalInfo {
            name: name.to_string(),
            variant: name.to_string(),
            index,
        }
    }

    #[test]
    fn test_terminal_info_generation() {
        let terminals = get_terminals(
            r#"
            %start S
            %%
            S: "foo" "bar" ;
        "#,
        );

        assert_eq!(
            terminals,
            vec![
                terminal("NewLine", 1),
                terminal("Whitespace", 2),
                terminal("LineComment", 3),
                terminal("BlockComment", 4),
                terminal("Foo", 5),
                terminal("Bar", 6),
            ]
        );
    }

    #[test]
    fn test_root_node_generation() {
        let root = get_nt(
            r#"
            %start MyStart
            %%
            MyStart: "a" ;
        "#,
            "Root",
        );

        assert_eq!(root, sequence("Root", vec![nt("MyStart")]));
    }

    #[test]
    fn test_clipped_non_terminal() {
        let s = get_nt(
            r#"
            %start S
            %%
            S: A^ ;
            A: "a" ;
        "#,
            "S",
        );

        assert_eq!(s, sequence("S", vec![clipped_nt("A")]));
    }

    #[test]
    fn test_clipped_terminal() {
        let s = get_nt(
            r#"
            %start S
            %%
            S: "foo"^ ;
        "#,
            "S",
        );

        assert_eq!(s, sequence("S", vec![clipped_t("S")]));
    }

    #[test]
    fn test_optional_terminal() {
        let input = r#"
            %start S
            %%
            S: [ "foo" ] ;
        "#;
        let node_types = generate_node_types(input);

        let s = find_nt(&node_types, "S");
        let s_opt = find_nt(&node_types, "SOpt");

        assert_eq!(s, &sequence("S", vec![opt_nt("SOpt")]));
        assert_eq!(s_opt, &option("SOpt", vec![t("Foo")]));
    }

    #[test]
    fn test_vec_terminal() {
        let input = r#"
            %start S
            %%
            S: { "foo" } ;
        "#;
        let node_types = generate_node_types(input);

        let s = find_nt(&node_types, "S");
        let s_list = find_nt(&node_types, "SList");

        assert_eq!(s, &sequence("S", vec![vec_nt("SList")]));
        assert_eq!(s_list, &recursion("SList", vec![t("Foo"), nt("SList")]));
    }
}
