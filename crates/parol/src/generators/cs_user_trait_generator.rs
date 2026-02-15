use crate::GrammarTypeInfo;
use crate::config::CommonGeneratorConfig;
use crate::generators::{GrammarConfig, NamingHelper};
use crate::parser::GrammarType;
use anyhow::Result;
use std::fmt::Write;

/// Generator for C# user trait code.
pub struct CSUserTraitGenerator<'a> {
    grammar_config: &'a GrammarConfig,
}

impl<'a> CSUserTraitGenerator<'a> {
    /// Creates a new instance of the C# user trait generator.
    pub fn new(grammar_config: &'a GrammarConfig) -> Self {
        Self { grammar_config }
    }

    /// Generates a production-based action name for the given production index.
    ///
    /// Mirrors the Rust naming convention:
    /// - Single production for a non-terminal → `UpperCamelCase(non_terminal_name)`
    /// - Multiple alternatives → `UpperCamelCase(non_terminal_name + "_" + alternation_index)`
    fn action_name(&self, prod_index: usize) -> String {
        let pr = &self.grammar_config.cfg.pr[prod_index];
        let non_terminal = pr.get_n_str();
        let alts = self
            .grammar_config
            .cfg
            .get_alternations_count(prod_index)
            .unwrap_or(1);

        if alts == 1 {
            NamingHelper::to_upper_camel_case(non_terminal)
        } else {
            let rel_idx = self
                .grammar_config
                .cfg
                .get_alternation_index_of_production(prod_index)
                .unwrap_or(0);
            NamingHelper::to_upper_camel_case(&format!("{}_{}", non_terminal, rel_idx))
        }
    }

    /// Generates the C# user trait source code.
    pub fn generate_user_trait_source<C: CommonGeneratorConfig>(
        &self,
        config: &C,
        _grammar_type: GrammarType,
        _type_info: &mut GrammarTypeInfo,
    ) -> Result<String> {
        let mut source = String::new();
        let user_type_name = config.user_type_name();
        let interface_name = format!(
            "I{}Actions",
            NamingHelper::to_upper_camel_case(user_type_name)
        );
        let class_name = format!(
            "{}Actions",
            NamingHelper::to_upper_camel_case(user_type_name)
        );

        writeln!(source, "using System;")?;
        writeln!(source, "using Parol.Runtime;")?;
        writeln!(source, "using Parol.Runtime.Scanner;")?;
        writeln!(source)?;
        writeln!(source, "namespace {} {{", config.module_name())?;
        writeln!(source, "    /// <summary>")?;
        writeln!(
            source,
            "    /// User actions interface for the {} grammar.",
            user_type_name
        )?;
        writeln!(source, "    /// </summary>")?;
        writeln!(
            source,
            "    public interface {} : IUserActions {{",
            interface_name
        )?;

        for (i, pr) in self.grammar_config.cfg.pr.iter().enumerate() {
            let action_name = self.action_name(i);
            writeln!(source, "        /// <summary>")?;
            writeln!(source, "        /// Semantic action for production {}:", i)?;
            writeln!(source, "        /// {} ", pr)?;
            writeln!(source, "        /// </summary>")?;
            writeln!(source, "        void {}(object[] children);", action_name)?;
            writeln!(source)?;
        }

        writeln!(source, "    }}")?;
        writeln!(source)?;

        // Skeleton implementation
        writeln!(source, "    /// <summary>")?;
        writeln!(
            source,
            "    /// Base class for user actions for the {} grammar.",
            user_type_name
        )?;
        writeln!(source, "    /// </summary>")?;
        writeln!(
            source,
            "    public partial class {} : {} {{",
            class_name, interface_name
        )?;
        writeln!(source, "        /// <inheritdoc/>")?;
        writeln!(
            source,
            "        public virtual void CallSemanticActionForProductionNumber(int productionNumber, object[] children) {{"
        )?;
        writeln!(source, "            switch (productionNumber) {{")?;
        for (i, _) in self.grammar_config.cfg.pr.iter().enumerate() {
            let action_name = self.action_name(i);
            writeln!(
                source,
                "                case {}: {}(children); break;",
                i, action_name
            )?;
        }
        writeln!(
            source,
            "                default: throw new ArgumentException($\"Invalid production number {{productionNumber}}\");"
        )?;
        writeln!(source, "            }}")?;
        writeln!(source, "        }}")?;
        writeln!(source)?;
        writeln!(source, "        /// <inheritdoc/>")?;
        writeln!(
            source,
            "        public virtual void OnComment(Token token) {{ }}"
        )?;
        writeln!(source)?;

        for (i, pr) in self.grammar_config.cfg.pr.iter().enumerate() {
            let action_name = self.action_name(i);
            writeln!(source, "        /// <summary>")?;
            writeln!(source, "        /// Semantic action for production {}:", i)?;
            writeln!(source, "        /// {} ", pr)?;
            writeln!(source, "        /// </summary>")?;
            writeln!(
                source,
                "        public virtual void {}(object[] children) {{ }}",
                action_name
            )?;
        }

        writeln!(source, "    }}")?;
        writeln!(source, "}}")?;

        Ok(source)
    }
}
