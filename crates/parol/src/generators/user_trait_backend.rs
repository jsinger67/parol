use crate::config::{CommonGeneratorConfig, UserTraitGeneratorConfig};
use crate::generators::UserTraitGenerator;
use crate::generators::user_trait_ir::UserTraitGenerationIR;
use anyhow::{Result, anyhow};

pub(crate) trait UserTraitLanguageBackend<C>
where
    C: CommonGeneratorConfig + UserTraitGeneratorConfig,
{
    fn generate_user_trait_source(
        &self,
        user_trait_ir: &mut UserTraitGenerationIR<'_, C>,
    ) -> Result<String>;
}

pub(crate) struct RustUserTraitBackend;

impl<C> UserTraitLanguageBackend<C> for RustUserTraitBackend
where
    C: CommonGeneratorConfig + UserTraitGeneratorConfig,
{
    fn generate_user_trait_source(
        &self,
        user_trait_ir: &mut UserTraitGenerationIR<'_, C>,
    ) -> Result<String> {
        let user_trait_generator = UserTraitGenerator::new(user_trait_ir.grammar_config);
        user_trait_generator.generate_user_trait_source(
            user_trait_ir.config,
            user_trait_ir.grammar_type,
            user_trait_ir.type_info,
        )
    }
}

pub(crate) struct CSharpUserTraitBackend;

impl<C> UserTraitLanguageBackend<C> for CSharpUserTraitBackend
where
    C: CommonGeneratorConfig + UserTraitGeneratorConfig,
{
    fn generate_user_trait_source(
        &self,
        user_trait_ir: &mut UserTraitGenerationIR<'_, C>,
    ) -> Result<String> {
        let user_trait_generator =
            crate::generators::cs_user_trait_generator::CSUserTraitGenerator::new(
                user_trait_ir.grammar_config,
            );
        user_trait_generator.generate_user_trait_source(
            user_trait_ir.config,
            user_trait_ir.grammar_type,
            user_trait_ir.type_info,
        )
    }
}

pub(crate) fn generate_user_trait_source_for_language<C>(
    backend: &impl UserTraitLanguageBackend<C>,
    user_trait_ir: &mut UserTraitGenerationIR<'_, C>,
) -> Result<String>
where
    C: CommonGeneratorConfig + UserTraitGeneratorConfig,
{
    if user_trait_ir.grammar_config.cfg.pr.is_empty() {
        return Err(anyhow!("Grammar contains no productions"));
    }
    backend.generate_user_trait_source(user_trait_ir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::InnerAttributes;
    use crate::config::{CommonGeneratorConfig, UserTraitGeneratorConfig};
    use crate::generators::{GrammarTypeInfo, UserTraitGenerator};
    use crate::utils::obtain_grammar_config;
    use std::path::PathBuf;

    const RUST_USER_TRAIT_OUTPUT_CHECKSUM: u64 = 9804298626913191341;
    const CSHARP_USER_TRAIT_OUTPUT_CHECKSUM: u64 = 15779923004694027088;

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
    fn rust_user_trait_backend_matches_direct_generation() {
        let grammar_config = obtain_grammar_config(test_grammar_path(), false).unwrap();
        let config = TestConfig;

        let mut type_info_backend = GrammarTypeInfo::try_new(config.user_type_name()).unwrap();
        let mut user_trait_ir = UserTraitGenerationIR::new(
            &grammar_config,
            &config,
            grammar_config.grammar_type,
            &mut type_info_backend,
        );
        let via_backend =
            generate_user_trait_source_for_language(&RustUserTraitBackend, &mut user_trait_ir)
                .unwrap();

        let mut type_info_direct = GrammarTypeInfo::try_new(config.user_type_name()).unwrap();
        let direct = UserTraitGenerator::new(&grammar_config)
            .generate_user_trait_source(&config, grammar_config.grammar_type, &mut type_info_direct)
            .unwrap();

        assert_eq!(direct, via_backend);
    }

    #[test]
    fn csharp_user_trait_backend_matches_direct_generation() {
        let grammar_config = obtain_grammar_config(test_grammar_path(), false).unwrap();
        let config = TestConfig;

        let mut type_info_backend = GrammarTypeInfo::try_new(config.user_type_name()).unwrap();
        let mut user_trait_ir = UserTraitGenerationIR::new(
            &grammar_config,
            &config,
            grammar_config.grammar_type,
            &mut type_info_backend,
        );
        let via_backend =
            generate_user_trait_source_for_language(&CSharpUserTraitBackend, &mut user_trait_ir)
                .unwrap();

        let mut type_info_direct = GrammarTypeInfo::try_new(config.user_type_name()).unwrap();
        let direct =
            crate::generators::cs_user_trait_generator::CSUserTraitGenerator::new(&grammar_config)
                .generate_user_trait_source(
                    &config,
                    grammar_config.grammar_type,
                    &mut type_info_direct,
                )
                .unwrap();

        assert_eq!(direct, via_backend);
    }

    #[test]
    fn rust_user_trait_output_checksum_stable() {
        let grammar_config = obtain_grammar_config(test_grammar_path(), false).unwrap();
        let config = TestConfig;
        let mut type_info = GrammarTypeInfo::try_new(config.user_type_name()).unwrap();

        let direct = UserTraitGenerator::new(&grammar_config)
            .generate_user_trait_source(&config, grammar_config.grammar_type, &mut type_info)
            .unwrap();

        let checksum = stable_checksum(&direct);
        assert_eq!(
            RUST_USER_TRAIT_OUTPUT_CHECKSUM, checksum,
            "Rust user-trait output checksum changed: {checksum}"
        );
    }

    #[test]
    fn csharp_user_trait_output_checksum_stable() {
        let grammar_config = obtain_grammar_config(test_grammar_path(), false).unwrap();
        let config = TestConfig;
        let mut type_info = GrammarTypeInfo::try_new(config.user_type_name()).unwrap();

        let direct =
            crate::generators::cs_user_trait_generator::CSUserTraitGenerator::new(&grammar_config)
                .generate_user_trait_source(&config, grammar_config.grammar_type, &mut type_info)
                .unwrap();

        let checksum = stable_checksum(&direct);
        assert_eq!(
            CSHARP_USER_TRAIT_OUTPUT_CHECKSUM, checksum,
            "C# user-trait output checksum changed: {checksum}"
        );
    }
}
