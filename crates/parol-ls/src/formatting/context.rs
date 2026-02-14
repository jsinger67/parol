use super::{FmtOptions, Line};

#[derive(Debug, Clone, Copy, Default)]
struct FormatterState {
    semicolon_starts_on_new_line: bool,
    directive_starts_on_new_line: bool,
    declaration_starts_on_new_line: bool,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct FormatterContext<'a> {
    policy: &'a FmtOptions,
    state: FormatterState,
}

impl<'a> FormatterContext<'a> {
    pub(crate) fn new(policy: &'a FmtOptions) -> Self {
        Self {
            policy,
            state: FormatterState::default(),
        }
    }

    pub(crate) fn with_semicolon_starts_on_new_line(
        mut self,
        semicolon_starts_on_new_line: bool,
    ) -> Self {
        self.state.semicolon_starts_on_new_line = semicolon_starts_on_new_line;
        self
    }

    pub(crate) fn with_directive_starts_on_new_line(
        mut self,
        directive_starts_on_new_line: bool,
    ) -> Self {
        self.state.directive_starts_on_new_line = directive_starts_on_new_line;
        self
    }

    pub(crate) fn with_declaration_starts_on_new_line(
        mut self,
        declaration_starts_on_new_line: bool,
    ) -> Self {
        self.state.declaration_starts_on_new_line = declaration_starts_on_new_line;
        self
    }

    pub(crate) fn policy(&self) -> &FmtOptions {
        self.policy
    }

    pub(crate) fn semicolon_starts_on_new_line(&self) -> bool {
        self.state.semicolon_starts_on_new_line
    }

    pub(crate) fn directive_starts_on_new_line(&self) -> bool {
        self.state.directive_starts_on_new_line
    }

    pub(crate) fn declaration_starts_on_new_line(&self) -> bool {
        self.state.declaration_starts_on_new_line
    }

    pub(crate) fn empty_line_after_prod(&self) -> bool {
        self.policy.empty_line_after_prod
    }
}

pub(crate) fn context_for_scanner_directive<'a>(
    context: &FormatterContext<'a>,
    comments_before_token: &str,
) -> FormatterContext<'a> {
    context
        .with_directive_starts_on_new_line(leading_token_delimiter(comments_before_token) == "\n")
}

pub(crate) fn scanner_directive_indent(
    base_indent: &str,
    context: &FormatterContext<'_>,
) -> String {
    if context.directive_starts_on_new_line() {
        format!("\n{base_indent}")
    } else {
        base_indent.to_string()
    }
}

pub(crate) fn context_for_declaration<'a>(
    context: &FormatterContext<'a>,
    comments_before_token: &str,
) -> FormatterContext<'a> {
    context
        .with_declaration_starts_on_new_line(leading_token_delimiter(comments_before_token) == "\n")
}

pub(crate) fn declaration_delimiter(context: &FormatterContext<'_>) -> &'static str {
    if context.declaration_starts_on_new_line() {
        "\n"
    } else {
        ""
    }
}

pub(crate) fn semicolon_starts_on_new_line(
    options: &FmtOptions,
    comments_before_semicolon: &str,
) -> bool {
    options.prod_semicolon_on_nl || Line::ends_with_nl(comments_before_semicolon)
}

fn leading_token_delimiter(comments_before_token: &str) -> &'static str {
    if comments_before_token.is_empty() || !Line::ends_with_nl(comments_before_token) {
        "\n"
    } else {
        ""
    }
}
