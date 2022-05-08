    /// Semantic action for {{^inner?}}user {{/inner}}production {{prod_num}}:
    /// 
    /// {{{prod_string}}}
    ///{{#named?}}
    #[named]{{/named}}
    fn {{fn_name}}(&mut self, {{{fn_arguments}}}) -> Result<()> {
        {{{code}}}Ok(())
    }
