use crate::parser::parol_grammar::ScannerStateSwitch;
use crate::config::CommonGeneratorConfig;
use crate::generators::{GrammarConfig, generate_terminal_names};
use anyhow::Result;
use parol_runtime::TerminalIndex;

pub(crate) type TerminalMapping = (String, TerminalIndex, Option<(bool, String)>, String);
pub(crate) type ScannerTransition = (TerminalIndex, ScannerStateSwitch);

pub(crate) struct ScannerModeBuildData {
    pub(crate) scanner_name: String,
    pub(crate) terminal_mappings: Vec<TerminalMapping>,
    pub(crate) transitions: Vec<ScannerTransition>,
}

pub(crate) struct LexerGenerationIR<'a, C>
where
    C: CommonGeneratorConfig,
{
    pub(crate) grammar_config: &'a GrammarConfig,
    pub(crate) config: &'a C,
    pub(crate) terminal_names: Vec<String>,
    pub(crate) scanner_mode_names: Vec<String>,
}

impl<'a, C> LexerGenerationIR<'a, C>
where
    C: CommonGeneratorConfig,
{
    pub(crate) fn new(grammar_config: &'a GrammarConfig, config: &'a C) -> Self {
        let terminal_names = generate_terminal_names(grammar_config);
        let scanner_mode_names = grammar_config
            .scanner_configurations
            .iter()
            .map(|sc| sc.scanner_name.clone())
            .collect();

        Self {
            grammar_config,
            config,
            terminal_names,
            scanner_mode_names,
        }
    }

    pub(crate) fn has_scanner_modes(&self) -> bool {
        !self.scanner_mode_names.is_empty()
    }
}

pub(crate) fn build_scanner_mode_data(
    grammar_config: &GrammarConfig,
    terminal_names: &[String],
) -> Result<Vec<ScannerModeBuildData>> {
    grammar_config
        .scanner_configurations
        .iter()
        .map(|sc| {
            sc.generate_build_information(grammar_config, terminal_names)
                .map(|(terminal_mappings, transitions)| ScannerModeBuildData {
                    scanner_name: sc.scanner_name.clone(),
                    terminal_mappings,
                    transitions,
                })
        })
                .collect::<std::result::Result<Vec<_>, anyhow::Error>>()
}
