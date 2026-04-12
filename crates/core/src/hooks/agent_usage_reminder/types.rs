use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentUsageState {
    pub session_id: String,
    pub agent_used: bool,
    pub reminder_count: u32,
    pub updated_at: i64,
}
