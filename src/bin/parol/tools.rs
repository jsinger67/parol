//! A registry for all the extra tools that can be used with parol.

pub type ToolFunc = fn(&[&str]) -> miette::Result<()>;

pub fn get_tool_main(name: &str) -> Option<ToolFunc> {
    TOOLS
        .iter()
        .find(|(actual_name, _)| *actual_name == name)
        .map(|tool| tool.1)
}

pub fn names() -> impl Iterator<Item = &'static str> {
    TOOLS.iter().map(|(name, _)| *name)
}

/*
 * For each specified tool name this
 *
 * 1. Declares `mod $tool`
 * 2. Registers it in the table of tools
 */
macro_rules! declare_tools {
    ($($tool:ident),*) => {
        $(mod $tool;)*
        pub static TOOLS: &[(&str, ToolFunc)] = &[
            $((stringify!($tool), self::$tool::main)),*
        ];
    }
}

declare_tools!(
    calculate_k,
    calculate_k_tuples,
    decidable,
    first,
    follow,
    format,
    generate,
    left_factor,
    left_recursions,
    productivity,
    serialize
);
