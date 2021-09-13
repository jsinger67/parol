use super::errors::*;
use crate::parser::{AstStackEntry, AstType};
use id_tree::Tree;

///
/// This trait is used as the coupling point between the generated parser and
/// the user item that implements the actual semantic actions.
/// The trait is generated for the users item and implemented for it
/// automatically.
///
pub trait UserActionsTrait {
    ///
    /// This function is implemented automatically for the user's item.
    ///
    fn call_semantic_action_for_production_number(
        &mut self,
        prod_num: usize,
        children: &[AstStackEntry],
        parse_tree: &Tree<AstType>,
    ) -> Result<()>;
}
