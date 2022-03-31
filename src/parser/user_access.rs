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
/// The lifetime parameter `'t` refers to the lifetime of the scanned text.
///
pub trait UserActionsTrait<'t> {
    ///
    /// Initialize the user with additional information.
    /// This function is called by the parser before parsing starts.
    /// Is is used to transport necessary data from parser to user.
    /// Is likely to be extended later.
    ///
    fn init(&mut self, file_name: &Path);

    ///
    /// This function is implemented automatically for the user's item.
    ///
    fn call_semantic_action_for_production_number(
        &mut self,
        prod_num: usize,
        children: &[ParseTreeStackEntry<'t>],
        parse_tree: &Tree<ParseTreeType<'t>>,
    ) -> Result<()>;
}
