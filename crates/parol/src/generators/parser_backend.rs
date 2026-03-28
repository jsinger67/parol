use crate::config::{CommonGeneratorConfig, ParserGeneratorConfig};
use crate::generators::parser_ir::{ParserAlgorithmIR, ParserGenerationIR};
use anyhow::{Result, anyhow};

pub(crate) trait ParserLanguageBackend<C>
where
    C: CommonGeneratorConfig + ParserGeneratorConfig,
{
    fn generate_parser_source(&self, parser_ir: &ParserGenerationIR<'_, C>) -> Result<String>;
}

pub(crate) struct RustParserBackend;

impl<C> ParserLanguageBackend<C> for RustParserBackend
where
    C: CommonGeneratorConfig + ParserGeneratorConfig,
{
    fn generate_parser_source(&self, parser_ir: &ParserGenerationIR<'_, C>) -> Result<String> {
        let ast_type_has_lifetime = parser_ir.common.ast_type_has_lifetime;
        match parser_ir.algorithm {
            ParserAlgorithmIR::Llk(lookahead_dfas) => {
                crate::generators::parser_generator::generate_parser_source(
                    parser_ir.grammar_config,
                    parser_ir.lexer_source,
                    parser_ir.config,
                    lookahead_dfas,
                    ast_type_has_lifetime,
                )
            }
            ParserAlgorithmIR::Lalr1(parse_table) => {
                crate::generators::parser_generator::generate_lalr1_parser_source(
                    parser_ir.grammar_config,
                    parser_ir.lexer_source,
                    parser_ir.config,
                    parse_table,
                    ast_type_has_lifetime,
                )
            }
        }
    }
}

pub(crate) struct CSharpParserBackend;

impl<C> ParserLanguageBackend<C> for CSharpParserBackend
where
    C: CommonGeneratorConfig + ParserGeneratorConfig,
{
    fn generate_parser_source(&self, parser_ir: &ParserGenerationIR<'_, C>) -> Result<String> {
        let ast_type_has_lifetime = parser_ir.common.ast_type_has_lifetime;
        match parser_ir.algorithm {
            ParserAlgorithmIR::Llk(lookahead_dfas) => {
                crate::generators::cs_parser_generator::generate_parser_source(
                    parser_ir.grammar_config,
                    parser_ir.lexer_source,
                    parser_ir.config,
                    lookahead_dfas,
                    ast_type_has_lifetime,
                )
            }
            ParserAlgorithmIR::Lalr1(parse_table) => {
                crate::generators::cs_parser_generator::generate_lalr1_parser_source(
                    parser_ir.grammar_config,
                    parser_ir.lexer_source,
                    parser_ir.config,
                    parse_table,
                    ast_type_has_lifetime,
                )
            }
        }
    }
}

pub(crate) fn generate_parser_source_for_language<C>(
    backend: &impl ParserLanguageBackend<C>,
    parser_ir: &ParserGenerationIR<'_, C>,
) -> Result<String>
where
    C: CommonGeneratorConfig + ParserGeneratorConfig,
{
    if !parser_ir.has_productions() {
        return Err(anyhow!("Grammar contains no productions"));
    }
    if parser_ir.common.non_terminal_names.is_empty() {
        return Err(anyhow!("Grammar contains no non-terminals"));
    }
    if parser_ir.common.start_symbol_index >= parser_ir.common.non_terminal_names.len() {
        return Err(anyhow!("Start symbol index is out of bounds"));
    }
    backend.generate_parser_source(parser_ir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{CommonGeneratorConfig, ParserGeneratorConfig, UserTraitGeneratorConfig};
    use crate::generators::lexer_generator;
    use crate::generators::{GrammarTypeInfo, UserTraitGenerator};
    use crate::utils::obtain_grammar_config;
    use crate::{InnerAttributes, calculate_lookahead_dfas};
    use std::path::PathBuf;

    const RUST_PARSER_OUTPUT_CHECKSUM: u64 = 5880726045965946773;
    const CSHARP_PARSER_OUTPUT_CHECKSUM: u64 = 10987753882266822372;

    #[derive(Debug)]
    struct TestConfig;

    impl CommonGeneratorConfig for TestConfig {
        fn user_type_name(&self) -> &str {
            "BackendTest"
        }

        fn module_name(&self) -> &str {
            "backend_test"
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

    impl UserTraitGeneratorConfig for TestConfig {
        fn inner_attributes(&self) -> &[InnerAttributes] {
            &[]
        }
    }

    fn test_grammar_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/arg_tests/generate.par")
    }

    fn stable_checksum(input: &str) -> u64 {
        input
            .as_bytes()
            .iter()
            .fold(14695981039346656037u64, |hash, b| {
                hash.wrapping_mul(1099511628211).wrapping_add(u64::from(*b))
            })
    }

    #[test]
    fn rust_parser_backend_matches_direct_generation() {
        let grammar_config = obtain_grammar_config(test_grammar_path(), false).unwrap();
        let config = TestConfig;
        let lexer_source =
            lexer_generator::generate_lexer_source(&grammar_config, &config).unwrap();
        let lookahead_dfas = calculate_lookahead_dfas(&grammar_config, 5).unwrap();

        let mut type_info = GrammarTypeInfo::try_new(config.user_type_name()).unwrap();
        UserTraitGenerator::new(&grammar_config)
            .generate_user_trait_source(&config, grammar_config.grammar_type, &mut type_info)
            .unwrap();

        let parser_ir = ParserGenerationIR::new(
            &grammar_config,
            &lexer_source,
            &config,
            type_info.symbol_table.has_lifetime(type_info.ast_enum_type),
            ParserAlgorithmIR::Llk(&lookahead_dfas),
        )
        .unwrap();

        let via_backend =
            generate_parser_source_for_language(&RustParserBackend, &parser_ir).unwrap();
        let direct = crate::generators::parser_generator::generate_parser_source(
            &grammar_config,
            &lexer_source,
            &config,
            &lookahead_dfas,
            parser_ir.common.ast_type_has_lifetime,
        )
        .unwrap();

        assert_eq!(direct, via_backend);
    }

    #[test]
    fn csharp_parser_backend_matches_direct_generation() {
        let grammar_config = obtain_grammar_config(test_grammar_path(), false).unwrap();
        let config = TestConfig;
        let lexer_source =
            lexer_generator::generate_lexer_source(&grammar_config, &config).unwrap();
        let lookahead_dfas = calculate_lookahead_dfas(&grammar_config, 5).unwrap();

        let mut type_info = GrammarTypeInfo::try_new(config.user_type_name()).unwrap();
        UserTraitGenerator::new(&grammar_config)
            .generate_user_trait_source(&config, grammar_config.grammar_type, &mut type_info)
            .unwrap();

        let parser_ir = ParserGenerationIR::new(
            &grammar_config,
            &lexer_source,
            &config,
            type_info.symbol_table.has_lifetime(type_info.ast_enum_type),
            ParserAlgorithmIR::Llk(&lookahead_dfas),
        )
        .unwrap();

        let via_backend =
            generate_parser_source_for_language(&CSharpParserBackend, &parser_ir).unwrap();
        let direct = crate::generators::cs_parser_generator::generate_parser_source(
            &grammar_config,
            &lexer_source,
            &config,
            &lookahead_dfas,
            parser_ir.common.ast_type_has_lifetime,
        )
        .unwrap();

        assert_eq!(direct, via_backend);
    }

    #[test]
    fn rust_parser_output_checksum_stable() {
        let grammar_config = obtain_grammar_config(test_grammar_path(), false).unwrap();
        let config = TestConfig;
        let lexer_source =
            lexer_generator::generate_lexer_source(&grammar_config, &config).unwrap();
        let lookahead_dfas = calculate_lookahead_dfas(&grammar_config, 5).unwrap();

        let mut type_info = GrammarTypeInfo::try_new(config.user_type_name()).unwrap();
        UserTraitGenerator::new(&grammar_config)
            .generate_user_trait_source(&config, grammar_config.grammar_type, &mut type_info)
            .unwrap();

        let direct = crate::generators::parser_generator::generate_parser_source(
            &grammar_config,
            &lexer_source,
            &config,
            &lookahead_dfas,
            type_info.symbol_table.has_lifetime(type_info.ast_enum_type),
        )
        .unwrap();

        let checksum = stable_checksum(&direct);
        assert_eq!(
            RUST_PARSER_OUTPUT_CHECKSUM, checksum,
            "Rust parser output checksum changed: {checksum}"
        );
    }

    #[test]
    fn csharp_parser_output_checksum_stable() {
        let grammar_config = obtain_grammar_config(test_grammar_path(), false).unwrap();
        let config = TestConfig;
        let lexer_source =
            lexer_generator::generate_lexer_source(&grammar_config, &config).unwrap();
        let lookahead_dfas = calculate_lookahead_dfas(&grammar_config, 5).unwrap();

        let mut type_info = GrammarTypeInfo::try_new(config.user_type_name()).unwrap();
        UserTraitGenerator::new(&grammar_config)
            .generate_user_trait_source(&config, grammar_config.grammar_type, &mut type_info)
            .unwrap();

        let direct = crate::generators::cs_parser_generator::generate_parser_source(
            &grammar_config,
            &lexer_source,
            &config,
            &lookahead_dfas,
            type_info.symbol_table.has_lifetime(type_info.ast_enum_type),
        )
        .unwrap();

        let checksum = stable_checksum(&direct);
        assert_eq!(
            CSHARP_PARSER_OUTPUT_CHECKSUM, checksum,
            "C# parser output checksum changed: {checksum}"
        );
    }
}
