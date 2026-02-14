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

#[derive(Debug, Default, Clone, PartialEq, Eq)]
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
}

#[allow(unused)]
impl FmtOptions {
    pub(crate) fn new() -> Self {
        let defaults = FormattingSettings::default();
        FmtOptions {
            empty_line_after_prod: defaults.empty_line_after_prod,
            prod_semicolon_on_nl: defaults.prod_semicolon_on_nl,
            max_line_length: defaults.max_line_length,
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
        let line = match self.trimming {
            Trimming::Unchanged => line,
            Trimming::TrimLeft => line.trim_start().to_string(),
            Trimming::TrimRight => line.trim_end().to_string(),
            Trimming::Trim => line.trim().to_string(),
        };
        let line = match self.line_end {
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

impl From<&FormattingOptions> for FmtOptions {
    fn from(options: &FormattingOptions) -> Self {
        let settings = FormattingSettings::from_options(options);
        Self {
            empty_line_after_prod: settings.empty_line_after_prod,
            prod_semicolon_on_nl: settings.prod_semicolon_on_nl,
            max_line_length: settings.max_line_length,
            ..Self::default()
        }
    }
}
