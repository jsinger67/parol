use derive_builder::Builder;

#[derive(Builder, Debug, Default)]
pub(crate) struct ProgramCsData<'a> {
    pub(crate) crate_name: &'a str,
    pub(crate) grammar_name: String,
    pub(crate) user_type_name: &'a str,
}

impl std::fmt::Display for ProgramCsData<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ProgramCsData {
            crate_name: _crate_name,
            grammar_name,
            user_type_name,
        } = self;

        write!(
            f,
            r#"using System;
using System.IO;
using {user_type_name};

namespace {user_type_name}
{{
    class Program
    {{
        // Parses a file provided on the command line and prints the semantic result.
        static void Main(string[] args)
        {{
            if (args.Length < 1)
            {{
                Console.WriteLine("Please provide a file name as first parameter!");
                return;
            }}

            string fileName = args[0];
            // Read the full input so the parser can process it in one pass.
            string input = File.ReadAllText(fileName);
            // User actions collect the typed parse result during semantic callbacks.
            I{grammar_name}Actions actions = new {grammar_name}UserActions();

            try
            {{
                {grammar_name}Parser.Parse(input, fileName, actions);
                Console.WriteLine("Success!");
                Console.WriteLine(actions.ToString());
            }}
            catch (Exception e)
            {{
                Console.WriteLine($"Error: {{e.Message}}");
            }}
        }}
    }}
}}
"#
        )
    }
}
