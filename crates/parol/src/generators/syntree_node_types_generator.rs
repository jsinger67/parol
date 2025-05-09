use std::collections::BTreeSet;
use std::io::Write;

use crate::generators::template_data::NameToNonTerminalVariant;
use crate::grammar::ProductionAttribute;
use crate::utils::str_iter::IteratorExt;
use crate::{StrVec, SymbolAttribute, Terminal};

use super::grammar_type_generator::GrammarTypeInfo;
use super::template_data::{
    ChildAttribute, ChildKind, ChildNodeKind, DisplayArm, NumToTerminalVariant,
};
use super::{GrammarConfig, generate_terminal_name};

/// Syntree node types generator.
pub struct SyntreeNodeTypesGenerator<'a> {
    grammar_config: &'a GrammarConfig,
    grammar_type_info: &'a GrammarTypeInfo,
    terminals: Vec<(usize, String)>,
}

impl<'a> SyntreeNodeTypesGenerator<'a> {
    /// Create a new syntree node types generator.
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

impl SyntreeNodeTypesGenerator<'_> {
    fn generate_imports(&self, f: &mut impl Write) -> anyhow::Result<()> {
        f.write_fmt(ume::ume! {
            use parol_runtime::parser::parse_tree_type::{NodeKind, ExpectedChildren, TerminalEnum, NonTerminalEnum, ExpectedChildrenKinds, ChildAttribute, ChildKind, Node};
        })?;
        Ok(())
    }

    /// Generate the AST enum type.
    fn generate_ast_enum_type(&self, f: &mut impl Write) -> anyhow::Result<()> {
        let SyntreeNodeTypesGenerator {
            grammar_type_info, ..
        } = self;

        let mut non_terminal = StrVec::new(0);
        non_terminal.push(String::default());
        non_terminal.push("All possible non-terminal kinds".to_string());
        non_terminal.push(String::default());
        let mut terminal = StrVec::new(0);
        terminal.push(String::default());
        terminal.push("All possible terminal kinds".to_string());
        terminal.push(String::default());
        let non_terminal_enum = grammar_type_info
            .generate_non_terminal_enum_type()
            .into_iter()
            .map(|(variant, _)| format!("{}", ume::ume!(#variant,)))
            .collect::<StrVec>();
        let terminal_enum = self
            .terminals
            .iter()
            .map(|(_, name)| format!("{}", ume::ume!(#name,)))
            .into_str_iter();
        f.write_fmt(ume::ume! {
            #[allow(dead_code)]
            #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
            pub enum NonTerminalKind {
                #non_terminal_enum
            }
        })?;

        f.write_fmt(ume::ume! {
            #[allow(dead_code)]
            #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
            pub enum TerminalKind {
                #terminal_enum
            }
        })?;

        Ok(())
    }

    /// Generate the implementation of the AST enum.
    fn generate_ast_enum_impl(&self, f: &mut impl Write) -> anyhow::Result<()> {
        let SyntreeNodeTypesGenerator {
            grammar_type_info, ..
        } = self;

        let num_to_terminal_match_arms = self
            .terminals
            .iter()
            .map(|(i, t)| NumToTerminalVariant {
                variant: t.to_string(),
                prod_num: *i,
            })
            .into_str_iter();

        let non_terminal_match_arms = grammar_type_info
            .generate_non_terminal_enum_type()
            .into_iter()
            .map(|(variant, name)| NameToNonTerminalVariant {
                variant: variant.to_owned(),
                name: name.to_owned(),
            })
            .into_str_iter();

        f.write_fmt(ume::ume! {
            impl TerminalEnum for TerminalKind {
                fn from_terminal_index(index: u16) -> Self {
                    match index {
                        #num_to_terminal_match_arms
                        _ => panic!("Invalid terminal index: {}", index),
                    }
                }

                fn is_builtin_terminal(&self) -> bool {
                    matches!(self, TerminalKind::NewLine | TerminalKind::Whitespace | TerminalKind::LineComment | TerminalKind::BlockComment)
                }

                fn is_builtin_new_line(&self) -> bool {
                    matches!(self, TerminalKind::NewLine)
                }

                fn is_builtin_whitespace(&self) -> bool {
                    matches!(self, TerminalKind::Whitespace)
                }

                fn is_builtin_line_comment(&self) -> bool {
                    matches!(self, TerminalKind::LineComment)
                }

                fn is_builtin_block_comment(&self) -> bool {
                    matches!(self, TerminalKind::BlockComment)
                }
            }
        })?;

        write!(f, "\n\n")?;

        f.write_fmt(ume::ume! {
            impl NonTerminalEnum for NonTerminalKind {
                fn from_non_terminal_name(name: &str) -> Self {
                    match name {
                        #non_terminal_match_arms
                        _ => panic!("Invalid non-terminal name: {}", name),
                    }
                }
            }
        })?;

        Ok(())
    }

    fn generate_display_impl(&self, f: &mut impl Write) -> anyhow::Result<()> {
        let SyntreeNodeTypesGenerator {
            grammar_type_info, ..
        } = self;

        let terminal_arms = self
            .terminals
            .iter()
            .map(|(_i, t)| DisplayArm {
                variant: t,
                value: t,
            })
            .into_str_iter();

        let non_terminals = grammar_type_info.generate_non_terminal_enum_type();
        let non_terminal_arms = non_terminals
            .iter()
            .map(|(variant, name)| {
                if variant == "Root" {
                    DisplayArm {
                        variant,
                        value: "Root",
                    }
                } else {
                    DisplayArm {
                        variant,
                        value: name,
                    }
                }
            })
            .into_str_iter();

        f.write_fmt(ume::ume! {
            impl std::fmt::Display for TerminalKind {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {
                        #terminal_arms
                    }
                }
            }
        })?;

        write!(f, "\n\n")?;

        f.write_fmt(ume::ume! {
            impl std::fmt::Display for NonTerminalKind {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {
                        #non_terminal_arms
                    }
                }
            }
        })?;

        Ok(())
    }

    fn generate_non_terminal_node_types_impl(&self, f: &mut impl Write) -> anyhow::Result<()> {
        write!(
            f,
            "impl ExpectedChildren<TerminalKind, NonTerminalKind> for NonTerminalKind {{ fn expected_children(&self) -> ExpectedChildrenKinds<TerminalKind, NonTerminalKind> {{ match self {{"
        )?;

        for pr in self.grammar_config.cfg.get_non_terminal_set() {
            self.generate_non_terminal_node_types_impl_single(f, &pr)?;
        }

        let name = self.grammar_config.cfg.get_start_symbol();

        f.write_fmt(ume::ume! {
            Self::Root => ExpectedChildrenKinds::Sequence(&[
                ChildKind {
                    kind: NodeKind::NonTerminal(NonTerminalKind::#name),
                    attribute: ChildAttribute::Normal,
                },
            ]),
        })?;

        write!(f, "}} }} }}")?;

        Ok(())
    }

    fn generate_non_terminal_node_types_impl_single(
        &self,
        f: &mut impl Write,
        pr: &str,
    ) -> anyhow::Result<()> {
        let child_kinds = self.generate_child_kinds(pr);

        let kind = match child_kinds.kind {
            ChildKindsKind::OneOf => "OneOf",
            ChildKindsKind::Sequence => "Sequence",
            ChildKindsKind::Recursion => "Recursion",
            ChildKindsKind::Option => "Option",
        };
        let children = child_kinds
            .children
            .iter()
            .map(|child| format!("{},", child))
            .into_str_iter();
        f.write_fmt(ume::ume! {
            Self::#pr => ExpectedChildrenKinds::#kind(&[#children]),
        })?;
        Ok(())
    }

    fn generate_node_wrappers(&self, f: &mut impl Write) -> anyhow::Result<()> {
        for pr in self.grammar_config.cfg.get_non_terminal_set() {
            self.generate_node_wrapper(f, &pr)?;
        }
        Ok(())
    }

    fn generate_node_wrapper(&self, f: &mut impl Write, pr: &str) -> anyhow::Result<()> {
        let child_kinds = self.generate_child_kinds(pr);

        match child_kinds.kind {
            ChildKindsKind::OneOf => self.generate_ast_enum(f, pr, child_kinds)?,
            ChildKindsKind::Sequence | ChildKindsKind::Recursion | ChildKindsKind::Option => {
                f.write_fmt(ume::ume! {
                    #[allow(dead_code)]
                    #[derive(Debug, Clone, Copy, PartialEq)]
                    pub struct #pr<T>(T);
                })?;
            }
        }

        f.write_fmt(ume::ume! {
            #[allow(dead_code)]
            impl<'a, N> #pr<N> where N: Node<'a, TerminalKind, NonTerminalKind>
        })?;

        write!(f, "{{")?;

        let child_kinds = self.generate_child_kinds(pr);

        match child_kinds.kind {
            ChildKindsKind::OneOf => self.generate_enum_new_impl(f, pr, child_kinds)?,
            ChildKindsKind::Sequence | ChildKindsKind::Recursion | ChildKindsKind::Option => {
                f.write_fmt(ume::ume! {
                    pub fn new(node: N) -> Self {
                        #pr(node)
                    }

                    pub fn node(&self) -> &N {
                        &self.0
                    }

                    pub fn node_mut(&mut self) -> &mut N {
                        &mut self.0
                    }
                })?;

                // Does not need to generate multiple find methods for the same child kind.
                let mut exists = BTreeSet::new();
                for child_kind in child_kinds.children {
                    if child_kind.attribute == ChildAttribute::Clipped {
                        continue;
                    }
                    if exists.insert(child_kind.print_find_method_name()) {
                        self.generate_find_methods_sequence_single(f, pr, child_kind)?;
                    }
                }
            }
        }

        write!(f, "}}")?;

        Ok(())
    }

    fn generate_ast_enum(
        &self,
        f: &mut impl Write,
        pr: &str,
        child_kinds: ChildKinds,
    ) -> anyhow::Result<()> {
        f.write_fmt(ume::ume! {
            #[allow(dead_code)]
            #[derive(Debug, Clone, Copy, PartialEq)]
            pub enum #pr<T>
        })?;
        write!(f, "{{")?;
        for child_kind in child_kinds.children {
            if child_kind.attribute == ChildAttribute::Clipped {
                continue;
            }
            let variant = child_kind.print_ast_enum_variant();
            write!(f, "{}", variant)?;
        }
        write!(f, "Invalid(T),")?;
        write!(f, "}}")?;
        Ok(())
    }

    fn generate_enum_new_impl(
        &self,
        f: &mut impl Write,
        pr: &str,
        child_kinds: ChildKinds,
    ) -> anyhow::Result<()> {
        let variants = child_kinds
            .children
            .iter()
            .filter(|child| child.attribute != ChildAttribute::Clipped)
            .map(|child| child.print_enum_new_match_arms())
            .into_str_iter();
        let node_match_arms = child_kinds
            .children
            .iter()
            .filter(|child| child.attribute != ChildAttribute::Clipped)
            .map(|child| child.print_enum_node_match_arms())
            .into_str_iter();
        let node_match_arms_mut = child_kinds
            .children
            .iter()
            .filter(|child| child.attribute != ChildAttribute::Clipped)
            .map(|child| child.print_enum_node_mut_match_arms())
            .into_str_iter();
        f.write_fmt(ume::ume! {
            pub fn new(node: N) -> Self {
                match node.kind() {
                    #variants
                    _ => #pr::Invalid(node),
                }
            }

            pub fn node(&self) -> &N {
                match self {
                    #node_match_arms
                    Self::Invalid(node) => node,
                }
            }

            pub fn node_mut(&mut self) -> &mut N {
                match self {
                    #node_match_arms_mut
                    Self::Invalid(node) => node,
                }
            }
        })?;
        Ok(())
    }

    fn generate_find_methods_sequence_single(
        &self,
        f: &mut impl Write,
        pr: &str,
        child_kind: ChildKind,
    ) -> anyhow::Result<()> {
        let method_name = child_kind.print_find_method_name();
        let child_kind_name = child_kind.print_node_kind();
        f.write_fmt(ume::ume! {
            pub fn #method_name(&self, cursor: usize) -> Result<Option<(usize, #pr<N>)>, N> {
                self.0.find_child(cursor, #child_kind_name).map(|option| option.map(|(i, node)| (i, #pr::new(node))))
            }
        })?;
        Ok(())
    }

    pub(crate) fn generate(&self, f: &mut impl Write) -> anyhow::Result<()> {
        self.generate_imports(f)?;
        self.generate_ast_enum_type(f)?;
        self.generate_ast_enum_impl(f)?;
        self.generate_display_impl(f)?;
        self.generate_non_terminal_node_types_impl(f)?;
        self.generate_node_wrappers(f)?;
        Ok(())
    }

    fn generate_child_kinds(&self, pr: &str) -> ChildKinds {
        let alts = self.grammar_config.cfg.matching_productions(pr);
        if alts.is_empty() {
            panic!("Not supported!");
        } else if alts.len() == 2 {
            match (alts[0].1.get_attribute(), alts[1].1.get_attribute()) {
                (ProductionAttribute::CollectionStart, ProductionAttribute::AddToCollection) => {
                    ChildKinds {
                        kind: ChildKindsKind::Recursion,
                        children: alts[1]
                            .1
                            .get_r()
                            .iter()
                            .filter_map(|s| self.child_kind(s))
                            .collect(),
                    }
                }
                (ProductionAttribute::AddToCollection, ProductionAttribute::CollectionStart) => {
                    ChildKinds {
                        kind: ChildKindsKind::Recursion,
                        children: alts[0]
                            .1
                            .get_r()
                            .iter()
                            .filter_map(|s| self.child_kind(s))
                            .collect(),
                    }
                }
                (ProductionAttribute::OptionalNone, ProductionAttribute::OptionalSome) => {
                    ChildKinds {
                        kind: ChildKindsKind::Option,
                        children: alts[1]
                            .1
                            .get_r()
                            .iter()
                            .filter_map(|s| self.child_kind(s))
                            .collect(),
                    }
                }
                (ProductionAttribute::OptionalSome, ProductionAttribute::OptionalNone) => {
                    ChildKinds {
                        kind: ChildKindsKind::Option,
                        children: alts[0]
                            .1
                            .get_r()
                            .iter()
                            .filter_map(|s| self.child_kind(s))
                            .collect(),
                    }
                }
                _ => ChildKinds {
                    kind: ChildKindsKind::OneOf,
                    children: alts
                        .into_iter()
                        .filter_map(|(_, p)| p.get_r().first().and_then(|s| self.child_kind(s)))
                        .collect(),
                },
            }
        } else if alts.len() == 1 {
            ChildKinds {
                kind: ChildKindsKind::Sequence,
                children: alts[0]
                    .1
                    .get_r()
                    .iter()
                    .filter_map(|s| self.child_kind(s))
                    .collect(),
            }
        } else {
            ChildKinds {
                kind: ChildKindsKind::OneOf,
                children: alts
                    .into_iter()
                    .filter_map(|(_, p)| p.get_r().first().and_then(|s| self.child_kind(s)))
                    .collect(),
            }
        }
    }

    fn child_kind(&self, symbol: &crate::Symbol) -> Option<ChildKind> {
        match symbol {
            crate::Symbol::N(s, attrs, _, _) => match attrs {
                SymbolAttribute::Option => Some(ChildKind {
                    kind: ChildNodeKind::NonTerminal,
                    name: s.clone(),
                    attribute: ChildAttribute::Optional,
                }),
                SymbolAttribute::RepetitionAnchor => Some(ChildKind {
                    kind: ChildNodeKind::NonTerminal,
                    name: s.clone(),
                    attribute: ChildAttribute::Vec,
                }),
                SymbolAttribute::Clipped => Some(ChildKind {
                    kind: ChildNodeKind::NonTerminal,
                    name: s.clone(),
                    attribute: ChildAttribute::Clipped,
                }),
                SymbolAttribute::None => Some(ChildKind {
                    kind: ChildNodeKind::NonTerminal,
                    name: s.clone(),
                    attribute: ChildAttribute::Normal,
                }),
            },
            crate::Symbol::T(Terminal::Trm(terminal, _, _, attrs, _, _, _)) => match attrs {
                SymbolAttribute::Option => Some(ChildKind {
                    kind: ChildNodeKind::Terminal,
                    name: generate_terminal_name(terminal, None, &self.grammar_config.cfg),
                    attribute: ChildAttribute::Optional,
                }),
                SymbolAttribute::RepetitionAnchor => Some(ChildKind {
                    kind: ChildNodeKind::Terminal,
                    name: generate_terminal_name(terminal, None, &self.grammar_config.cfg),
                    attribute: ChildAttribute::Vec,
                }),
                SymbolAttribute::Clipped => Some(ChildKind {
                    kind: ChildNodeKind::Terminal,
                    name: generate_terminal_name(terminal, None, &self.grammar_config.cfg),
                    attribute: ChildAttribute::Clipped,
                }),
                SymbolAttribute::None => Some(ChildKind {
                    kind: ChildNodeKind::Terminal,
                    name: generate_terminal_name(terminal, None, &self.grammar_config.cfg),
                    attribute: ChildAttribute::Normal,
                }),
            },
            crate::Symbol::T(Terminal::Eps) => None,
            crate::Symbol::T(Terminal::End) => None,
            crate::Symbol::S(_) => None,
            crate::Symbol::Push(_) => None,
            crate::Symbol::Pop => None,
        }
    }
}

enum ChildKindsKind {
    Sequence,
    OneOf,
    Recursion,
    Option,
}

struct ChildKinds {
    kind: ChildKindsKind,
    children: Vec<ChildKind>,
}
