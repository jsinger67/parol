use crate::errors::*;
use crate::parser::{ParseTreeStackEntry, ParseTreeType, ScannerAccess};
use id_tree::Tree;
use std::cell::RefCell;
use std::rc::Rc;

///
/// This trait is used as the coupling point between the generated parser and
/// the user item that implements the actual semantic actions.
/// The trait is generated for the users item and implemented for it
/// automatically.
///
pub trait UserActionsTrait: UserAccess {
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

///
/// This trait is automatically default implemented for the user's item.
/// The methods can be overwritten to be effective.
///
pub trait UserAccess {
    ///
    /// By calling this function the parser sets its access trait on the user's
    /// item.
    /// The lifetime 'a refers to the lifetime of the ScannerAccess trait object
    /// which actually is the TokenStream that is given as parameter of the
    /// `parse`function.
    ///
    fn set_scanner_access<'a>(&mut self, scanner_access: Rc<RefCell<dyn ScannerAccess + 'a>>);
}
