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
    fn_name: String,
    prod_num: usize,
    fn_arguments: String,
    prod_string: String,
}

#[derive(BartDisplay, Builder, Debug, Default)]
#[template = "templates/user_trait_template.rs.tpl"]
pub(crate) struct UserTraitData<'a> {
    pub user_type_name: &'a str,
    pub auto_generate: bool,
    pub production_output_types: StrVec,
    pub non_terminal_types: StrVec,
    pub ast_type_decl: &'a str,
    pub trait_functions: StrVec,
    pub trait_caller: StrVec,
    pub module_name: &'a str,
    pub user_trait_functions: StrVec,
}

#[derive(BartDisplay, Debug, Default)]
#[template = "templates/non_terminal_type_struct_template.rs.tpl"]
pub(crate) struct NonTerminalTypeStruct {
    pub comment: String,
    pub non_terminal: String,
    pub members: StrVec,
}

#[derive(BartDisplay, Debug, Default)]
#[template = "templates/non_terminal_type_enum_template.rs.tpl"]
pub(crate) struct NonTerminalTypeEnum {
    pub comment: String,
    pub non_terminal: String,
    pub members: StrVec,
}

#[derive(BartDisplay, Debug, Default)]
#[template = "templates/non_terminal_type_vec_template.rs.tpl"]
pub(crate) struct NonTerminalTypeVec {
    pub comment: String,
    pub non_terminal: String,
    pub type_ref: String,
}
