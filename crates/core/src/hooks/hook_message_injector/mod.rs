use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub provider_id: String,
    pub model_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathInfo {
    pub cwd: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookMessageOptions {
    pub agent: Option<String>,
    pub model: Option<ModelInfo>,
    pub path: Option<PathInfo>,
}

pub struct HookMessageInjectorHook;

impl HookMessageInjectorHook {
    pub fn new() -> Self { Self }
}

pub async fn inject_hook_message(
    session_id: &str,
    message: &str,
    options: &HookMessageOptions,
) -> bool {
    // Stub implementation
    true
}

pub fn find_nearest_message_with_fields(dir: &str) -> Option<HookMessage> { None }
pub fn find_first_message_with_agent(dir: &str) -> Option<String> { None }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookMessage {
    pub agent: Option<String>,
}

pub const MESSAGE_STORAGE_DIR: &str = ".sisyphus/messages";
