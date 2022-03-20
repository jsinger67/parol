//! A registry for all the extra tools that can be used with parol.

/*
 * For each specified tool name this
 *
 * 1. Declares `mod $tool`
 * 2. Registers it in the table of tools
 */
macro_rules! declare_tools {
    ($($tool:ident),*) => {
        $(mod $tool;)*

        #[derive(clap::Subcommand)]
        #[allow(non_camel_case_types)]
        pub enum ToolsSubcommands {
            $(
                $tool($tool ::Args),
            )*
        }

        impl ToolsSubcommands {
            pub fn invoke_main(&self) -> miette::Result<()> {
                match self {
                    $(
                        ToolsSubcommands::$tool(args) => {
                            self::$tool::main(args)
                        }
                    )*
                }
            }
        }
    }
}

declare_tools!(
    calculate_k,
    calculate_k_tuples,
    decidable,
    deduce_types,
    first,
    follow,
    format,
    generate,
    left_factor,
    left_recursions,
    productivity,
    serialize
);
