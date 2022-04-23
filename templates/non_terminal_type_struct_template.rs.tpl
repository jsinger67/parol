{{#comment}}
/// {{{.}}}{{/comment}}
#[allow(dead_code)]
#[derive(Builder, Debug, Clone)]
pub struct {{struct_type_name}}{{{lifetime}}} {
{{#members}}
  pub {{{.}}}{{/members}}}
