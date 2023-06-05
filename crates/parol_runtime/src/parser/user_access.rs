use crate::{ParseTreeType, Result, Token};

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
    /// This function is implemented automatically for the user's item.
    ///
    fn call_semantic_action_for_production_number(
        &mut self,
        prod_num: usize,
        children: &[ParseTreeType<'t>],
    ) -> Result<()>;

    ///
    /// This function is called when a token is parsed that is associated to the
    /// user defined terminals declared by
    /// * %line_comment
    /// and
    /// * %block_comment
    /// directives.
    /// This can improve handling of comments that are not captured by the grammar definition
    /// itself.
    ///
    fn on_comment_parsed(&mut self, _token: Token<'t>);
}
