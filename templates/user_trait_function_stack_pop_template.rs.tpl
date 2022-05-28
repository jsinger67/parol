        let {{#vec_push_semantic?}}mut {{/vec_push_semantic}}{{arg_name}} = if let Some(ASTType::{{{arg_type}}}({{#vec_anchor?}}mut {{/vec_anchor}}{{arg_name}})) = self.pop(context) {
            {{#vec_anchor?}}{{arg_name}}.reverse();
            {{/vec_anchor}}{{arg_name}}
        } else {
            bail!("{}: Expecting ASTType::{{{arg_type}}}", context);
        };