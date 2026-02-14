use crate::parol_ls_grammar_trait::{
    SimpleToken, SimpleTokenOpt, TokenExpression, TokenExpressionOpt, TokenWithStates,
    TokenWithStatesOpt,
};

use super::{Comments, Fmt, FmtOptions};

impl Fmt for SimpleToken {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        let (token_literal, comments) = self.token_expression.txt(options, comments);
        let (simple_token_opt, comments) =
            if let Some(simple_token_opt) = self.simple_token_opt.as_ref() {
                simple_token_opt.txt(options, comments)
            } else {
                (String::default(), comments)
            };
        (format!("{token_literal}{simple_token_opt}"), comments)
    }
}

impl Fmt for SimpleTokenOpt {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        self.a_s_t_control.txt(options, comments)
    }
}

impl Fmt for TokenExpression {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        let (token_literal, comments) = self.token_literal.txt(options, comments);
        let (token_expression_opt, comments) =
            if let Some(token_expression_opt) = self.token_expression_opt.as_ref() {
                token_expression_opt.txt(options, comments)
            } else {
                (String::default(), comments)
            };
        (format!("{token_literal}{token_expression_opt}"), comments)
    }
}

impl Fmt for TokenExpressionOpt {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        let (la, comments) = self.look_ahead.txt(options, comments);
        (la.to_string(), comments)
    }
}

impl Fmt for TokenWithStates {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        let (mut state_list, comments) = self.identifier_list.txt(options, comments);
        let (token_literal, comments) = self.token_expression.txt(options, comments);
        let (token_with_states_opt, comments) =
            if let Some(token_with_states_opt) = self.token_with_states_opt.as_ref() {
                token_with_states_opt.txt(options, comments)
            } else {
                (String::default(), comments)
            };
        state_list.clone_from(&state_list.trim().to_owned());
        (
            format!(
                "{}{}{}{}{}",
                self.l_t, state_list, self.g_t, token_literal, token_with_states_opt
            ),
            comments,
        )
    }
}

impl Fmt for TokenWithStatesOpt {
    fn txt(&self, options: &FmtOptions, comments: Comments) -> (String, Comments) {
        self.a_s_t_control.txt(options, comments)
    }
}
