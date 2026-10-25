use super::{FormattingSettings, Line};
use lsp_types::FormattingOptions;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum Padding {
    #[default]
    None,
    Left,
    Right,
    Both,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum LineEnd {
    #[default]
    Unchanged,
    ForceAdd,
    ForceSingleNewline,
    ForceRemove,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum Trimming {
    #[default]
    Unchanged,
    TrimLeft,
    TrimRight,
    Trim,
}

/// Indentation style for formatting
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndentStyle {
    /// Use spaces (default)
    Spaces(u8),
    /// Use tabs
    Tabs,
}

impl Default for IndentStyle {
    fn default() -> Self {
        IndentStyle::Spaces(4)
    }
}

#[allow(unused)]
#[derive(Debug, Clone, Default)]
pub(crate) struct FmtOptions {
    pub(crate) padding: Padding,
    pub(crate) line_end: LineEnd,
    pub(crate) trimming: Trimming,
    pub(crate) nesting_depth: u16,

    /// Add an empty line after each production
    pub(crate) empty_line_after_prod: bool,

    /// Place the semicolon after each production on a new line
    pub(crate) prod_semicolon_on_nl: bool,

    /// Maximum number of characters per line
    pub(crate) max_line_length: usize,

    /// Indentation style (spaces count or tabs)
    pub(crate) indent_style: IndentStyle,

    /// Add a space after ':' in productions
    pub(crate) space_after_colon: bool,

    /// Add a space before ';' in productions
    pub(crate) space_before_semicolon: bool,

    /// Add a space around '|' in alternations
    pub(crate) space_around_pipe: bool,

    /// Add a space around '=' in declarations
    pub(crate) space_around_equals: bool,

    /// Place opening brace on new line for scanner directives
    pub(crate) scanner_brace_newline: bool,

    /// Place semicolons at end of line when possible
    pub(crate) semicolon_end_of_line: bool,

    /// Place top-level alternatives on new line each
    pub(crate) place_top_level_alternatives_on_new_line_each: bool,
    /// Keep leading line comments on their own line
    pub(crate) leading_line_comments_on_own_line: bool,
}

#[allow(unused)]
impl FmtOptions {
    pub(crate) fn new() -> Self {
        let defaults = FormattingSettings::default();
        FmtOptions {
            empty_line_after_prod: defaults.empty_line_after_prod,
            prod_semicolon_on_nl: defaults.prod_semicolon_on_nl,
            max_line_length: defaults.max_line_length,
            indent_style: defaults.indent_style,
            space_after_colon: defaults.space_after_colon,
            space_before_semicolon: defaults.space_before_semicolon,
            space_around_pipe: defaults.space_around_pipe,
            space_around_equals: defaults.space_around_equals,
            scanner_brace_newline: defaults.scanner_brace_newline,
            semicolon_end_of_line: defaults.semicolon_end_of_line,
            place_top_level_alternatives_on_new_line_each: defaults.place_top_level_alternatives_on_new_line_each,
            leading_line_comments_on_own_line: false,
            ..Default::default()
        }
    }
    pub(crate) fn with_padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self
    }
    pub(crate) fn with_line_end(mut self, line_end: LineEnd) -> Self {
        self.line_end = line_end;
        self
    }
    pub(crate) fn with_trimming(mut self, trimming: Trimming) -> Self {
        self.trimming = trimming;
        self
    }
    pub(crate) fn next_depth(mut self) -> Self {
        self.nesting_depth += 1;
        self
    }

    pub(crate) fn apply_formatting(&self, line: String) -> String {
        self.apply_line_end_policy(line, self.line_end)
    }

    /// Applies the trimming policy, the given line-end policy and the padding
    /// policy to a line of text.
    pub(crate) fn apply_line_end_policy(&self, line: String, line_end: LineEnd) -> String {
        let line = match self.trimming {
            Trimming::Unchanged => line,
            Trimming::TrimLeft => line.trim_start().to_string(),
            Trimming::TrimRight => line.trim_end().to_string(),
            Trimming::Trim => line.trim().to_string(),
        };
        let line = match line_end {
            LineEnd::Unchanged => line,
            LineEnd::ForceAdd => {
                if line.is_empty() {
                    line
                } else {
                    line + "\n"
                }
            }
            LineEnd::ForceRemove => line.trim_end_matches(['\r', '\n']).to_string(),
            LineEnd::ForceSingleNewline => {
                let mut trimmed = line.trim_matches(|c| c == '\r' || c == '\n').to_string();
                trimmed.push('\n');
                trimmed
            }
        };
        if !line.is_empty() {
            match self.padding {
                Padding::None => line,
                Padding::Left => format!(" {line}"),
                Padding::Right => {
                    // Don't add a space character if the line already ends with a newline
                    // TODO: Maybe we should expand this to all whitespace characters
                    if Line::ends_with_nl(&line) {
                        line
                    } else {
                        format!("{line} ")
                    }
                }
                Padding::Both => format!(" {line} "),
            }
        } else {
            line
        }
    }
}

/// Dedicated policy that controls how comments are placed onto lines
/// during formatting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct CommentLinePolicy {
    /// Keep leading line comments on their own line.
    pub(crate) own_line: bool,
}

impl CommentLinePolicy {
    /// Creates the comment line policy from the formatting options.
    pub(crate) fn from_options(options: &FmtOptions) -> Self {
        Self {
            own_line: options.leading_line_comments_on_own_line,
        }
    }

    /// Resolves the line-end policy that applies to comment text.
    pub(crate) fn resolve_line_end(&self, inherited: LineEnd, is_line_comment: bool) -> LineEnd {
        if is_line_comment && self.own_line && inherited != LineEnd::ForceRemove {
            LineEnd::ForceSingleNewline
        } else {
            inherited
        }
    }
}

impl From<&FormattingOptions> for FmtOptions {
    fn from(options: &FormattingOptions) -> Self {
        let settings = FormattingSettings::from_options(options);
        Self {
            empty_line_after_prod: settings.empty_line_after_prod,
            prod_semicolon_on_nl: settings.prod_semicolon_on_nl,
            max_line_length: settings.max_line_length,
            indent_style: settings.indent_style,
            space_after_colon: settings.space_after_colon,
            space_before_semicolon: settings.space_before_semicolon,
            space_around_pipe: settings.space_around_pipe,
            space_around_equals: settings.space_around_equals,
            scanner_brace_newline: settings.scanner_brace_newline,
            semicolon_end_of_line: settings.semicolon_end_of_line,
            place_top_level_alternatives_on_new_line_each: settings.place_top_level_alternatives_on_new_line_each,
            leading_line_comments_on_own_line: settings.leading_line_comments_on_own_line,
            ..Self::default()
        }
    }
}
