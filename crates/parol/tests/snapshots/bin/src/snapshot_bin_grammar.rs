use crate::snapshot_bin_grammar_trait::{SnapshotBin, SnapshotBinGrammarTrait};
#[allow(unused_imports)]
use parol_runtime::Result;
use std::fmt::{Debug, Display, Error, Formatter};

///
/// Data structure that implements the semantic actions for our SnapshotBin grammar
/// !Change this type as needed!
///
#[derive(Debug, Default)]
pub struct SnapshotBinGrammar<'t> {
    pub snapshot_bin: Option<SnapshotBin<'t>>,
}

impl SnapshotBinGrammar<'_> {
    pub fn new() -> Self {
        SnapshotBinGrammar::default()
    }
}

impl Display for SnapshotBin<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        write!(f, "{:?}", self)
    }
}

impl Display for SnapshotBinGrammar<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match &self.snapshot_bin {
            Some(snapshot_bin) => writeln!(f, "{}", snapshot_bin),
            None => write!(f, "No parse result"),
        }
    }
}

impl<'t> SnapshotBinGrammarTrait<'t> for SnapshotBinGrammar<'t> {
    // !Adjust your implementation as needed!

    /// Semantic action for non-terminal 'SnapshotBin'
    fn snapshot_bin(&mut self, arg: &SnapshotBin<'t>) -> Result<()> {
        self.snapshot_bin = Some(arg.clone());
        Ok(())
    }
}
