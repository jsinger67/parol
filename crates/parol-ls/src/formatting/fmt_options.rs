use super::Line;
use lsp_types::{FormattingOptions, FormattingProperty};

macro_rules! add_boolean_formatting_option {
    ($self:ident, $options:ident, $option_name:ident, $default:literal) => {
        $self.$option_name = if let Some(&FormattingProperty::Bool(val)) = $options
            .properties
            .get(concat!("formatting.", stringify!($option_name)))
        {
            val
        } else {
            $default
        };
        eprintln!(
            concat!("FmtOptions: ", stringify!($option_name), ": {}"),
            $self.$option_name
        );
    };
}

macro_rules! add_number_formatting_option {
    ($self:ident, $options:ident, $option_name:ident, $default:literal) => {
        $self.$option_name = if let Some(&FormattingProperty::Number(val)) = $options
            .properties
            .get(concat!("formatting.", stringify!($option_name)))
        {
            val as usize
        } else {
            $default
        };
        eprintln!(
            concat!("FmtOptions: ", stringify!($option_name), ": {}"),
            $self.$option_name
        );
    };
}

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
        FmtOptions {
            empty_line_after_prod: true,
            prod_semicolon_on_nl: true,
            max_line_length: 100,
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
        let mut me = Self::new();
        add_boolean_formatting_option!(me, options, empty_line_after_prod, true);
        add_boolean_formatting_option!(me, options, prod_semicolon_on_nl, true);
        add_number_formatting_option!(me, options, max_line_length, 100);
        me
    }
}
