use crate::InnerAttributes;

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
/// Common configuration of both parser generator an user trait generator.
pub trait CommonGeneratorConfig {
    /// User type that implements the language processing
    fn user_type_name(&self) -> &str;
    /// User type's module name
    fn module_name(&self) -> &str;
    /// Activate the minimization of boxed types in the generated parser
    fn minimize_boxed_types(&self) -> bool;
    /// Generate range information for AST types
    fn range(&self) -> bool;
    /// Generate typed syntree node wrappers
    fn syntree_node_wrappers(&self) -> bool;
}

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
/// Configuration of parser generator
pub trait ParserGeneratorConfig {
    /// Enables trimming of the parse tree during parsing.
    /// Generates the call to trim_parse_tree on the parser object before the call of parse.
    fn trim_parse_tree(&self) -> bool;

    /// If true error recovery in the generated parser should be disabled.
    fn recovery_disabled(&self) -> bool;
}

// ---------------------------------------------------
// Part of the Public API
// *Changes will affect crate's version according to semver*
// ---------------------------------------------------
/// Configuration of user trait generator
pub trait UserTraitGeneratorConfig {
    /// Inserts the given inner attributes at the top of the generated trait source.
    fn inner_attributes(&self) -> &[InnerAttributes];
}
