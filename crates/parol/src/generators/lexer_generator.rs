use crate::generate_name;
use crate::generators::{generate_terminal_name, GrammarConfig};
use anyhow::Result;
use parol_runtime::TerminalIndex;

use crate::StrVec;
use std::fmt::{Debug, Write};

use super::regex_serializer::RegexSerializer;

#[derive(Debug, Default)]
struct ScannerBuildInfo {
    scanner_index: usize,
    scanner_name: String,
    terminal_index_count: usize,
    special_tokens: StrVec,
    terminal_indices: StrVec,
    fwd_dfa: String,
    fwd_bytes: usize,
    rev_dfa: String,
    rev_bytes: usize,
}

impl ScannerBuildInfo {
    fn try_from_scanner_build_info(
        augmented_terminals: &[String],
        scanner_index: usize,
        scanner_name: String,
        terminal_names: &[String],
        width: usize,
        special_tokens: &[String],
        terminal_indices: &[TerminalIndex],
    ) -> Result<Self> {
        let special_tokens =
            special_tokens
                .iter()
                .enumerate()
                .fold(StrVec::new(0), |mut acc, (i, e)| {
                    let e = match e.as_str() {
                        "UNMATCHABLE_TOKEN" | "NEW_LINE_TOKEN" | "WHITESPACE_TOKEN"
                        | "ERROR_TOKEN" => e.to_owned(),
                        _ => {
                            let hashes = determine_hashes_for_raw_string(e);
                            format!(r#"r{}"{}"{}"#, hashes, e, hashes)
                        }
                    };
                    acc.push(format!("/* {:w$} */ {},", i, e, w = width));
                    acc
                });
        let terminal_names = terminal_indices.iter().fold(StrVec::new(8), |mut acc, e| {
            acc.push(format!(r#"{}, /* {} */"#, e, terminal_names[*e as usize]));
            acc
        });

        let mut token_types = Vec::with_capacity(terminal_names.len());
        let (fwd, rev) = RegexSerializer::serialize(
            augmented_terminals,
            special_tokens.as_slice(),
            terminal_indices,
            &mut token_types,
        )?;
        let fwd_bytes = fwd.len();
        let rev_bytes = rev.len();

        let mut fwd_dfa = String::new();
        let mut rev_dfa = String::new();
        write!(&mut fwd_dfa, "{}", fwd)?;
        write!(&mut rev_dfa, "{}", rev)?;

        Ok(Self {
            scanner_index,
            scanner_name,
            terminal_index_count: terminal_names.len(),
            special_tokens,
            terminal_indices: terminal_names,
            fwd_dfa,
            fwd_bytes,
            rev_dfa,
            rev_bytes,
        })
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
    augmented_terminals: StrVec,
    used_token_constants: String,
    terminal_names: StrVec,
    terminal_count: usize,
    scanner_build_configs: StrVec,
}

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
///
/// Generates the lexer part of the parser output file.
///
pub fn generate_lexer_source(grammar_config: &GrammarConfig) -> Result<String> {
    let original_augmented_terminals = grammar_config.generate_augmented_terminals();

    let terminal_count = original_augmented_terminals.len();
    let width = (terminal_count as f32).log10() as usize + 1;

    let augmented_terminals =
        original_augmented_terminals
            .iter()
            .enumerate()
            .fold(StrVec::new(4), |mut acc, (i, e)| {
                let e = match e.as_str() {
                    "UNMATCHABLE_TOKEN" | "NEW_LINE_TOKEN" | "WHITESPACE_TOKEN" | "ERROR_TOKEN" => {
                        e.to_owned()
                    }
                    _ => {
                        let hashes = determine_hashes_for_raw_string(e);
                        format!(r#"r{}"{}"{}"#, hashes, e, hashes)
                    }
                };
                acc.push(format!("/* {:w$} */ {},", i, e, w = width));
                acc
            });

    let token_constants: Vec<(&str, bool)> = vec![
        ("ERROR_TOKEN,", true),
        (
            "NEW_LINE_TOKEN,",
            grammar_config
                .scanner_configurations
                .iter()
                .any(|sc| sc.auto_newline),
        ),
        ("UNMATCHABLE_TOKEN,", true),
        (
            "WHITESPACE_TOKEN,",
            grammar_config
                .scanner_configurations
                .iter()
                .any(|sc| sc.auto_ws),
        ),
    ];

    let used_token_constants = token_constants
        .iter()
        .fold(String::new(), |mut acc, (c, u)| {
            if *u {
                acc.push_str(c);
            }
            acc
        });

    let terminal_names =
        original_augmented_terminals
            .iter()
            .enumerate()
            .fold(Vec::new(), |mut acc, (i, e)| {
                let n = generate_name(
                    &acc,
                    generate_terminal_name(e, Some(i as TerminalIndex), &grammar_config.cfg),
                );
                acc.push(n);
                acc
            });

    let scanner_build_configs = grammar_config
        .scanner_configurations
        .iter()
        .enumerate()
        .map(|(i, sc)| (i, sc.generate_build_information(&grammar_config.cfg)))
        .map(|(i, (sp, ti, n))| {
            ScannerBuildInfo::try_from_scanner_build_info(
                augmented_terminals.as_slice(),
                i,
                n,
                &terminal_names,
                width,
                &sp,
                &ti,
            )
        })
        .try_fold(StrVec::new(0), |mut acc, s| {
            acc.push(format!("{}", s?));
            Ok::<StrVec, anyhow::Error>(acc)
        })?;

    let terminal_names =
        terminal_names
            .iter()
            .enumerate()
            .fold(StrVec::new(4), |mut acc, (i, e)| {
                acc.push(format!(r#"/* {:w$} */ "{}","#, i, e, w = width));
                acc
            });

    let lexer_data = LexerData {
        augmented_terminals,
        used_token_constants,
        terminal_names,
        terminal_count,
        scanner_build_configs,
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
                &acc,
                generate_terminal_name(e, Some(i as TerminalIndex), &grammar_config.cfg),
            );
            acc.push(n);
            acc
        })
}

impl std::fmt::Display for LexerData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let LexerData {
            augmented_terminals,
            used_token_constants,
            terminal_names,
            terminal_count,
            scanner_build_configs,
        } = self;

        let blank_line = "\n\n";
        let scanner_build_configs = scanner_build_configs.join("\n\n");
        f.write_fmt(ume::ume! {
        use parol_runtime::lexer::tokenizer::{
            #used_token_constants
        };
        #blank_line
        pub const TERMINALS: &[&str; #terminal_count] = &[
        #augmented_terminals];
        #blank_line
        pub const TERMINAL_NAMES: &[&str; #terminal_count] = &[
        #terminal_names];
        #blank_line
        #scanner_build_configs
        })
    }
}

impl std::fmt::Display for ScannerBuildInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ScannerBuildInfo {
            scanner_index,
            scanner_name,
            terminal_index_count,
            special_tokens,
            terminal_indices,
            fwd_dfa,
            fwd_bytes,
            rev_dfa,
            rev_bytes,
        } = self;

        writeln!(f, r#"/* SCANNER_{scanner_index}: "{scanner_name}" */"#)?;
        let scanner_name = format!("SCANNER_{}", scanner_index);
        f.write_fmt(ume::ume! {
            const #scanner_name: (&[&str; 5], &[TerminalIndex; #terminal_index_count], &[u8; #fwd_bytes], &[u8; #rev_bytes]) = (
                &[#special_tokens],
                &[#terminal_indices],
                // Forward DFA
                &#fwd_dfa,
                // Reverse DFA
                &#rev_dfa,
            );
        })
    }
}
