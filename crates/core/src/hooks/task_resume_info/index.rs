use std::collections::HashMap;
use regex::Regex;
use serde::{Deserialize, Serialize};

const TARGET_TOOLS: &[&str] = &["task", "Task", "call_omo_agent", "delegate_task"];

const SESSION_ID_PATTERNS: &[&str] = &[
    r"Session ID: (ses_[a-zA-Z0-9_-]+)",
    r"session_id: (ses_[a-zA-Z0-9_-]+)",
    r"(?s)<task_metadata>\s*session_id: (ses_[a-zA-Z0-9_-]+)",
    r"sessionId: (ses_[a-zA-Z0-9_-]+)",
];

fn extract_session_id(output: &str) -> Option<String> {
    for pattern_str in SESSION_ID_PATTERNS {
        if let Ok(re) = Regex::new(pattern_str) {
            if let Some(caps) = re.captures(output) {
                if let Some(session_id_match) = caps.get(1) {
                    return Some(session_id_match.as_str().to_string());
                }
            }
        }
    }
    None
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecuteInput {
    pub tool: String,
    pub session_id: String,
    pub call_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecuteOutput {
    pub title: String,
    pub output: String,
    pub metadata: Option<serde_json::Value>,
}

pub struct TaskResumeInfoHook;

impl TaskResumeInfoHook {
    pub fn new() -> Self {
        Self
    }

    pub async fn on_tool_execute_after(
        &self,
        input: &ToolExecuteInput,
        output: &mut ToolExecuteOutput,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if !TARGET_TOOLS.contains(&input.tool.as_str()) {
            return Ok(());
        }

        if output.output.starts_with("Error:") || 
           output.output.starts_with("Failed") || 
           output.output.contains("\nto continue:") {
            return Ok(());
        }

        if let Some(session_id) = extract_session_id(&output.output) {
            output.output = format!("{}\n\nto continue: delegate_task(session_id=\"{}\", prompt=\"...\")", 
                                  output.output.trim_end(), 
                                  session_id);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_session_id() {
        let output1 = "Session ID: ses_abc123";
        assert_eq!(extract_session_id(output1), Some("ses_abc123".to_string()));

        let output2 = "session_id: ses_def456";
        assert_eq!(extract_session_id(output2), Some("ses_def456".to_string()));

        let output3 = "<task_metadata>\nsession_id: ses_ghi789\n</task_metadata>";
        assert_eq!(extract_session_id(output3), Some("ses_ghi789".to_string()));

        let output4 = "sessionId: ses_jkl012";
        assert_eq!(extract_session_id(output4), Some("ses_jkl012".to_string()));

        let output5 = "No session ID here";
        assert_eq!(extract_session_id(output5), None);
    }

    #[tokio::test]
    async fn test_on_tool_execute_after() {
        let hook = TaskResumeInfoHook::new();
        let input = ToolExecuteInput {
            tool: "task".to_string(),
            session_id: "session123".to_string(),
            call_id: "call123".to_string(),
        };

        let mut output = ToolExecuteOutput {
            title: "Test".to_string(),
            output: "Session ID: ses_test123\nSome output".to_string(),
            metadata: None,
        };

        let result = hook.on_tool_execute_after(&input, &mut output).await;
        assert!(result.is_ok());
        assert!(output.output.contains("to continue: delegate_task(session_id=\"ses_test123\""));

        // Test with error output
        let mut error_output = ToolExecuteOutput {
            title: "Test".to_string(),
            output: "Error: Something went wrong".to_string(),
            metadata: None,
        };

        let result = hook.on_tool_execute_after(&input, &mut error_output).await;
        assert!(result.is_ok());
        assert!(!error_output.output.contains("to continue:"));
    }
}