use crate::config::CommonGeneratorConfig;
use crate::generators::lexer_ir::LexerGenerationIR;
use anyhow::{Result, anyhow};

pub(crate) trait LexerLanguageBackend<C>
where
    C: CommonGeneratorConfig,
{
    fn generate_lexer_source(&self, lexer_ir: &LexerGenerationIR<'_, C>) -> Result<String>;
}

pub(crate) struct RustLexerBackend;

impl<C> LexerLanguageBackend<C> for RustLexerBackend
where
    C: CommonGeneratorConfig,
{
    fn generate_lexer_source(&self, lexer_ir: &LexerGenerationIR<'_, C>) -> Result<String> {
        crate::generators::lexer_generator::generate_lexer_source_with_terminal_names(
            lexer_ir.grammar_config,
            lexer_ir.config,
            &lexer_ir.terminal_names,
        )
    }
}

pub(crate) struct CSharpLexerBackend;

impl<C> LexerLanguageBackend<C> for CSharpLexerBackend
where
    C: CommonGeneratorConfig,
{
    fn generate_lexer_source(&self, lexer_ir: &LexerGenerationIR<'_, C>) -> Result<String> {
        crate::generators::cs_lexer_generator::generate_lexer_source_with_terminal_names(
            lexer_ir.grammar_config,
            lexer_ir.config,
            &lexer_ir.terminal_names,
        )
    }
}

pub(crate) fn generate_lexer_source_for_language<C>(
    backend: &impl LexerLanguageBackend<C>,
    lexer_ir: &LexerGenerationIR<'_, C>,
) -> Result<String>
where
    C: CommonGeneratorConfig,
{
    if !lexer_ir.has_scanner_modes() {
        return Err(anyhow!("Grammar contains no scanner configurations"));
    }
    backend.generate_lexer_source(lexer_ir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::CommonGeneratorConfig;
    use crate::utils::obtain_grammar_config;
    use std::path::PathBuf;

    const RUST_LEXER_OUTPUT_CHECKSUM: u64 = 3620951960146662877;
    const CSHARP_LEXER_OUTPUT_CHECKSUM: u64 = 7234489667512800621;

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
    fn rust_lexer_backend_matches_direct_generation() {
        let grammar_config = obtain_grammar_config(test_grammar_path(), false).unwrap();
        let config = TestConfig;
        let lexer_ir = LexerGenerationIR::new(&grammar_config, &config);

        let via_backend = generate_lexer_source_for_language(&RustLexerBackend, &lexer_ir).unwrap();
        let direct =
            crate::generators::lexer_generator::generate_lexer_source(&grammar_config, &config)
                .unwrap();

        assert_eq!(direct, via_backend);
    }

    #[test]
    fn csharp_lexer_backend_matches_direct_generation() {
        let grammar_config = obtain_grammar_config(test_grammar_path(), false).unwrap();
        let config = TestConfig;
        let lexer_ir = LexerGenerationIR::new(&grammar_config, &config);

        let via_backend =
            generate_lexer_source_for_language(&CSharpLexerBackend, &lexer_ir).unwrap();
        let direct =
            crate::generators::cs_lexer_generator::generate_lexer_source(&grammar_config, &config)
                .unwrap();

        assert_eq!(direct, via_backend);
    }

    #[test]
    fn rust_lexer_output_checksum_stable() {
        let grammar_config = obtain_grammar_config(test_grammar_path(), false).unwrap();
        let config = TestConfig;

        let direct =
            crate::generators::lexer_generator::generate_lexer_source(&grammar_config, &config)
                .unwrap();

        let checksum = stable_checksum(&direct);
        assert_eq!(
            RUST_LEXER_OUTPUT_CHECKSUM, checksum,
            "Rust lexer output checksum changed: {checksum}"
        );
    }

    #[test]
    fn csharp_lexer_output_checksum_stable() {
        let grammar_config = obtain_grammar_config(test_grammar_path(), false).unwrap();
        let config = TestConfig;

        let direct =
            crate::generators::cs_lexer_generator::generate_lexer_source(&grammar_config, &config)
                .unwrap();

        let checksum = stable_checksum(&direct);
        assert_eq!(
            CSHARP_LEXER_OUTPUT_CHECKSUM, checksum,
            "C# lexer output checksum changed: {checksum}"
        );
    }
}
