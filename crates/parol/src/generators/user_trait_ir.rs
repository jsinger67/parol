use crate::config::{CommonGeneratorConfig, UserTraitGeneratorConfig};
use crate::generators::{GrammarConfig, GrammarTypeInfo};
use crate::parser::GrammarType;

pub(crate) struct UserTraitGenerationIR<'a, C>
where
    C: CommonGeneratorConfig + UserTraitGeneratorConfig,
{
    pub(crate) grammar_config: &'a GrammarConfig,
    pub(crate) config: &'a C,
    pub(crate) grammar_type: GrammarType,
    pub(crate) type_info: &'a mut GrammarTypeInfo,
}

impl<'a, C> UserTraitGenerationIR<'a, C>
where
    C: CommonGeneratorConfig + UserTraitGeneratorConfig,
{
    pub(crate) fn new(
        grammar_config: &'a GrammarConfig,
        config: &'a C,
        grammar_type: GrammarType,
        type_info: &'a mut GrammarTypeInfo,
    ) -> Self {
        Self {
            grammar_config,
            config,
            grammar_type,
            type_info,
        }
    }
}
