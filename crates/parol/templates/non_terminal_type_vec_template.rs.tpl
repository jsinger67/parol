{{#comment}}
/// {{{.}}}{{/comment}}
#[allow(dead_code)]
#[derive(Builder, Debug, Clone)]
#[builder(crate = "parol_runtime::derive_builder")]
pub struct {{non_terminal}} {
    vec: Vec<{{{type_ref}}}{{{lifetime}}}>}
