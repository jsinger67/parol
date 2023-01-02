use crate::{
    parser::{ParseTreeStackEntry, ParseTreeType},
    ParolError,
};
use id_tree::Tree;

///
/// This trait is used as a coupling point between the generated parser and
/// the user item that implements the actual semantic actions.
/// The trait is generated for the users item and implemented for it
/// automatically.
///
/// The lifetime parameter `'t` refers to the lifetime of the scanned text.
///
pub trait UserActionsTrait<'t> {
    /// The type of errors that can be returned by the user's semantic actions.
    type UserError: std::error::Error;
    ///
    /// This function is implemented automatically for the user's item.
    ///
    fn call_semantic_action_for_production_number(
        &mut self,
        prod_num: usize,
        children: &[ParseTreeStackEntry<'t>],
        parse_tree: &Tree<ParseTreeType<'t>>,
    ) -> Result<(), ParolError<Self::UserError>>;
}
