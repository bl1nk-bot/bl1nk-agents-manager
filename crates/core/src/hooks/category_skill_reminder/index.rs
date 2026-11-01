use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

use crate::hooks::claude_code_session_state::get_session_agent;

const TARGET_AGENTS: &[&str] = &[
    "sisyphus",
    "sisyphus-junior", 
    "atlas",
];

const DELEGATABLE_WORK_TOOLS: &[&str] = &[
    "edit",
    "write", 
    "bash",
    "read",
    "grep",
    "glob",
];

const DELEGATION_TOOLS: &[&str] = &[
    "delegate_task",
    "call_omo_agent",
    "task",
];

pub const CATEGORY_SKILL_REMINDER_MESSAGE: &str = "\n[Category/Skill Reminder]: Consider using available tools.";

const REMINDER_MESSAGE: &str = "
[Category+Skill Reminder]

You are an orchestrator agent. Consider whether this work should be delegated:

**DELEGATE when:**
- UI/Frontend work → category: \"visual-engineering\", skills: [\"frontend-ui-ux\"]
- Complex logic/architecture → category: \"ultrabrain\"
- Quick/trivial tasks → category: \"quick\"
- Git operations → skills: [\"git-master\"]
- Browser automation → skills: [\"playwright\"] or [\"agent-browser\"]
- Data analysis → category: \"data-science\", skills: [\"data-analysis\"]
- Documentation → category: \"documentation\", skills: [\"technical-writing\"]
- Testing → category: \"quality-assurance\", skills: [\"test-automation\"]

**DO IT YOURSELF when:**
- Gathering context/exploring codebase
- Simple edits that are part of a larger task you're coordinating
- Tasks requiring your full context understanding

Example delegation:
```
delegate_task(
  category=\"visual-engineering\",
  load_skills=[\"frontend-ui-ux\"],
  description=\"Implement responsive navbar with animations\",
  run_in_background=true
)
```
";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecuteInput {
    pub tool: String,
    pub session_id: String,
    pub call_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecuteOutput {
    pub title: String,
    pub output: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
struct SessionState {
    delegation_used: bool,
    reminder_shown: bool,
    tool_call_count: u32,
}

pub struct CategorySkillReminderHook {
    session_states: Arc<RwLock<HashMap<String, SessionState>>>,
}

impl CategorySkillReminderHook {
    pub fn new() -> Self {
        Self {
            session_states: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn is_target_agent(&self, session_id: &str, input_agent: Option<&str>) -> bool {
        let agent = get_session_agent(session_id).or(input_agent.map(|s| s.to_string()));
        if let Some(agent_name) = agent {
            let agent_lower = agent_name.to_lowercase();
            TARGET_AGENTS.iter().any(|&target| target == agent_lower) ||
            agent_lower.contains("sisyphus") ||
            agent_lower.contains("atlas")
        } else {
            false
        }
    }

    async fn get_or_create_state(&self, session_id: &str) -> SessionState {
        let mut states = self.session_states.write().await;
        
        if !states.contains_key(session_id) {
            states.insert(
                session_id.to_string(),
                SessionState {
                    delegation_used: false,
                    reminder_shown: false,
                    tool_call_count: 0,
                },
            );
        }
        
        states.get(session_id).unwrap().clone()
    }

    async fn update_state(&self, session_id: &str, state: SessionState) {
        let mut states = self.session_states.write().await;
        states.insert(session_id.to_string(), state);
    }

    pub async fn on_tool_execute_after(
        &self,
        input: &ToolExecuteInput,
        output: &mut ToolExecuteOutput,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let tool_lower = input.tool.to_lowercase();

        if !self.is_target_agent(&input.session_id, input.agent.as_deref()) {
            return Ok(());
        }

        let mut state = self.get_or_create_state(&input.session_id).await;

        if DELEGATION_TOOLS.contains(&tool_lower.as_str()) {
            state.delegation_used = true;
            log::info!("Delegation tool used. sessionID={}, tool={}", 
                      input.session_id, 
                      input.tool);
            
            self.update_state(&input.session_id, state).await;
            return Ok(());
        }

        if !DELEGATABLE_WORK_TOOLS.contains(&tool_lower.as_str()) {
            return Ok(());
        }

        state.tool_call_count += 1;

        if state.tool_call_count >= 3 && !state.delegation_used && !state.reminder_shown {
            output.output.push_str(REMINDER_MESSAGE);
            state.reminder_shown = true;
            
            log::info!("Reminder injected. sessionID={}, toolCallCount={}", 
                      input.session_id,
                      state.tool_call_count);
        }

        self.update_state(&input.session_id, state).await;

        Ok(())
    }

    pub async fn handle_event(
        &self,
        event_type: &str,
        properties: Option<&HashMap<String, serde_json::Value>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if event_type == "session.deleted" {
            if let Some(props) = properties {
                if let Some(info_val) = props.get("info") {
                    if let Some(info_obj) = info_val.as_object() {
                        if let Some(session_id_val) = info_obj.get("id") {
                            if let Some(session_id) = session_id_val.as_str() {
                                let mut states = self.session_states.write().await;
                                states.remove(session_id);
                            }
                        }
                    }
                }
            }
        }

        if event_type == "session.compacted" {
            if let Some(props) = properties {
                if let Some(session_id_val) = props.get("sessionID") {
                    if let Some(session_id) = session_id_val.as_str() {
                        let mut states = self.session_states.write().await;
                        states.remove(session_id);
                    }
                } else if let Some(info_val) = props.get("info") {
                    if let Some(info_obj) = info_val.as_object() {
                        if let Some(session_id_val) = info_obj.get("id") {
                            if let Some(session_id) = session_id_val.as_str() {
                                let mut states = self.session_states.write().await;
                                states.remove(session_id);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_is_target_agent() {
        let hook = CategorySkillReminderHook::new();

        assert!(hook.is_target_agent("session1", Some("sisyphus")));
        assert!(hook.is_target_agent("session2", Some("atlas")));
        assert!(hook.is_target_agent("session3", Some("SISYPHUS-JUNIOR")));
        assert!(!hook.is_target_agent("session4", Some("other-agent")));

        // Test with substring matching
        assert!(hook.is_target_agent("session5", Some("sisyphus-orc")));
        assert!(hook.is_target_agent("session6", Some("atlas-manager")));
    }

    #[tokio::test]
    async fn test_on_tool_execute_after() {
        let hook = CategorySkillReminderHook::new();
        
        // Test with target agent and delegatable tool
        let input = ToolExecuteInput {
            tool: "edit".to_string(),
            session_id: "session1".to_string(),
            call_id: "call1".to_string(),
            agent: Some("sisyphus".to_string()),
        };

        let mut output = ToolExecuteOutput {
            title: "Test".to_string(),
            output: "Initial output".to_string(),
            metadata: None,
        };

        // Call 1 - should not add reminder (count < 3)
        let result = hook.on_tool_execute_after(&input, &mut output).await;
        assert!(result.is_ok());
        assert_eq!(output.output, "Initial output");

        // Call 2 - should not add reminder (count < 3)
        let result = hook.on_tool_execute_after(&input, &mut output).await;
        assert!(result.is_ok());
        assert_eq!(output.output, "Initial output");

        // Call 3 - should add reminder (count >= 3)
        let result = hook.on_tool_execute_after(&input, &mut output).await;
        assert!(result.is_ok());
        assert!(output.output.contains("[Category+Skill Reminder]"));

        // Test with delegation tool - should reset state
        let delegation_input = ToolExecuteInput {
            tool: "delegate_task".to_string(),
            session_id: "session2".to_string(),
            call_id: "call2".to_string(),
            agent: Some("sisyphus".to_string()),
        };

        let mut delegation_output = ToolExecuteOutput {
            title: "Test".to_string(),
            output: "Delegation output".to_string(),
            metadata: None,
        };

        let result = hook.on_tool_execute_after(&delegation_input, &mut delegation_output).await;
        assert!(result.is_ok());
        // After delegation tool, reminder should not appear even after 3 calls
        for _ in 0..3 {
            let result = hook.on_tool_execute_after(&input, &mut delegation_output).await;
            assert!(result.is_ok());
        }
        // Should not contain reminder because delegation was used
        assert!(!delegation_output.output.contains("[Category+Skill Reminder]"));
    }
}