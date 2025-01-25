use std::io::Write;

use crate::generators::template_data::NameToNonTerminalVariant;
use crate::grammar::ProductionAttribute;
use crate::utils::str_iter::IteratorExt;
use crate::{StrVec, Terminal};

use super::grammar_type_generator::GrammarTypeInfo;
use super::template_data::{ChildKind, DisplayArm, NumToTerminalVariant};
use super::{generate_terminal_name, GrammarConfig};

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
            use parol_runtime::parser::parse_tree_type::{NodeKind, ExpectedChildren, TerminalEnum, NonTerminalEnum, ExpectedChildrenKinds};
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

                fn is_builtin_new_line(&self) -> bool {
                    matches!(self, TerminalKind::NewLine)
                }

                fn is_builtin_whitespace(&self) -> bool {
                    matches!(self, TerminalKind::Whitespace)
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

        let non_terminal_arms = grammar_type_info
            .generate_non_terminal_enum_type()
            .map(|(variant, name)| DisplayArm {
                variant,
                value: name,
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
        write!(f, "impl ExpectedChildren<TerminalKind, NonTerminalKind> for NonTerminalKind {{ fn expected_children(&self) -> ExpectedChildrenKinds<TerminalKind, NonTerminalKind> {{ match self {{")?;

        for pr in self.grammar_config.cfg.get_non_terminal_set() {
            self.generate_non_terminal_node_types_impl_single(f, &pr)?;
        }

        let name = self.grammar_config.cfg.get_start_symbol();

        f.write_fmt(ume::ume! {
            Self::Root => ExpectedChildrenKinds::Sequence(&[
                NodeKind::NonTerminal(NonTerminalKind::#name),
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
        let alts = self.grammar_config.cfg.matching_productions(pr);
        let is_enum;
        let mut children: Vec<ChildKind> = Vec::new();
        if alts.len() == 2 {
            match (alts[0].1.get_attribute(), alts[1].1.get_attribute()) {
                (ProductionAttribute::CollectionStart, ProductionAttribute::AddToCollection) => {
                    children.extend(alts[1].1.get_r().iter().filter_map(|s| self.child_kind(s)));
                    is_enum = false;
                }
                (ProductionAttribute::AddToCollection, ProductionAttribute::CollectionStart) => {
                    children.extend(alts[0].1.get_r().iter().filter_map(|s| self.child_kind(s)));
                    is_enum = false;
                }
                _ => {
                    is_enum = true;
                }
            }
        } else if alts.is_empty() {
            panic!("Not supported!");
        } else if alts.len() == 1 {
            children.extend(alts[0].1.get_r().iter().filter_map(|s| self.child_kind(s)));
            is_enum = false;
        } else {
            children.extend(
                alts.iter()
                    .filter_map(|(_, p)| p.get_r().first().and_then(|s| self.child_kind(s))),
            );
            is_enum = true;
        }

        let kind = if is_enum { "OneOf" } else { "Sequence" };
        f.write_fmt(format_args!(
            "Self::{} => ExpectedChildrenKinds::{}(&[{}]),",
            pr,
            kind,
            children
                .iter()
                .map(|child| format!("{}", child))
                .collect::<Vec<_>>()
                .join(", ")
        ))?;
        Ok(())
    }

    pub(crate) fn generate(&self, f: &mut impl Write) -> anyhow::Result<()> {
        self.generate_imports(f)?;
        self.generate_ast_enum_type(f)?;
        self.generate_ast_enum_impl(f)?;
        self.generate_display_impl(f)?;
        self.generate_non_terminal_node_types_impl(f)?;
        Ok(())
    }

    fn child_kind(&self, symbol: &crate::Symbol) -> Option<ChildKind> {
        match symbol {
            crate::Symbol::N(_, _, _) => Some(ChildKind::NonTerminal(symbol.to_string())),
            crate::Symbol::T(Terminal::Trm(terminal, _, _, _, _, _)) => Some(ChildKind::Terminal(
                generate_terminal_name(terminal, None, &self.grammar_config.cfg),
            )),
            crate::Symbol::T(Terminal::Eps) => None,
            crate::Symbol::T(Terminal::End) => None,
            crate::Symbol::S(_) => None,
            crate::Symbol::Push(_) => None,
            crate::Symbol::Pop => None,
        }
    }
}
