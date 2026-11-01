use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RalphLoopState {
    pub active: bool,
    pub iteration: u32,
    pub max_iterations: u32,
    pub completion_promise: String,
    pub started_at: String,
    pub prompt: String,
    pub session_id: Option<String>,
    pub ultrawork: Option<bool>,
}
