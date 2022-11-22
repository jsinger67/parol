{{#comment}}
/// {{{.}}}{{/comment}}
#[allow(dead_code)]
#[derive(Builder, Debug, Clone)]
pub struct {{type_name}}{{{lifetime}}} {
{{#members}}
  pub {{{.}}}{{/members}}}
