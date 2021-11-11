use crate::errors::*;

///
/// This trait is implemented by the TokenStream and provides some basic
/// functionality the user can call in his semantic actions.
/// The user's item is automatically initialized with this trait during parser
/// setup to make it accessible to the user.
///
pub trait ScannerAccess {
    ///
    /// This function provides the possibility to switch the active scanner from
    /// within a semantic action.
    ///
    fn switch_scanner(&mut self, scanner_name: &str) -> Result<()>;
}
