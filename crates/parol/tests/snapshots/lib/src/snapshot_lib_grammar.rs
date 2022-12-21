use crate::snapshot_lib_grammar_trait::{ SnapshotLib, SnapshotLibGrammarTrait};
#[allow(unused_imports)]
use parol_runtime::miette::Result;
use std::fmt::{Debug, Display, Error, Formatter};

///
/// Data structure that implements the semantic actions for our SnapshotLib grammar
/// !Change this type as needed!
///
#[derive(Debug, Default)]
pub struct SnapshotLibGrammar<'t> {
    pub snapshot_lib: Option<SnapshotLib<'t>>,
}

impl SnapshotLibGrammar<'_> {
    pub fn new() -> Self {
        SnapshotLibGrammar::default()
    }
}

impl Display for SnapshotLib<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(f, "{:?}", self)
    }
}

impl Display for SnapshotLibGrammar<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match &self.snapshot_lib {
            Some(snapshot_lib) => writeln!(f, "{}", snapshot_lib),
            None => write!(f, "No parse result"),
        }
    }
}

impl<'t> SnapshotLibGrammarTrait<'t> for SnapshotLibGrammar<'t> {
    // !Adjust your implementation as needed!

    /// Semantic action for non-terminal 'SnapshotLib'
    fn snapshot_lib(&mut self, arg: &SnapshotLib<'t>) -> Result<()> {
        self.snapshot_lib = Some(arg.clone());
        Ok(())
    }
}
