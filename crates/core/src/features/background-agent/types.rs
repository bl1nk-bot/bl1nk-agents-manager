use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BackgroundTaskConfig {
    pub stale_timeout_ms: Option<u64>,
    pub max_concurrency_per_key: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TmuxConfig {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRef {
    #[serde(rename = "providerID")]
    pub provider_id: String,
    #[serde(rename = "modelID")]
    pub model_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundTaskProgress {
    pub tool_calls: u32,
    pub last_update: SystemTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_tool: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_message_at: Option<SystemTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BackgroundTaskStatus {
    Pending,
    Running,
    Completed,
    Cancelled,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundTask {
    pub id: String,
    pub status: BackgroundTaskStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queued_at: Option<SystemTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<SystemTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<SystemTime>,
    pub description: String,
    pub prompt: String,
    pub agent: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_message_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_model: Option<ModelRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_unstable_agent: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<ModelRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<BackgroundTaskProgress>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub concurrency_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub concurrency_group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_msg_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stable_polls: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchInput {
    pub agent: String,
    pub model: Option<ModelRef>,
    pub description: String,
    pub parent_session_id: String,
    pub parent_message_id: Option<String>,
    pub parent_model: Option<ModelRef>,
    pub parent_agent: Option<String>,
    pub prompt: String,
    pub category: Option<String>,
    pub is_unstable_agent: Option<bool>,
    pub skills: Option<Vec<String>>,
    pub skill_content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumeInput {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    pub prompt: String,
    pub parent_session_id: String,
    pub parent_message_id: Option<String>,
    pub parent_model: Option<ModelRef>,
    pub parent_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub content: String,
    pub status: String,
    pub priority: String,
    pub id: String,
}
