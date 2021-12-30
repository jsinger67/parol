use crate::parser::{ParseTreeStackEntry, ParseTreeType};
use id_tree::Tree;
use miette::Result;
use std::path::Path;

///
/// This trait is used as a coupling point between the generated parser and
/// the user item that implements the actual semantic actions.
/// The trait is generated for the users item and implemented for it
/// automatically.
///
pub trait UserActionsTrait {
    ///
    /// Initialize the user with additional information.
    /// This function is called by the parser before paring starts.
    /// Is is used to transform necessary data from parser to user.
    ///
    fn init(&mut self, file_name: &Path);

    ///
    /// This function is implemented automatically for the user's item.
    ///
    fn call_semantic_action_for_production_number(
        &mut self,
        prod_num: usize,
        children: &[ParseTreeStackEntry],
        parse_tree: &Tree<ParseTreeType>,
    ) -> Result<()>;
}
