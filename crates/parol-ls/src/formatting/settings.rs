use lsp_types::{FormattingOptions, FormattingProperty};

use crate::config::ConfigProperties;
use super::fmt_options::IndentStyle;

pub(crate) const EMPTY_LINE_AFTER_PROD_KEY: &str = "formatting.empty_line_after_prod";
pub(crate) const PROD_SEMICOLON_ON_NL_KEY: &str = "formatting.prod_semicolon_on_nl";
pub(crate) const MAX_LINE_LENGTH_KEY: &str = "formatting.max_line_length";
pub(crate) const INDENT_STYLE_KEY: &str = "formatting.indent_style";
pub(crate) const SPACE_AFTER_COLON_KEY: &str = "formatting.space_after_colon";
pub(crate) const SPACE_BEFORE_SEMICOLON_KEY: &str = "formatting.space_before_semicolon";
pub(crate) const SPACE_AROUND_PIPE_KEY: &str = "formatting.space_around_pipe";
pub(crate) const SPACE_AROUND_EQUALS_KEY: &str = "formatting.space_around_equals";
pub(crate) const SCANNER_BRACE_NEWLINE_KEY: &str = "formatting.scanner_brace_newline";
pub(crate) const SEMICOLON_END_OF_LINE_KEY: &str = "formatting.semicolon_end_of_line";

#[derive(Debug, Clone)]
pub(crate) struct FormattingSettings {
    pub(crate) empty_line_after_prod: bool,
    pub(crate) prod_semicolon_on_nl: bool,
    pub(crate) max_line_length: usize,
    pub(crate) indent_style: IndentStyle,
    pub(crate) space_after_colon: bool,
    pub(crate) space_before_semicolon: bool,
    pub(crate) space_around_pipe: bool,
    pub(crate) space_around_equals: bool,
    pub(crate) scanner_brace_newline: bool,
    pub(crate) semicolon_end_of_line: bool,
}

impl Default for FormattingSettings {
    fn default() -> Self {
        Self {
            empty_line_after_prod: true,
            prod_semicolon_on_nl: true,
            max_line_length: 100,
            indent_style: IndentStyle::Spaces(4),
            space_after_colon: true,
            space_before_semicolon: false,
            space_around_pipe: true,
            space_around_equals: true,
            scanner_brace_newline: false,
            semicolon_end_of_line: false,
        }
    }
}

impl FormattingSettings {
    pub(crate) fn update_from_config_properties(
        &mut self,
        props: &ConfigProperties,
    ) -> Result<(), serde_json::error::Error> {
        self.empty_line_after_prod =
            read_config_bool(props, EMPTY_LINE_AFTER_PROD_KEY, self.empty_line_after_prod)?;
        self.prod_semicolon_on_nl =
            read_config_bool(props, PROD_SEMICOLON_ON_NL_KEY, self.prod_semicolon_on_nl)?;
        self.max_line_length = read_config_usize(props, MAX_LINE_LENGTH_KEY, self.max_line_length)?;
        self.space_after_colon =
            read_config_bool(props, SPACE_AFTER_COLON_KEY, self.space_after_colon)?;
        self.space_before_semicolon =
            read_config_bool(props, SPACE_BEFORE_SEMICOLON_KEY, self.space_before_semicolon)?;
        self.space_around_pipe =
            read_config_bool(props, SPACE_AROUND_PIPE_KEY, self.space_around_pipe)?;
        self.space_around_equals =
            read_config_bool(props, SPACE_AROUND_EQUALS_KEY, self.space_around_equals)?;
        self.scanner_brace_newline =
            read_config_bool(props, SCANNER_BRACE_NEWLINE_KEY, self.scanner_brace_newline)?;
        self.semicolon_end_of_line =
            read_config_bool(props, SEMICOLON_END_OF_LINE_KEY, self.semicolon_end_of_line)?;

        // Handle indent_style specially since it's an enum
        if let Some(value) = props.0.get(INDENT_STYLE_KEY) {
            let style_str: String = serde_json::from_value(value.clone())?;
            self.indent_style = match style_str.as_str() {
                "tabs" => IndentStyle::Tabs,
                _ => {
                    // Try to parse as number of spaces
                    if let Ok(spaces) = style_str.parse::<u8>() {
                        IndentStyle::Spaces(spaces)
                    } else {
                        self.indent_style.clone()
                    }
                }
            };
        }

        eprintln!("{EMPTY_LINE_AFTER_PROD_KEY}: {}", self.empty_line_after_prod);
        eprintln!("{PROD_SEMICOLON_ON_NL_KEY}: {}", self.prod_semicolon_on_nl);
        eprintln!("{MAX_LINE_LENGTH_KEY}: {}", self.max_line_length);
        eprintln!("{SPACE_AFTER_COLON_KEY}: {}", self.space_after_colon);
        eprintln!("{SPACE_AROUND_PIPE_KEY}: {}", self.space_around_pipe);

        Ok(())
    }

    pub(crate) fn add_to_options(&self, options: &mut FormattingOptions) {
        options.properties.insert(
            EMPTY_LINE_AFTER_PROD_KEY.to_owned(),
            FormattingProperty::Bool(self.empty_line_after_prod),
        );
        options.properties.insert(
            PROD_SEMICOLON_ON_NL_KEY.to_owned(),
            FormattingProperty::Bool(self.prod_semicolon_on_nl),
        );
        options.properties.insert(
            MAX_LINE_LENGTH_KEY.to_owned(),
            FormattingProperty::Number(self.max_line_length as i32),
        );
        options.properties.insert(
            SPACE_AFTER_COLON_KEY.to_owned(),
            FormattingProperty::Bool(self.space_after_colon),
        );
        options.properties.insert(
            SPACE_BEFORE_SEMICOLON_KEY.to_owned(),
            FormattingProperty::Bool(self.space_before_semicolon),
        );
        options.properties.insert(
            SPACE_AROUND_PIPE_KEY.to_owned(),
            FormattingProperty::Bool(self.space_around_pipe),
        );
        options.properties.insert(
            SPACE_AROUND_EQUALS_KEY.to_owned(),
            FormattingProperty::Bool(self.space_around_equals),
        );
        options.properties.insert(
            SCANNER_BRACE_NEWLINE_KEY.to_owned(),
            FormattingProperty::Bool(self.scanner_brace_newline),
        );
        options.properties.insert(
            SEMICOLON_END_OF_LINE_KEY.to_owned(),
            FormattingProperty::Bool(self.semicolon_end_of_line),
        );

        // Add indent_style as a string
        let indent_style_str = match &self.indent_style {
            IndentStyle::Spaces(n) => n.to_string(),
            IndentStyle::Tabs => "tabs".to_string(),
        };
        options.properties.insert(
            INDENT_STYLE_KEY.to_owned(),
            FormattingProperty::String(indent_style_str),
        );
    }

    pub(crate) fn from_options(options: &FormattingOptions) -> Self {
        let defaults = Self::default();
        Self {
            empty_line_after_prod: read_option_bool(
                options,
                EMPTY_LINE_AFTER_PROD_KEY,
                defaults.empty_line_after_prod,
            ),
            prod_semicolon_on_nl: read_option_bool(
                options,
                PROD_SEMICOLON_ON_NL_KEY,
                defaults.prod_semicolon_on_nl,
            ),
            max_line_length: read_option_usize(
                options,
                MAX_LINE_LENGTH_KEY,
                defaults.max_line_length,
            ),
            indent_style: read_option_indent_style(options, INDENT_STYLE_KEY, &defaults.indent_style),
            space_after_colon: read_option_bool(
                options,
                SPACE_AFTER_COLON_KEY,
                defaults.space_after_colon,
            ),
            space_before_semicolon: read_option_bool(
                options,
                SPACE_BEFORE_SEMICOLON_KEY,
                defaults.space_before_semicolon,
            ),
            space_around_pipe: read_option_bool(
                options,
                SPACE_AROUND_PIPE_KEY,
                defaults.space_around_pipe,
            ),
            space_around_equals: read_option_bool(
                options,
                SPACE_AROUND_EQUALS_KEY,
                defaults.space_around_equals,
            ),
            scanner_brace_newline: read_option_bool(
                options,
                SCANNER_BRACE_NEWLINE_KEY,
                defaults.scanner_brace_newline,
            ),
            semicolon_end_of_line: read_option_bool(
                options,
                SEMICOLON_END_OF_LINE_KEY,
                defaults.semicolon_end_of_line,
            ),
        }
    }
}

fn read_config_bool(
    props: &ConfigProperties,
    key: &str,
    default: bool,
) -> Result<bool, serde_json::error::Error> {
    if let Some(value) = props.0.get(key) {
        serde_json::from_value(value.clone())
    } else {
        Ok(default)
    }
}

fn read_config_usize(
    props: &ConfigProperties,
    key: &str,
    default: usize,
) -> Result<usize, serde_json::error::Error> {
    if let Some(value) = props.0.get(key) {
        serde_json::from_value(value.clone())
    } else {
        Ok(default)
    }
}

fn read_option_bool(options: &FormattingOptions, key: &str, default: bool) -> bool {
    if let Some(FormattingProperty::Bool(value)) = options.properties.get(key) {
        *value
    } else {
        default
    }
}

fn read_option_usize(options: &FormattingOptions, key: &str, default: usize) -> usize {
    if let Some(FormattingProperty::Number(value)) = options.properties.get(key) {
        *value as usize
    } else {
        default
    }
}

fn read_option_indent_style(
    options: &FormattingOptions,
    key: &str,
    default: &IndentStyle,
) -> IndentStyle {
    if let Some(FormattingProperty::String(value)) = options.properties.get(key) {
        match value.as_str() {
            "tabs" => IndentStyle::Tabs,
            _ => {
                if let Ok(spaces) = value.parse::<u8>() {
                    IndentStyle::Spaces(spaces)
                } else {
                    default.clone()
                }
            }
        }
    } else {
        default.clone()
    }
}
