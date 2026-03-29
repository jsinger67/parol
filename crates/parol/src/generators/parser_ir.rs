//! Backend-facing parser generation context (algorithm selector + shared metadata).

use crate::LRParseTable;
use crate::analysis::LookaheadDFA;
use crate::config::{CommonGeneratorConfig, ParserGeneratorConfig};
use crate::generators::GrammarConfig;
use crate::generators::parser_model;
use anyhow::Result;
use std::collections::BTreeMap;

/// Parser algorithm input passed to language backends.
pub(crate) enum ParserAlgorithmIR<'a> {
    /// LL(k) generation with lookahead DFAs.
    Llk(&'a BTreeMap<String, LookaheadDFA>),
    /// LALR(1) generation with a parse table.
    Lalr1(&'a LRParseTable),
}

/// Shared parser metadata derived once and reused by backends.
pub(crate) struct ParserCommonIR {
    pub(crate) non_terminal_names: Vec<String>,
    pub(crate) start_symbol_index: usize,
    pub(crate) ast_type_has_lifetime: bool,
}

/// Backend-facing parser generation context.
pub(crate) struct ParserGenerationIR<'a, C>
where
    C: CommonGeneratorConfig + ParserGeneratorConfig,
{
    pub(crate) grammar_config: &'a GrammarConfig,
    pub(crate) lexer_source: &'a str,
    pub(crate) config: &'a C,
    pub(crate) algorithm: ParserAlgorithmIR<'a>,
    pub(crate) common: ParserCommonIR,
}

impl<'a, C> ParserGenerationIR<'a, C>
where
    C: CommonGeneratorConfig + ParserGeneratorConfig,
{
    pub(crate) fn new(
        grammar_config: &'a GrammarConfig,
        lexer_source: &'a str,
        config: &'a C,
        ast_type_has_lifetime: bool,
        algorithm: ParserAlgorithmIR<'a>,
    ) -> Result<Self> {
        let non_terminal_names = parser_model::ordered_non_terminal_names(grammar_config);
        let start_symbol_index =
            parser_model::find_start_symbol_index(&non_terminal_names, grammar_config)?;
        Ok(Self {
            grammar_config,
            lexer_source,
            config,
            algorithm,
            common: ParserCommonIR {
                non_terminal_names,
                start_symbol_index,
                ast_type_has_lifetime,
            },
        })
    }

    pub(crate) fn has_productions(&self) -> bool {
        !self.grammar_config.cfg.pr.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{CommonGeneratorConfig, ParserGeneratorConfig};
    use crate::utils::obtain_grammar_config;
    use std::path::PathBuf;

    #[derive(Debug)]
    struct TestConfig;

    impl CommonGeneratorConfig for TestConfig {
        fn user_type_name(&self) -> &str {
            "ParserIrTest"
        }

        fn module_name(&self) -> &str {
            "parser_ir_test"
        }

        fn minimize_boxed_types(&self) -> bool {
            false
        }

        fn range(&self) -> bool {
            false
        }

        fn node_kind_enums(&self) -> bool {
            false
        }
    }

    impl ParserGeneratorConfig for TestConfig {
        fn trim_parse_tree(&self) -> bool {
            false
        }

        fn recovery_disabled(&self) -> bool {
            false
        }
    }

    fn grammar_path(file_name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/valid")
            .join(file_name)
    }

    #[test]
    fn parser_generation_ir_initializes_common_metadata() {
        let grammar_config = obtain_grammar_config(grammar_path("clipped1.par"), false).unwrap();
        let lookahead = crate::calculate_lookahead_dfas(&grammar_config, 5).unwrap();
        let config = TestConfig;

        let ir = ParserGenerationIR::new(
            &grammar_config,
            "// lexer",
            &config,
            false,
            ParserAlgorithmIR::Llk(&lookahead),
        )
        .unwrap();

        assert_eq!(ir.common.non_terminal_names, vec!["Start".to_string()]);
        assert_eq!(ir.common.start_symbol_index, 0);
        assert!(!ir.common.ast_type_has_lifetime);
        assert!(matches!(ir.algorithm, ParserAlgorithmIR::Llk(_)));
    }

    #[test]
    fn parser_generation_ir_reports_production_presence() {
        let grammar_config = obtain_grammar_config(grammar_path("clipped1.par"), false).unwrap();
        let lookahead = crate::calculate_lookahead_dfas(&grammar_config, 5).unwrap();
        let config = TestConfig;

        let ir = ParserGenerationIR::new(
            &grammar_config,
            "// lexer",
            &config,
            true,
            ParserAlgorithmIR::Llk(&lookahead),
        )
        .unwrap();

        assert!(ir.has_productions());
        assert_eq!(ir.common.non_terminal_names, vec!["Start".to_string()]);
        assert_eq!(ir.common.start_symbol_index, 0);
        assert!(ir.common.ast_type_has_lifetime);
    }
}
