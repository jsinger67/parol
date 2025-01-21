use std::io::Write;

use crate::generators::symbol_table::TypeEntrails;
use crate::generators::symbol_table_facade::{InstanceFacade, SymbolFacade as _};
use crate::generators::template_data::NameToNonTerminalVariant;
use crate::utils::str_iter::IteratorExt;
use crate::StrVec;

use super::grammar_type_generator::GrammarTypeInfo;
use super::symbol_table::{NodeType, SymbolId};
use super::symbol_table_facade::TypeFacade;
use super::template_data::{ChildKind, DisplayArm, ExpectedChildrenArm, NumToTerminalVariant};
use super::GrammarConfig;

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
            use parol_runtime::parser::parse_tree_type::{ChildKind, ExpectedChildren, TerminalEnum, NonTerminalEnum, ExpectedChildrenKinds};
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
        let SyntreeNodeTypesGenerator {
            grammar_type_info, ..
        } = self;

        let ast_members = grammar_type_info
            .symbol_table()
            .members(grammar_type_info.ast_enum_type)?;

        write!(f, "impl ExpectedChildren<TerminalKind, NonTerminalKind> for NonTerminalKind {{ fn expected_children(&self) -> ExpectedChildrenKinds<TerminalKind, NonTerminalKind> {{ match self {{")?;

        for member in ast_members {
            let symbol_type = grammar_type_info.symbol_table().symbol_as_type(*member);
            let TypeEntrails::EnumVariant(member) = symbol_type.entrails() else {
                unreachable!();
            };
            let type_facade = grammar_type_info.symbol_table().symbol_as_type(*member);
            let symbol_type = type_facade.entrails();
            let member = match symbol_type {
                TypeEntrails::Vec(member) => member,
                TypeEntrails::Option(member) => member,
                _ => member,
            };

            self.generate_non_terminal_node_types_impl_single(f, *member)?;
        }

        let name = self.grammar_config.cfg.get_start_symbol();

        f.write_fmt(ume::ume! {
            Self::Root => ExpectedChildrenKinds::Sequence(&[
                ChildKind::NonTerminal(NonTerminalKind::#name),
            ]),
        })?;

        write!(f, "}} }} }}")?;

        Ok(())
    }

    fn generate_non_terminal_node_types_impl_single(
        &self,
        f: &mut impl Write,
        symbol: SymbolId,
    ) -> anyhow::Result<()> {
        let SyntreeNodeTypesGenerator {
            grammar_type_info, ..
        } = self;

        println!("================");

        let mut children = vec![];

        let name = grammar_type_info.symbol_table().name(symbol);
        println!("name: {}", name);

        let members = grammar_type_info.symbol_table().members(symbol)?;
        println!("members: {:?}", members);

        let symbol_type = grammar_type_info.symbol_table().symbol_as_type(symbol);
        println!("symbol_type: {:?}", symbol_type.entrails());

        let is_enum = *symbol_type.entrails() == TypeEntrails::Enum;

        for member in members {
            let name = grammar_type_info.symbol_table().name(*member);
            println!("name: {}", name);
            let member_type = if is_enum {
                grammar_type_info.symbol_table().symbol_as_type(*member)
            } else {
                let inst = grammar_type_info.symbol_table().symbol_as_instance(*member);
                grammar_type_info
                    .symbol_table()
                    .symbol_as_type(inst.type_id())
            };

            println!("member_type: {:?}", member_type.entrails());
            match member_type
                .entrails()
                .node_type(*member, grammar_type_info.symbol_table())?
            {
                NodeType::None => {
                    children.push(ChildKind::Terminal(symbol_type.name()));
                }
                NodeType::Terminal(symbol_id) => {
                    // let name = grammar_type_info.symbol_table().type_name(symbol_id)?;
                    // children.push(ChildKind::Terminal(name));
                    unreachable!();
                }
                NodeType::NonTerminal(symbol_id) => {
                    let name = grammar_type_info.symbol_table().name(symbol_id);
                    children.push(ChildKind::NonTerminal(name));
                }
                NodeType::NonTerminalVec(symbol_id) => {
                    let name = grammar_type_info.symbol_table().name(symbol_id);
                    children.push(ChildKind::Vec(name));
                }
                NodeType::NonTerminalOption(symbol_id) => {
                    let name = grammar_type_info.symbol_table().name(symbol_id);
                    if is_enum {
                        children.push(ChildKind::NonTerminal(name));
                        children.push(ChildKind::Optional(name));
                    } else {
                        children.push(ChildKind::Optional(name));
                    }
                }
            }
        }

        let kind = if is_enum { "OneOf" } else { "Sequence" };
        f.write_fmt(format_args!(
            "Self::{} => ExpectedChildrenKinds::{}(&[{}]),",
            name,
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
}
