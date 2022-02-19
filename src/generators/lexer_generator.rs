use crate::generate_name;
use crate::generators::{generate_terminal_name, GrammarConfig};
use miette::Result;

use crate::StrVec;
use std::fmt::Debug;

#[derive(BartDisplay, Debug, Default)]
#[template = "templates/scanner_build_info.rs"]
struct ScannerBuildInfo {
    scanner_index: usize,
    scanner_name: String,
    terminal_index_count: usize,
    special_tokens: StrVec,
    terminal_indices: StrVec,
}

impl ScannerBuildInfo {
    fn from_scanner_build_info(
        scanner_index: usize,
        scanner_name: String,
        terminal_names: &[String],
        width: usize,
        special_tokens: &[String],
        terminal_indices: &[usize],
    ) -> Self {
        let special_tokens =
            special_tokens
                .iter()
                .enumerate()
                .fold(StrVec::new(0), |mut acc, (i, e)| {
                    let e = match e.as_str() {
                        "UNMATCHABLE_TOKEN" | "NEW_LINE_TOKEN" | "WHITESPACE_TOKEN"
                        | "ERROR_TOKEN" => e.to_owned(),
                        _ => format!(r####"r###"{}"###"####, e),
                    };
                    acc.push(format!("/* {:w$} */ {},", i, e, w = width));
                    acc
                });
        let terminal_indices = terminal_indices.iter().fold(StrVec::new(8), |mut acc, e| {
            acc.push(format!(r#"{}, /* {} */"#, e, terminal_names[*e]));
            acc
        });
        Self {
            scanner_index,
            scanner_name,
            terminal_index_count: terminal_indices.len(),
            special_tokens,
            terminal_indices,
        }
    }
}

#[derive(BartDisplay, Debug, Default)]
#[template = "templates/lexer_template.rs"]
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
                    _ => format!(r####"r###"{}"###"####, e),
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
                    generate_terminal_name(e, Some(i), &grammar_config.cfg),
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
            ScannerBuildInfo::from_scanner_build_info(i, n, &terminal_names, width, &sp, &ti)
        })
        .fold(StrVec::new(0), |mut acc, e| {
            acc.push(format!("{}", e));
            acc
        });

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
                generate_terminal_name(e, Some(i), &grammar_config.cfg),
            );
            acc.push(n);
            acc
        })
}
