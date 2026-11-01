use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ContextWindowMonitorHook {
    reminded_sessions: Arc<RwLock<HashSet<String>>>,
}

impl ContextWindowMonitorHook {
    pub fn new() -> Self {
        Self {
            reminded_sessions: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    pub async fn on_tool_execute_after(
        &self,
        session_id: &str,
        provider_id: &str,
        input_tokens: usize,
        output: &mut String,
    ) {
        if self.reminded_sessions.read().await.contains(session_id) {
            return;
        }

        if provider_id != "anthropic" { return; }

        let actual_limit = 1_000_000;
        let warning_threshold = 0.70;
        let usage = input_tokens as f32 / actual_limit as f32;

        if usage >= warning_threshold {
            output.push_str("\n\n[Context Warning] High context window usage detected.");
            self.reminded_sessions.write().await.insert(session_id.to_string());
        }
    }
}

