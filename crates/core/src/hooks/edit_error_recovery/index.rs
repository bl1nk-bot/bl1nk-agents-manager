use std::collections::HashMap;
use serde::{Deserialize, Serialize};

pub const EDIT_ERROR_PATTERNS: &[&str] = &[
    "oldString and newString must be different",
    "oldString not found",
    "oldString found multiple times",
];

pub const EDIT_ERROR_REMINDER: &str = "
[EDIT ERROR - IMMEDIATE ACTION REQUIRED]

You made an Edit mistake. STOP and do this NOW:

1. READ the file immediately to see its ACTUAL current state
2. VERIFY what the content really looks like (your assumption was wrong)
3. APOLOGIZE briefly to the user for the error
4. CONTINUE with corrected action based on the real file content

DO NOT attempt another edit until you've read and verified the file state.
";

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

pub struct EditErrorRecoveryHook;

impl EditErrorRecoveryHook {
    pub fn new() -> Self {
        Self
    }

    pub async fn on_tool_execute_after(
        &self,
        input: &ToolExecuteInput,
        output: &mut ToolExecuteOutput,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if input.tool.to_lowercase() != "edit" {
            return Ok(());
        }

        if has_edit_error(&output.output) {
            output.output.push_str(EDIT_ERROR_REMINDER);
        }

        Ok(())
    }
}

pub fn has_edit_error(output: &str) -> bool {
    let output_lower = output.to_lowercase();
    EDIT_ERROR_PATTERNS.iter()
        .any(|pattern| output_lower.contains(&pattern.to_lowercase()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_edit_error() {
        let output1 = "Error: oldString and newString must be different";
        assert!(has_edit_error(output1));

        let output2 = "Error: oldString not found";
        assert!(has_edit_error(output2));

        let output3 = "Error: oldString found multiple times";
        assert!(has_edit_error(output3));

        let output4 = "Success: File updated";
        assert!(!has_edit_error(output4));
    }

    #[tokio::test]
    async fn test_on_tool_execute_after() {
        let hook = EditErrorRecoveryHook::new();
        let input = ToolExecuteInput {
            tool: "edit".to_string(),
            session_id: "session123".to_string(),
            call_id: "call123".to_string(),
        };

        let mut output = ToolExecuteOutput {
            title: "Test".to_string(),
            output: "Error: oldString and newString must be different".to_string(),
            metadata: None,
        };

        let result = hook.on_tool_execute_after(&input, &mut output).await;
        assert!(result.is_ok());
        assert!(output.output.contains("[EDIT ERROR - IMMEDIATE ACTION REQUIRED]"));

        // Test with non-edit tool
        let input2 = ToolExecuteInput {
            tool: "other_tool".to_string(),
            session_id: "session123".to_string(),
            call_id: "call123".to_string(),
        };

        let mut output2 = ToolExecuteOutput {
            title: "Test".to_string(),
            output: "Error: oldString and newString must be different".to_string(),
            metadata: None,
        };

        let result2 = hook.on_tool_execute_after(&input2, &mut output2).await;
        assert!(result2.is_ok());
        assert!(!output2.output.contains("[EDIT ERROR - IMMEDIATE ACTION REQUIRED]"));

        // Test with success output
        let input3 = ToolExecuteInput {
            tool: "edit".to_string(),
            session_id: "session123".to_string(),
            call_id: "call123".to_string(),
        };

        let mut output3 = ToolExecuteOutput {
            title: "Test".to_string(),
            output: "Success: File updated".to_string(),
            metadata: None,
        };

        let result3 = hook.on_tool_execute_after(&input3, &mut output3).await;
        assert!(result3.is_ok());
        assert!(!output3.output.contains("[EDIT ERROR - IMMEDIATE ACTION REQUIRED]"));
    }
}