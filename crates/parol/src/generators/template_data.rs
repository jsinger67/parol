use crate::StrVec;
use std::fmt::Write;

#[derive(Builder, Debug, Default)]
pub(crate) struct UserTraitCallerFunctionData {
    fn_name: String,
    prod_num: usize,
    fn_arguments: String,
}

impl std::fmt::Display for UserTraitCallerFunctionData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let UserTraitCallerFunctionData {
            fn_name,
            prod_num,
            fn_arguments,
        } = self;
        f.write_fmt(ume::ume!(#prod_num => self.#fn_name(#fn_arguments),))
    }
}

#[derive(Builder, Debug, Default)]
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

impl std::fmt::Display for UserTraitFunctionData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let UserTraitFunctionData {
            fn_name,
            prod_num,
            fn_arguments,
            prod_string,
            non_terminal,
            named,
            code,
            inner,
        } = self;
        if *inner {
            writeln!(
                f,
                "
                /// Semantic action for production {prod_num}:
                ///
                /// `{prod_string}`
                ///"
            )?;
        } else {
            writeln!(f, "/// Semantic action for non-terminal '{non_terminal}'")?;
        }

        if *named {
            f.write_fmt(ume::ume!(#[parol_runtime::function_name::named]))?;
        }
        f.write_fmt(ume::ume! {
            fn #fn_name(&mut self, #fn_arguments) -> Result<()> {
                #code
                Ok(())
            }
        })
    }
}

#[derive(Builder, Debug, Default)]
pub(crate) struct UserTraitFunctionStackPopData {
    pub arg_name: String,
    pub arg_type: String,
    pub vec_anchor: bool,
    pub vec_push_semantic: bool,
}

impl std::fmt::Display for UserTraitFunctionStackPopData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let UserTraitFunctionStackPopData {
            arg_name,
            arg_type,
            vec_anchor,
            vec_push_semantic,
        } = self;
        let mutability = if *vec_push_semantic { "mut" } else { "" };
        let macro_name = if *vec_anchor {
            "pop_and_reverse_item"
        } else {
            "pop_item"
        };
        f.write_fmt(ume::ume! {
            let #mutability #arg_name = #macro_name!(self, #arg_name, #arg_type, context);
        })
    }
}

#[derive(Builder, Debug, Default)]
pub(crate) struct UserTraitData<'a> {
    pub user_type_name: &'a str,
    pub auto_generate: bool,
    pub range: bool,
    pub user_provided_attributes: StrVec,
    pub production_output_types: StrVec,
    pub non_terminal_types: StrVec,
    pub ast_type_decl: String,
    pub ast_type_has_lifetime: bool,
    pub trait_functions: StrVec,
    pub trait_caller: StrVec,
    pub module_name: &'a str,
    pub user_trait_functions: StrVec,
}

impl std::fmt::Display for UserTraitData<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let UserTraitData {
            user_type_name,
            auto_generate,
            range,
            user_provided_attributes,
            production_output_types,
            non_terminal_types,
            ast_type_decl,
            ast_type_has_lifetime,
            trait_functions,
            trait_caller,
            module_name,
            user_trait_functions,
        } = self;

        write!(
            f,
            "
            // ---------------------------------------------------------
            // This file was generated by parol.
            // It is not intended for manual editing and changes will be
            // lost after next build.
            // ---------------------------------------------------------

            // Disable clippy warnings that can result in the way how parol generates code.
            "
        )?;

        f.write_fmt(ume::ume! {
            #user_provided_attributes
            #![allow(clippy::enum_variant_names)]
            #![allow(clippy::large_enum_variant)]
            #![allow(clippy::upper_case_acronyms)]
        })?;

        writeln!(f, "\n")?;

        if *range {
            f.write_fmt(ume::ume!(
                use parol_runtime::{Span, ToSpan};
            ))?;
        }
        if *auto_generate {
            f.write_fmt(ume::ume! {
                use parol_runtime::derive_builder::Builder;
                #[allow(unused_imports)]
                use parol_runtime::parol_macros::{pop_item, pop_and_reverse_item};
                use parol_runtime::log::trace;
            })?;
        } else {
            f.write_fmt(ume::ume! {
                use crate::#module_name::#user_type_name;
            })?;
        }
        if !*ast_type_has_lifetime {
            f.write_fmt(ume::ume!(
                use std::marker::PhantomData;
            ))?;
        }
        f.write_fmt(ume::ume! {
            use parol_runtime::parser::{ParseTreeType, UserActionsTrait};
            use parol_runtime::{ParserError, Result, Token};
        })?;

        let trait_name = format!("{}Trait", user_type_name);
        let blank_line = "\n\n";
        let call_semantic_action_for_production_number_doc = format!(
            "
		///
		/// This function is implemented automatically for the user's item {user_type_name}.
		///
		"
        );
        if *auto_generate {
            let lifetime = if *ast_type_has_lifetime { "<'t>" } else { "" };
            let anonymous_lifetime = if *ast_type_has_lifetime {
                "<'t>"
            } else {
                "<'_>"
            };
            let auto_name = format!("{}Auto", user_type_name);
            writeln!(
                f,
                "

                /// Semantic actions trait generated for the user grammar
                /// All functions have default implementations."
            )?;
            let user_trait_functions = user_trait_functions.join("\n\n");
            let on_comment_parsed_comment = r"

                /// This method provides skipped language comments.
                /// If you need comments please provide your own implementation of this method.
            ";
            f.write_fmt(ume::ume! {
                pub trait #trait_name #lifetime {
                    #user_trait_functions
                    #on_comment_parsed_comment
                    fn on_comment_parsed(&mut self, _token: Token #anonymous_lifetime) {}
                }
            })?;

            let production_output_types = production_output_types.join("\n\n");
            let non_terminal_types = non_terminal_types.join("\n\n");
            write!(f, "

                // -------------------------------------------------------------------------------------------------
                //
                // Output Types of productions deduced from the structure of the transformed grammar
                //

                {production_output_types}

                // -------------------------------------------------------------------------------------------------
                //
                // Types of non-terminals deduced from the structure of the transformed grammar
                //

                {non_terminal_types}

                // -------------------------------------------------------------------------------------------------

                {ast_type_decl}
                ")?;

            let phantom_data_field = if *ast_type_has_lifetime {
                "".into()
            } else {
                let comment = "\n// Just to hold the lifetime generated by parol\n";
                ume::ume! {
                    #comment
                    phantom: PhantomData<&'t str>,
                }
                .to_string()
            };
            writeln!(
                f,
                "
                /// Auto-implemented adapter grammar
                ///
                /// The lifetime parameter `'t` refers to the lifetime of the scanned text.
                /// The lifetime parameter `'u` refers to the lifetime of user grammar object.
                ///"
            )?;

            let user_grammar_comment = "// Mutable reference of the actual user grammar to be able to call the semantic actions on it\n";
            let item_stack_comment = "\n// Stack to construct the AST on it\n";
            f.write_fmt(ume::ume! {
                #[allow(dead_code)]
                pub struct #auto_name<'t, 'u> where 't: 'u {
                    #user_grammar_comment
                    user_grammar: &'u mut dyn #trait_name #lifetime,
                    #item_stack_comment
                    item_stack: Vec<ASTType #lifetime>,
                    #phantom_data_field
                }
            })?;

            let phantom_data_field_default = if *ast_type_has_lifetime {
                "".into()
            } else {
                ume::ume! {
                    phantom: PhantomData,
                }
                .to_string()
            };
            writeln!(
                f,
                "

                ///
                /// The `{user_type_name}Auto` impl is automatically generated for the
                /// given grammar.
                ///"
            )?;
            let trace_item_stack_comment = r#"
			// Use this function for debugging purposes:
			// trace!("{}", self.trace_item_stack(context));
			"#;
            f.write_fmt(ume::ume! {
                impl<'t, 'u> #auto_name<'t, 'u> {
                    pub fn new(user_grammar: &'u mut dyn #trait_name #lifetime) -> Self {
                        Self {
                            user_grammar,
                            item_stack: Vec::new(),
                            #phantom_data_field_default
                        }
                    }
                    #blank_line
                    #[allow(dead_code)]
                    fn push(&mut self, item: ASTType #lifetime, context: &str) {
                        trace!("push    {}: {:?}", context, item);
                        self.item_stack.push(item)
                    }
                    #blank_line
                    #[allow(dead_code)]
                    fn pop(&mut self, context: &str) -> Option<ASTType #lifetime> {
                        let item = self.item_stack.pop();
                        if let Some(ref item) = item {
                            trace!("pop     {}: {:?}", context, item);
                        }
                        item
                    }
                    #blank_line
                    #[allow(dead_code)]
                    #trace_item_stack_comment
                    fn trace_item_stack(&self, context: &str) -> std::string::String {
                        format!(
                            "Item stack at {}:\n{}",
                            context,
                            self.item_stack
                                .iter()
                                .rev()
                                .map(|s| format!("  {:?}", s))
                                .collect::<Vec<std::string::String>>()
                                .join("\n")
                        )
                    }
                    #blank_line
                    #trait_functions
                }
            })?;

            writeln!(f, "\n")?;
            f.write_fmt(ume::ume! {
                #blank_line
                impl<'t> UserActionsTrait<'t> for #auto_name<'t, '_> {
                    #call_semantic_action_for_production_number_doc
                    fn call_semantic_action_for_production_number(
                        &mut self,
                        prod_num: usize,
                        children: &[ParseTreeType<'t>]) -> Result<()> {
                        match prod_num {
                            #trait_caller
                            _ => Err(ParserError::InternalError(format!("Unhandled production number: {}", prod_num)).into()),
                        }
                    }
                    #blank_line
                    fn on_comment_parsed(&mut self, token: Token<'t>) {
                        self.user_grammar.on_comment_parsed(token)
                    }
                }
            })?;
        } else {
            writeln!(
                f,
                "
                ///
                /// The `{user_type_name}Trait` trait is automatically generated for the
                /// given grammar.
                /// All functions have default implementations.
                ///"
            )?;
            let supported_comment = r"
                // This is currently only supported for auto generate mode.
                // Please, file an issue if need arises.
                ";
            f.write_fmt(ume::ume! {
                pub trait #trait_name {
                    #trait_functions
                }
                #blank_line
                #blank_line
                impl UserActionsTrait<'_> for #user_type_name {
                    #call_semantic_action_for_production_number_doc
                    fn call_semantic_action_for_production_number(
                        &mut self,
                        prod_num: usize,
                        children: &[ParseTreeType]) -> Result<()> {
                        match prod_num {
                            #trait_caller
                            _ => Err(ParserError::InternalError(format!("Unhandled production number: {}", prod_num)).into()),
                        }
                    }

                    fn on_comment_parsed(&mut self, _token: Token<'_>) {
                        #supported_comment
                    }
                }
            })?;
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub(crate) struct NonTerminalTypeStruct {
    pub comment: StrVec,
    pub type_name: String,
    pub lifetime: String,
    pub members: StrVec,
}

impl std::fmt::Display for NonTerminalTypeStruct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let NonTerminalTypeStruct {
            comment,
            type_name,
            lifetime,
            members,
        } = self;
        for comment in comment {
            writeln!(f, "/// {}", comment)?
        }
        let members = members.iter().fold(String::new(), |mut output, member| {
            let _ = writeln!(output, "pub {member}");
            output
        });
        f.write_fmt(ume::ume! {
            #[allow(dead_code)]
            #[derive(Builder, Debug, Clone)]
            #[builder(crate = "parol_runtime::derive_builder")]
            pub struct #type_name #lifetime {
                #members
            }
        })
    }
}

#[derive(Debug, Default)]
pub(crate) struct NonTerminalTypeEnum {
    pub comment: StrVec,
    pub type_name: String,
    pub lifetime: String,
    pub members: StrVec,
}

impl std::fmt::Display for NonTerminalTypeEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let NonTerminalTypeEnum {
            comment,
            type_name,
            lifetime,
            members,
        } = self;
        for comment in comment {
            writeln!(f, "/// {}", comment)?
        }
        f.write_fmt(ume::ume! {
            #[allow(dead_code)]
            #[derive(Debug, Clone)]
            pub enum #type_name #lifetime {
                #members
            }
        })
    }
}

#[derive(Builder, Debug, Default)]
pub(crate) struct RangeCalculation {
    pub type_name: String,
    pub lifetime: String,
    #[builder(default)]
    pub code: StrVec,
}

impl std::fmt::Display for RangeCalculation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let RangeCalculation {
            type_name,
            lifetime,
            code,
        } = self;
        f.write_fmt(ume::ume! {
            impl #lifetime ToSpan for #type_name #lifetime {
                fn span(&self) -> Span {
                    #code
                }
            }
        })
    }
}
