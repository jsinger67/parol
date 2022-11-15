use std::collections::HashMap;

use lsp_types::{DynamicRegistrationClientCapabilities, InitializeParams};
use serde::Deserialize;
use serde_json::Value;

use crate::arguments::Arguments;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct ConfigProperties(HashMap<String, Value>);

#[derive(Debug)]
pub(crate) struct Config {
    initialization_params: InitializeParams,
    args: Arguments,
    config_properties: ConfigProperties,
}

impl Config {
    pub(crate) fn new(initialization_params: InitializeParams, args: Arguments) -> Config {
        let config_properties: ConfigProperties = serde_json::from_value(
            initialization_params
                .initialization_options
                .clone()
                .unwrap_or_default(),
        )
        .unwrap_or_default();
        Self {
            initialization_params,
            args,
            config_properties,
        }
    }

    pub(crate) fn lookahead(&self) -> usize {
        self.args.lookahead
    }

    pub(crate) fn initialization_params(&self) -> &InitializeParams {
        &self.initialization_params
    }

    pub(crate) fn initialization_options(&self) -> Option<&Value> {
        self.initialization_params.initialization_options.as_ref()
    }

    pub(crate) fn supports_dynamic_registration_for_change_config(&self) -> bool {
        if let Some(workspace) = self.initialization_params.capabilities.workspace.as_ref() {
            matches!(
                workspace.did_change_configuration,
                Some(DynamicRegistrationClientCapabilities {
                    dynamic_registration: Some(true)
                })
            )
        } else {
            false
        }
    }
}
