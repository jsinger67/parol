use std::io::Write;

use crate::StrVec;
use crate::generators::template_data::NameToNonTerminalVariant;
use crate::utils::str_iter::IteratorExt;

use super::GrammarConfig;
use super::grammar_type_generator::{GrammarTypeInfo, NonTerminalEnumType};
use super::template_data::{DisplayArm, NumToTerminalVariant};

/// Syntree node types generator.
pub struct NodeKindTypesGenerator<'a> {
    grammar_type_info: &'a GrammarTypeInfo,
    terminals: Vec<(usize, String)>,
}

impl<'a> NodeKindTypesGenerator<'a> {
    /// Create a new syntree node types generator.
    ///
    /// Arguments must be properly prepared before calling this function.
    pub fn new(grammar_config: &'a GrammarConfig, grammar_type_info: &'a GrammarTypeInfo) -> Self {
        Self {
            grammar_type_info,
            terminals: grammar_config.generate_terminal_names(),
        }
    }
}

impl NodeKindTypesGenerator<'_> {
    /// Generate the AST enum type.
    fn generate_ast_enum_type(&self, f: &mut impl Write) -> anyhow::Result<()> {
        let NodeKindTypesGenerator {
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
            .map(|NonTerminalEnumType { name, .. }| format!("{}", ume::ume!(#name,)))
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
        let NodeKindTypesGenerator {
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
            .map(
                |NonTerminalEnumType {
                     name,
                     from_non_terminal_name,
                     ..
                 }| NameToNonTerminalVariant {
                    variant: name.to_owned(),
                    name: from_non_terminal_name.to_owned(),
                },
            )
            .into_str_iter();

        f.write_fmt(ume::ume! {
            #[allow(dead_code)]
            impl TerminalKind {
                pub fn from_terminal_index(index: u16) -> Self {
                    match index {
                        #num_to_terminal_match_arms
                        _ => panic!("Invalid terminal index: {}", index),
                    }
                }

                pub fn is_builtin_terminal(&self) -> bool {
                    matches!(self, TerminalKind::NewLine | TerminalKind::Whitespace | TerminalKind::LineComment | TerminalKind::BlockComment)
                }

                pub fn is_builtin_new_line(&self) -> bool {
                    matches!(self, TerminalKind::NewLine)
                }

                pub fn is_builtin_whitespace(&self) -> bool {
                    matches!(self, TerminalKind::Whitespace)
                }

                pub fn is_builtin_line_comment(&self) -> bool {
                    matches!(self, TerminalKind::LineComment)
                }

                pub fn is_builtin_block_comment(&self) -> bool {
                    matches!(self, TerminalKind::BlockComment)
                }
            }
        })?;

        write!(f, "\n\n")?;

        f.write_fmt(ume::ume! {
            #[allow(dead_code)]
            impl NonTerminalKind {
                pub fn from_non_terminal_name(name: &str) -> Self {
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
        let NodeKindTypesGenerator {
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
            .map(|NonTerminalEnumType { name, .. }| DisplayArm {
                variant: name,
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

    pub(crate) fn generate(&self, f: &mut impl Write) -> anyhow::Result<()> {
        self.generate_ast_enum_type(f)?;
        self.generate_ast_enum_impl(f)?;
        self.generate_display_impl(f)?;
        Ok(())
    }
}
