use crate::StrVec;

#[derive(BartDisplay, Builder, Debug, Default)]
#[template = "templates/user_trait_caller_function_template.rs.tpl"]
pub(crate) struct UserTraitCallerFunctionData {
    fn_name: String,
    prod_num: usize,
    fn_arguments: String,
}

#[derive(BartDisplay, Builder, Debug, Default)]
#[template = "templates/user_trait_function_template.rs.tpl"]
pub(crate) struct UserTraitFunctionData {
    pub fn_name: String,
    #[builder(default)]
    pub prod_num: usize,
    pub fn_arguments: String,
    #[builder(default)]
    pub prod_string: String,
    #[builder(default)]
    pub non_terminal: String,
    // This is used to control whether the #[named] is generated
    #[builder(default)]
    pub named: bool,
    #[builder(default)]
    pub code: StrVec,
    // Inner means the expanded version of the grammar.
    // If set to false the actual user grammar is meant.
    #[builder(default)]
    pub inner: bool,
}

#[derive(BartDisplay, Builder, Debug, Default)]
#[template = "templates/user_trait_function_stack_pop_template.rs.tpl"]
pub(crate) struct UserTraitFunctionStackPopData {
    pub arg_name: String,
    pub arg_type: String,
    pub vec_anchor: bool,
    pub vec_push_semantic: bool,
}

#[derive(BartDisplay, Builder, Debug, Default)]
#[template = "templates/user_trait_template.rs.tpl"]
pub(crate) struct UserTraitData<'a> {
    pub user_type_name: &'a str,
    pub auto_generate: bool,
    pub production_output_types: StrVec,
    pub non_terminal_types: StrVec,
    pub ast_type_decl: String,
    pub trait_functions: StrVec,
    pub trait_caller: StrVec,
    pub module_name: &'a str,
    pub user_trait_functions: StrVec,
}

#[derive(BartDisplay, Debug, Default)]
#[template = "templates/non_terminal_type_struct_template.rs.tpl"]
pub(crate) struct NonTerminalTypeStruct {
    pub comment: StrVec,
    pub type_name: String,
    pub lifetime: String,
    pub members: StrVec,
}

#[derive(BartDisplay, Debug, Default)]
#[template = "templates/non_terminal_type_enum_template.rs.tpl"]
pub(crate) struct NonTerminalTypeEnum {
    pub comment: StrVec,
    pub type_name: String,
    pub lifetime: String,
    pub members: StrVec,
}

#[derive(BartDisplay, Debug, Default)]
#[template = "templates/non_terminal_type_vec_template.rs.tpl"]
pub(crate) struct NonTerminalTypeVec {
    pub comment: StrVec,
    pub non_terminal: String,
    pub lifetime: String,
    pub type_ref: String,
}
