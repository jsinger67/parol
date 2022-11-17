use crate::{{crate_name}}_grammar_trait::{ {{grammar_name}}, {{grammar_name}}GrammarTrait};
#[allow(unused_imports)]
use parol_runtime::miette::Result;
use std::fmt::{Debug, Display, Error, Formatter};

///
/// Data structure that implements the semantic actions for our {{grammar_name}} grammar
/// !Change this type as needed!
///
#[derive(Debug, Default)]
pub struct {{grammar_name}}Grammar<'t> {
    pub {{crate_name}}: Option<{{grammar_name}}<'t>>,
}

impl {{grammar_name}}Grammar<'_> {
    pub fn new() -> Self {
        {{grammar_name}}Grammar::default()
    }
}

impl Display for {{grammar_name}}<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(f, "{:?}", self)
    }
}

impl Display for {{grammar_name}}Grammar<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match &self.{{crate_name}} {
            Some({{crate_name}}) => writeln!(f, "{}", {{crate_name}}),
            None => write!(f, "No parse result"),
        }
    }
}

impl<'t> {{grammar_name}}GrammarTrait<'t> for {{grammar_name}}Grammar<'t> {
    // !Adjust your implementation as needed!

    /// Semantic action for non-terminal '{{grammar_name}}'
    fn {{crate_name}}(&mut self, arg: &{{grammar_name}}<'t>) -> Result<()> {
        self.{{crate_name}} = Some(arg.clone());
        Ok(())
    }
}
