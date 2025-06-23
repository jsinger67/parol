use crate::generators::{GrammarConfig, NamingHelper, generate_terminal_name};
use crate::{CommonGeneratorConfig, generate_name};
use anyhow::Result;
use parol_runtime::TerminalIndex;

use crate::StrVec;
use std::fmt::Debug;

// Regular expression + terminal index + optional lookahead expression + generated token name
type TerminalMapping = (String, TerminalIndex, Option<(bool, String)>, String);
// Scanner transition is a tuple of terminal index and the name of the next scanner mode
type ScannerTransition = (TerminalIndex, String);

#[derive(Debug, Default)]
struct ScannerBuildInfo {
    scanner_name: String,
    terminal_mappings: Vec<TerminalMapping>,
    transitions: Vec<ScannerTransition>,
}

impl ScannerBuildInfo {
    fn from_scanner_build_info(
        terminal_mappings: Vec<TerminalMapping>,
        transitions: Vec<ScannerTransition>,
        scanner_name: String,
    ) -> Self {
        Self {
            scanner_name,
            terminal_mappings,
            transitions,
        }
    }
}

fn determine_hashes_for_raw_string(e: &str) -> String {
    let mut pattern = r#"""#.to_string();
    let mut count = 0;
    while e.contains(&pattern) {
        pattern.push('#');
        count += 1;
    }
    "#".repeat(count)
}

#[derive(Debug, Default)]
struct LexerData {
    terminal_names: StrVec,
    terminal_count: usize,
    scanner_macro: StrVec,
}

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
///
/// Generates the lexer part of the parser output file.
///
pub fn generate_lexer_source<C: CommonGeneratorConfig>(
    grammar_config: &GrammarConfig,
    config: &C,
) -> Result<String> {
    let original_augmented_terminals = grammar_config.generate_augmented_terminals();

    let terminal_count = original_augmented_terminals.len();
    let width = (terminal_count as f32).log10() as usize + 1;

    let terminal_names =
        original_augmented_terminals
            .iter()
            .enumerate()
            .fold(Vec::new(), |mut acc, (i, e)| {
                let n = generate_name(
                    acc.iter(),
                    generate_terminal_name(&e.0, Some(i as TerminalIndex), &grammar_config.cfg),
                );
                acc.push(n);
                acc
            });

    let macro_start =
        StrVec::from_iter(vec![format!("\n    {} {{", get_scanner_type_name(config))]);
    let mut scanner_macro = grammar_config
        .scanner_configurations
        .iter()
        .map(|sc| {
            sc.generate_build_information(grammar_config, &terminal_names)
                .map(|(r, t)| (r, t, sc.scanner_name.clone()))
        })
        .collect::<Result<
            Vec<(
                Vec<TerminalMapping>,
                Vec<ScannerTransition>,
                String,
            )>,
            anyhow::Error,
        >>()?
        .into_iter()
        .map(|(terminal_mappings, transitions, scanner_name)| {
            ScannerBuildInfo::from_scanner_build_info(terminal_mappings, transitions, scanner_name)
        })
        .fold(macro_start, |mut acc, e| {
            acc.push(format!("{}", e));
            acc
        });
    scanner_macro.push("    }".to_string());

    let terminal_names =
        terminal_names
            .iter()
            .enumerate()
            .fold(StrVec::new(4), |mut acc, (i, e)| {
                acc.push(format!(r#"/* {:w$} */ "{}","#, i, e, w = width));
                acc
            });

    let lexer_data = LexerData {
        terminal_names,
        terminal_count,
        scanner_macro,
    };

    Ok(format!("{}", lexer_data))
}

/// Generates all terminal names of a given grammar
pub fn generate_terminal_names(grammar_config: &GrammarConfig) -> Vec<String> {
    grammar_config
        .generate_augmented_terminals()
        .iter()
        .enumerate()
        .fold(Vec::new(), |mut acc, (i, e)| {
            let n = generate_name(
                acc.iter(),
                generate_terminal_name(&e.0, Some(i as TerminalIndex), &grammar_config.cfg),
            );
            acc.push(n);
            acc
        })
}

fn get_scanner_type_name<C: CommonGeneratorConfig>(config: &C) -> String {
    let scanner_type_name = NamingHelper::to_upper_camel_case(config.user_type_name());
    scanner_type_name + "Scanner"
}

impl std::fmt::Display for LexerData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let LexerData {
            terminal_names,
            terminal_count,
            scanner_macro,
        } = self;

        let blank_line = "\n\n";
        f.write_fmt(ume::ume! {
        #blank_line
        // pub const TERMINALS: &[(&str, Option<(bool, &str)>); #terminal_count] = &[
        // #augmented_terminals];
        #blank_line
        pub const TERMINAL_NAMES: &[&str; #terminal_count] = &[
        #terminal_names];
        #blank_line
        })?;
        f.write_fmt(format_args!("scanner! {{{}}}", scanner_macro))
    }
}

impl std::fmt::Display for ScannerBuildInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ScannerBuildInfo {
            scanner_name,
            terminal_mappings,
            transitions,
        } = self;

        let tokens = terminal_mappings
            .iter()
            .fold(StrVec::new(12), |mut acc, (rx, i, l, tn)| {
                // Generate the token definition
                //   No lookahead expression
                //     token r"World" => 10;
                //   With positive lookahead expression
                //     token r"World" followed by r"!" => 11;
                //   With negative lookahead expression
                //     token r"!" not followed by r"!" => 12;

                let hashes = determine_hashes_for_raw_string(rx);
                let terminal_name_comment = if tn.is_empty() {
                    String::new()
                } else {
                    format!(r#" // "{}""#, tn)
                };
                let lookahead = if let Some((is_positive, pattern)) = l {
                    let hashes = determine_hashes_for_raw_string(pattern);
                    if *is_positive {
                        format!(" followed by r{}\"{}\"{}", hashes, pattern, hashes)
                    } else {
                        format!(" not followed by r{}\"{}\"{}", hashes, pattern, hashes)
                    }
                } else {
                    String::new()
                };

                let token = format!(
                    r#"token r{}"{}"{} {}=> {};{}"#,
                    hashes, rx, hashes, lookahead, i, terminal_name_comment
                );

                acc.push(token);
                acc
            });

        let transitions = transitions.iter().fold(StrVec::new(12), |mut acc, (i, e)| {
            // Generate the transition definition
            //   on 10 enter World;
            acc.push(format!(r#"on {} enter {};"#, i, e));
            acc
        });

        // Generate the scanner's part of the macro code
        f.write_fmt(format_args!("        mode {} {{\n", scanner_name))?;
        f.write_fmt(format_args!("{}", tokens))?;
        f.write_fmt(format_args!("{}", transitions))?;
        f.write_str("        }")
    }
}
