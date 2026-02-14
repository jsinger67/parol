use lsp_types::{FormattingOptions, FormattingProperty};

use crate::config::ConfigProperties;

pub(crate) const EMPTY_LINE_AFTER_PROD_KEY: &str = "formatting.empty_line_after_prod";
pub(crate) const PROD_SEMICOLON_ON_NL_KEY: &str = "formatting.prod_semicolon_on_nl";
pub(crate) const MAX_LINE_LENGTH_KEY: &str = "formatting.max_line_length";

#[derive(Debug, Clone)]
pub(crate) struct FormattingSettings {
    pub(crate) empty_line_after_prod: bool,
    pub(crate) prod_semicolon_on_nl: bool,
    pub(crate) max_line_length: usize,
}

impl Default for FormattingSettings {
    fn default() -> Self {
        Self {
            empty_line_after_prod: true,
            prod_semicolon_on_nl: true,
            max_line_length: 100,
        }
    }
}

impl FormattingSettings {
    pub(crate) fn update_from_config_properties(
        &mut self,
        props: &ConfigProperties,
    ) -> Result<(), serde_json::error::Error> {
        self.empty_line_after_prod = read_config_bool(
            props,
            EMPTY_LINE_AFTER_PROD_KEY,
            self.empty_line_after_prod,
        )?;
        self.prod_semicolon_on_nl =
            read_config_bool(props, PROD_SEMICOLON_ON_NL_KEY, self.prod_semicolon_on_nl)?;
        self.max_line_length =
            read_config_usize(props, MAX_LINE_LENGTH_KEY, self.max_line_length)?;

        eprintln!("{EMPTY_LINE_AFTER_PROD_KEY}: {}", self.empty_line_after_prod);
        eprintln!("{PROD_SEMICOLON_ON_NL_KEY}: {}", self.prod_semicolon_on_nl);
        eprintln!("{MAX_LINE_LENGTH_KEY}: {}", self.max_line_length);

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