use crate::oberon2_grammar_trait::{Oberon2, Oberon2GrammarTrait};
#[allow(unused_imports)]
use miette::Result;
use std::{
    fmt::{Debug, Display, Error, Formatter},
    path::{Path, PathBuf},
};

///
/// Data structure that implements the semantic actions for our Oberon2 grammar
/// !Change this type as needed!
///
#[derive(Debug, Default)]
pub struct Oberon2Grammar<'t> {
    pub oberon2: Option<Oberon2<'t>>,
    file_name: PathBuf,
}

impl Oberon2Grammar<'_> {
    pub fn new() -> Self {
        Oberon2Grammar::default()
    }
}

impl Display for Oberon2<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(f, "{:?}", self)
    }
}

impl Display for Oberon2Grammar<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match &self.oberon2 {
            Some(oberon2) => writeln!(f, "{}", oberon2),
            None => write!(f, "No parse result"),
        }
    }
}

impl<'t> Oberon2GrammarTrait<'t> for Oberon2Grammar<'t> {
    // !Adjust your implementation as needed!

    fn init(&mut self, file_name: &Path) {
        self.file_name = file_name.into();
    }

    /// Semantic action for non-terminal 'Oberon2'
    fn oberon2(&mut self, arg: &Oberon2<'t>) -> Result<()> {
        self.oberon2 = Some(arg.clone());
        Ok(())
    }
}
