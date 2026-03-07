use derive_builder::Builder;

#[derive(Builder, Debug, Default)]
pub(crate) struct ActionsCsData<'a> {
    pub(crate) grammar_name: String,
    pub(crate) user_type_name: &'a str,
}

impl std::fmt::Display for ActionsCsData<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ActionsCsData {
            grammar_name,
            user_type_name,
        } = self;

        write!(
            f,
            r#"using System;
using Parol.Runtime;

namespace {user_type_name}
{{
    // Extend the generated base actions with user-defined result handling.
    public partial class {grammar_name}UserActions : {grammar_name}Actions
    {{
        // Stores the start-symbol value so it can be easily used for grammar processing.
        private {grammar_name}? _parseResult;

        // Expose the parse result in a simple form for the scaffolded Program output.
        public override string ToString() => _parseResult?.ToString() ?? string.Empty;

        // Called when the start symbol has been parsed. Contains the processed input.
        public override void On{grammar_name}({grammar_name} arg)
        {{
            _parseResult = arg;
        }}
    }}
}}
"#
        )
    }
}
