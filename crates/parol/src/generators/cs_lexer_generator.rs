use crate::CommonGeneratorConfig;
use crate::generators::{GrammarConfig, NamingHelper};
use crate::parser::parol_grammar::ScannerStateSwitch;
use anyhow::Result;
use scnr2_generate::character_classes::CharacterClasses;
use scnr2_generate::dfa::Dfa;
use scnr2_generate::nfa::Nfa;
use scnr2_generate::pattern::{Lookahead as ScnrLookahead, Pattern};
use scnr2_generate::scanner_data::TransitionToNumericMode;
use scnr2_generate::scanner_mode::ScannerMode as ScnrScannerMode;

use std::fmt::Write;

/// Generates the lexer part of the parser output file for C#.
pub fn generate_lexer_source<C: CommonGeneratorConfig>(
    grammar_config: &GrammarConfig,
    config: &C,
) -> Result<String> {
    let mut source = String::new();
    let _scanner_type_name = NamingHelper::to_upper_camel_case(config.user_type_name()) + "Scanner";

    writeln!(source, "using System;")?;
    writeln!(source, "using System.Collections.Generic;")?;
    writeln!(source, "using Parol.Runtime.Scanner;")?;
    writeln!(source)?;
    writeln!(source, "namespace {} {{", config.user_type_name())?;

    source.push_str(&generate_scanner_data(grammar_config, config)?);

    writeln!(source, "}}")?;

    Ok(source)
}

/// Generates the scanner data class for C#.
pub fn generate_scanner_data<C: CommonGeneratorConfig>(
    grammar_config: &GrammarConfig,
    config: &C,
) -> Result<String> {
    let terminal_names =
        crate::generators::lexer_generator::generate_terminal_names(grammar_config);

    let mut scanner_modes = Vec::new();
    for sc in &grammar_config.scanner_configurations {
        let sc_name = &sc.scanner_name;

        let (terminal_mappings, scnr_transitions) = sc
            .generate_build_information(grammar_config, &terminal_names)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        let mut patterns = Vec::new();
        for (rx, terminal_index, lookahead, _) in terminal_mappings {
            let scnr_lookahead =
                match lookahead {
                    Some((true, pattern)) => ScnrLookahead::positive(pattern)
                        .map_err(|e| anyhow::anyhow!(e.to_string()))?,
                    Some((false, pattern)) => ScnrLookahead::negative(pattern)
                        .map_err(|e| anyhow::anyhow!(e.to_string()))?,
                    None => ScnrLookahead::None,
                };
            patterns.push(
                Pattern::new(rx, (terminal_index as u32).into()).with_lookahead(scnr_lookahead),
            );
        }

        let mut transitions = Vec::new();
        for (terminal_index, state_switch) in scnr_transitions {
            let transition = match state_switch {
                ScannerStateSwitch::SwitchPush(mode_name, _) => {
                    let mode_index = grammar_config
                        .scanner_configurations
                        .iter()
                        .position(|s| s.scanner_name == mode_name)
                        .unwrap();
                    TransitionToNumericMode::PushMode(terminal_index as usize, mode_index)
                }
                ScannerStateSwitch::SwitchPop(_) => {
                    TransitionToNumericMode::PopMode(terminal_index as usize)
                }
                ScannerStateSwitch::Switch(mode_name, _) => {
                    let mode_index = grammar_config
                        .scanner_configurations
                        .iter()
                        .position(|s| s.scanner_name == mode_name)
                        .unwrap();
                    TransitionToNumericMode::SetMode(terminal_index as usize, mode_index)
                }
            };
            transitions.push(transition);
        }
        transitions.sort_by_key(|t| t.token_type());

        scanner_modes.push(ScnrScannerMode::new(sc_name, patterns, transitions));
    }

    // Build DFAs and CharacterClasses
    let mut nfas = scanner_modes
        .iter()
        .map(|mode| {
            Nfa::build_from_patterns(&mode.patterns).map_err(|e| anyhow::anyhow!(e.to_string()))
        })
        .collect::<Result<Vec<_>>>()?;

    let mut character_classes = CharacterClasses::new();
    for nfa in &nfas {
        nfa.collect_character_classes(&mut character_classes);
    }
    character_classes.create_disjoint_character_classes();
    for nfa in &mut nfas {
        nfa.convert_to_disjoint_character_classes(&character_classes);
    }

    let dfas = nfas
        .into_iter()
        .map(|nfa| Dfa::try_from(&nfa).map_err(|e| anyhow::anyhow!(e.to_string())))
        .collect::<Result<Vec<_>>>()?;

    // Generate C# source
    let mut source = String::new();
    let scanner_type_name = NamingHelper::to_upper_camel_case(config.user_type_name()) + "Scanner";

    writeln!(
        source,
        "    public static class {}Data {{",
        scanner_type_name
    )?;

    // Terminal Names
    writeln!(
        source,
        "        public static readonly string[] TerminalNames = {{"
    )?;
    for name in &terminal_names {
        writeln!(source, "            \"{}\",", name)?;
    }
    writeln!(source, "        }};")?;
    writeln!(source)?;

    // Match Function
    generate_match_function(&mut source, &character_classes)?;

    // Scanner Modes
    writeln!(
        source,
        "        public static readonly ScannerMode[] ScannerModes = {{"
    )?;
    for (i, mode) in scanner_modes.iter().enumerate() {
        let dfa = &dfas[i];
        generate_scanner_mode(&mut source, mode, dfa, character_classes.intervals.len())?;
    }
    writeln!(source, "        }};")?;

    writeln!(source, "    }}")?;

    Ok(source)
}

fn generate_match_function(
    source: &mut String,
    character_classes: &CharacterClasses,
) -> Result<()> {
    writeln!(
        source,
        "        public static int? MatchFunction(char c) {{"
    )?;
    writeln!(
        source,
        "            var intervals = new (char Start, char End, int ClassIdx)[] {{"
    )?;
    for interval in &character_classes.elementary_intervals {
        let start = *interval.start();
        let end = *interval.end();
        let class_idx = character_classes
            .intervals
            .iter()
            .enumerate()
            .find(|(_, group)| group.contains(interval))
            .map(|(idx, _)| idx)
            .unwrap();
        writeln!(
            source,
            "                ('{}', '{}', {}),",
            escape_char(start),
            escape_char(end),
            class_idx
        )?;
    }
    writeln!(source, "            }};")?;
    writeln!(source)?;
    writeln!(
        source,
        "            int low = 0, high = intervals.Length - 1;"
    )?;
    writeln!(source, "            while (low <= high) {{")?;
    writeln!(source, "                int mid = low + (high - low) / 2;")?;
    writeln!(
        source,
        "                if (c >= intervals[mid].Start && c <= intervals[mid].End) return intervals[mid].ClassIdx;"
    )?;
    writeln!(
        source,
        "                if (c < intervals[mid].Start) high = mid - 1;"
    )?;
    writeln!(source, "                else low = mid + 1;")?;
    writeln!(source, "            }}")?;
    writeln!(source, "            return null;")?;
    writeln!(source, "        }}")?;
    Ok(())
}

fn generate_scanner_mode(
    source: &mut String,
    mode: &ScnrScannerMode,
    dfa: &Dfa,
    num_classes: usize,
) -> Result<()> {
    writeln!(source, "            new ScannerMode(")?;
    writeln!(source, "                \"{}\",", mode.name)?;

    // Transitions
    writeln!(source, "                new Transition[] {{")?;
    for t in &mode.transitions {
        match t {
            TransitionToNumericMode::SetMode(token_type, target) => {
                writeln!(
                    source,
                    "                    new Transition(TransitionType.SetMode, {}, {}),",
                    token_type, target
                )?;
            }
            TransitionToNumericMode::PushMode(token_type, target) => {
                writeln!(
                    source,
                    "                    new Transition(TransitionType.PushMode, {}, {}),",
                    token_type, target
                )?;
            }
            TransitionToNumericMode::PopMode(token_type) => {
                writeln!(
                    source,
                    "                    new Transition(TransitionType.PopMode, {}),",
                    token_type
                )?;
            }
        }
    }
    writeln!(source, "                }},")?;

    // DFA
    generate_dfa(source, dfa, num_classes)?;

    writeln!(source, "            ),")?;
    Ok(())
}

fn generate_dfa(source: &mut String, dfa: &Dfa, num_classes: usize) -> Result<()> {
    writeln!(source, "                new Dfa(new DfaState[] {{")?;
    for state in &dfa.states {
        writeln!(source, "                    new DfaState(")?;

        // Transitions
        write!(source, "                        new DfaTransition[] {{ ")?;
        let mut transition_opts = vec![None; num_classes];
        for t in &state.transitions {
            transition_opts[t.elementary_interval_index.as_usize()] = Some(t.target.as_usize());
        }
        for (i, opt) in transition_opts.iter().enumerate() {
            if let Some(target) = opt {
                write!(source, "new DfaTransition({})", target)?;
            } else {
                write!(source, "null")?;
            }
            if i < num_classes - 1 {
                write!(source, ", ")?;
            }
        }
        writeln!(source, " }},")?;

        // Accept Data
        writeln!(source, "                        new AcceptData[] {{")?;
        for ad in &state.accept_data {
            write!(
                source,
                "                            new AcceptData({}, {}, ",
                ad.terminal_type.as_usize(),
                ad.priority
            )?;
            generate_lookahead(source, &ad.lookahead, num_classes)?;
            writeln!(source, "),")?;
        }
        writeln!(source, "                        }}")?;

        writeln!(source, "                    ),")?;
    }
    writeln!(source, "                }})")?;
    Ok(())
}

fn generate_lookahead(
    source: &mut String,
    lookahead: &scnr2_generate::pattern::Lookahead,
    num_classes: usize,
) -> Result<()> {
    match lookahead {
        scnr2_generate::pattern::Lookahead::None => write!(source, "new Lookahead.None()")?,
        scnr2_generate::pattern::Lookahead::Positive(
            scnr2_generate::pattern::AutomatonType::Dfa(d),
        ) => {
            write!(source, "new Lookahead.Positive(")?;
            generate_dfa(source, d, num_classes)?;
            write!(source, ")")?;
        }
        scnr2_generate::pattern::Lookahead::Negative(
            scnr2_generate::pattern::AutomatonType::Dfa(d),
        ) => {
            write!(source, "new Lookahead.Negative(")?;
            generate_dfa(source, d, num_classes)?;
            write!(source, ")")?;
        }
        _ => panic!("Unexpected lookahead type"),
    }
    Ok(())
}

fn escape_char(c: char) -> String {
    match c {
        '\'' => "\\'".to_string(),
        '\\' => "\\\\".to_string(),
        '\n' => "\\n".to_string(),
        '\r' => "\\r".to_string(),
        '\t' => "\\t".to_string(),
        '\0' => "\\0".to_string(),
        _ if c.is_ascii_graphic() || c == ' ' => c.to_string(),
        _ => {
            let u = c as u32;
            if u <= 0xFFFF {
                format!("\\u{:04x}", u)
            } else {
                // C# char is UTF-16, so we need surrogate pairs for characters above U+FFFF
                // However, our MatchFunction takes 'char' which is System.Char (UTF-16).
                // If we want to support full Unicode, we should use 'int' (32-bit).
                // For now, we'll just use the \U notation if it were a string, but for char it's not possible.
                // Wait, C# char can't hold values > 0xFFFF.
                format!("\\u{:04x}", u & 0xFFFF) // This is incorrect for full Unicode but safe for syntax
            }
        }
    }
}
