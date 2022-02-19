use crate::StrVec;

#[derive(BartDisplay, Builder, Debug, Default)]
#[template = "templates/user_trait_caller_function_template.rs"]
pub(crate) struct UserTraitCallerFunctionData {
    fn_name: String,
    prod_num: usize,
    fn_arguments: String,
}

#[derive(BartDisplay, Builder, Debug, Default)]
#[template = "templates/user_trait_function_template.rs"]
pub(crate) struct UserTraitFunctionData {
    fn_name: String,
    prod_num: usize,
    fn_arguments: String,
    prod_string: String,
}

#[derive(BartDisplay, Builder, Debug, Default)]
#[template = "templates/user_trait_template.rs"]
pub(crate) struct UserTraitData<'a> {
    user_type_name: &'a str,
    trait_functions: StrVec,
    trait_caller: StrVec,
    user_trait_module_name: &'a str,
}
