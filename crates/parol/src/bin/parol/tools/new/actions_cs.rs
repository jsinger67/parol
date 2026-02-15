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
    public partial class {grammar_name}Actions : I{grammar_name}Actions
    {{
        public override string ToString() => "Grammar Result";
    }}
}}
"#
        )
    }
}
