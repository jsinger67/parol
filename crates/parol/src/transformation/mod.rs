///
/// Module with grammar transformations to canonicalize grammar.
/// This is actually the transformation from an EBNF like structure to a BNF like structure.
///
pub mod canonicalization;
pub(crate) use canonicalization::transform_productions;

///
/// Module with left-factoring functionality
///
pub mod left_factoring;
pub use left_factoring::left_factor;

/// Module that handles the augmentation of the grammar with a new start symbol for LR parsing
pub mod lr_augmentation;
pub use lr_augmentation::augment_grammar;