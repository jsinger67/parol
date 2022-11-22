    /// Semantic action for {{^inner?}}non-terminal '{{non_terminal}}'{{/inner}}{{#inner?}}production {{prod_num}}:{{/inner}}{{#inner?}}
    ///
    /// {{{prod_string}}}
    ///{{/inner}}{{#named?}}
    #[parol_runtime::function_name::named]{{/named}}
    fn {{fn_name}}(&mut self, {{{fn_arguments}}}) -> Result<()> {
        {{{code}}}Ok(())
    }
