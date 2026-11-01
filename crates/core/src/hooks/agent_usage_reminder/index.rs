use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::Utc;
use crate::hooks::agent_usage_reminder::types::AgentUsageState;
use crate::hooks::agent_usage_reminder::storage::{load_agent_usage_state, save_agent_usage_state, clear_agent_usage_state};
use crate::hooks::agent_usage_reminder::constants::{target_tools, agent_tools, REMINDER_MESSAGE};

pub struct AgentUsageReminderHook {
    session_states: Arc<RwLock<HashMap<String, AgentUsageState>>>,
}

impl AgentUsageReminderHook {
    pub fn new() -> Self {
        Self {
            session_states: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn get_or_create_state(&self, session_id: &str) -> AgentUsageState {
        let mut states = self.session_states.write().await;
        if let Some(state) = states.get(session_id) {
            return state.clone();
        }

        let state = load_agent_usage_state(session_id).unwrap_or_else(|| AgentUsageState {
            session_id: session_id.to_string(),
            agent_used: false,
            reminder_count: 0,
            updated_at: Utc::now().timestamp_millis(),
        });

        states.insert(session_id.to_string(), state.clone());
        state
    }

    pub async fn mark_agent_used(&self, session_id: &str) -> anyhow::Result<()> {
        let mut state = self.get_or_create_state(session_id).await;
        state.agent_used = true;
        state.updated_at = Utc::now().timestamp_millis();
        
        save_agent_usage_state(&state)?;
        self.session_states.write().await.insert(session_id.to_string(), state);
        Ok(())
    }

    pub async fn reset_state(&self, session_id: &str) {
        self.session_states.write().await.remove(session_id);
        clear_agent_usage_state(session_id);
    }

    pub async fn on_tool_execute_after(
        &self, 
        tool: &str, 
        session_id: &str, 
        output: &mut String
    ) -> anyhow::Result<()> {
        let tool_lower = tool.to_lowercase();

        if agent_tools().contains(tool_lower.as_str()) {
            self.mark_agent_used(session_id).await?;
            return Ok(());
        }

        if !target_tools().contains(tool_lower.as_str()) {
            return Ok(());
        }

        let mut state = self.get_or_create_state(session_id).await;
        if state.agent_used {
            return Ok(());
        }

        // Append reminder message
        output.push_str(REMINDER_MESSAGE);
        
        state.reminder_count += 1;
        state.updated_at = Utc::now().timestamp_millis();
        
        save_agent_usage_state(&state)?;
        self.session_states.write().await.insert(session_id.to_string(), state);
        
        Ok(())
    }
}
