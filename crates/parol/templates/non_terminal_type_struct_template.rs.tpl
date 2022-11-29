{{#comment}}
/// {{{.}}}{{/comment}}
#[allow(dead_code)]
#[derive(Builder, Debug, Clone)]
#[builder(crate = "parol_runtime::derive_builder")]
pub struct {{type_name}}{{{lifetime}}} {
{{#members}}
  pub {{{.}}}{{/members}}}
