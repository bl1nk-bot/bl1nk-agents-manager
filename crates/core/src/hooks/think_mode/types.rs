use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThinkModeState {
    pub requested: bool,
    pub model_switched: bool,
    pub thinking_config_injected: bool,
    pub provider_id: Option<String>,
    pub model_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRef {
    pub provider_id: String,
    pub model_id: String,
}
