use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::hooks::shared::session_utils::is_caller_orchestrator;
use crate::hooks::shared::system_directive::SYSTEM_DIRECTIVE_PREFIX;
use crate::hooks::shared::logger::log;

pub const HOOK_NAME: &str = "sisyphus-junior-notepad";

pub const NOTEPAD_DIRECTIVE: &str = "
<Work_Context>
## Notepad Location (for recording learnings)
NOTEPAD PATH: .sisyphus/notepads/{plan-name}/
- learnings.md: Record patterns, conventions, successful approaches
- issues.md: Record problems, blockers, gotchas encountered
- decisions.md: Record architectural choices and rationales
- problems.md: Record unresolved issues, technical debt

You SHOULD append findings to notepad files after completing work.
IMPORTANT: Always APPEND to notepad files - never overwrite or use Edit tool.

## Plan Location (READ ONLY)
PLAN PATH: .sisyphus/plans/{plan-name}.md

CRITICAL RULE: NEVER MODIFY THE PLAN FILE

The plan file (.sisyphus/plans/*.md) is SACRED and READ-ONLY.
- You may READ the plan to understand tasks
- You may READ checkbox items to know what to do
- You MUST NOT edit, modify, or update the plan file
- You MUST NOT mark checkboxes as complete in the plan
- Only the Orchestrator manages the plan file

VIOLATION = IMMEDIATE FAILURE. The Orchestrator tracks plan state.
</Work_Context>
";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecuteInput {
    pub tool: String,
    pub session_id: String,
    pub call_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecuteOutput {
    pub args: HashMap<String, serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

pub struct SisyphusJuniorNotepadHook;

impl SisyphusJuniorNotepadHook {
    pub fn new() -> Self {
        Self
    }

    pub async fn on_tool_execute_before(
        &self,
        input: &ToolExecuteInput,
        output: &mut ToolExecuteOutput,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 1. Check if tool is delegate_task
        if input.tool != "delegate_task" {
            return Ok(());
        }

        // 2. Check if caller is Atlas (orchestrator)
        if !is_caller_orchestrator(&input.session_id).await {
            return Ok(());
        }

        // 3. Get prompt from output.args
        let prompt = if let Some(Value::String(prompt_str)) = output.args.get("prompt") {
            prompt_str.clone()
        } else {
            return Ok(());
        };

        // 4. Check for double injection
        if prompt.contains(SYSTEM_DIRECTIVE_PREFIX) {
            return Ok(());
        }

        // 5. Prepend directive
        let new_prompt = format!("{}{}", NOTEPAD_DIRECTIVE, prompt);
        output.args.insert("prompt".to_string(), Value::String(new_prompt));

        // 6. Log injection
        log::info!("[{}] Injected notepad directive to delegate_task. sessionID={}", HOOK_NAME, input.session_id);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_on_tool_execute_before() {
        let hook = SisyphusJuniorNotepadHook::new();
        let input = ToolExecuteInput {
            tool: "delegate_task".to_string(),
            session_id: "test_session".to_string(),
            call_id: "test_call".to_string(),
        };

        let mut output = ToolExecuteOutput {
            args: HashMap::new(),
            message: None,
        };
        output.args.insert("prompt".to_string(), Value::String("Original prompt".to_string()));

        // Mock the is_caller_orchestrator function to return true
        // In a real test, we would need to properly mock this function
        let result = hook.on_tool_execute_before(&input, &mut output).await;
        assert!(result.is_ok());

        // Verify that the directive was prepended if the session is an orchestrator
        if is_caller_orchestrator(&input.session_id).await {
            let updated_prompt = output.args.get("prompt").and_then(|v| v.as_str()).unwrap();
            assert!(updated_prompt.starts_with(NOTEPAD_DIRECTIVE.trim()));
        }
    }

    #[tokio::test]
    async fn test_on_tool_execute_before_non_delegated_task() {
        let hook = SisyphusJuniorNotepadHook::new();
        let input = ToolExecuteInput {
            tool: "other_tool".to_string(),
            session_id: "test_session".to_string(),
            call_id: "test_call".to_string(),
        };

        let mut output = ToolExecuteOutput {
            args: HashMap::new(),
            message: None,
        };
        output.args.insert("prompt".to_string(), Value::String("Original prompt".to_string()));

        let result = hook.on_tool_execute_before(&input, &mut output).await;
        assert!(result.is_ok());

        // Directive should not be prepended for non-delegate_task tools
        let original_prompt = output.args.get("prompt").and_then(|v| v.as_str()).unwrap();
        assert_eq!(original_prompt, "Original prompt");
    }

    #[test]
    fn test_constants() {
        assert_eq!(HOOK_NAME, "sisyphus-junior-notepad");
        assert!(NOTEPAD_DIRECTIVE.contains("Notepad Location"));
        assert!(NOTEPAD_DIRECTIVE.contains(".sisyphus/notepads"));
        assert!(NOTEPAD_DIRECTIVE.contains("learnings.md"));
        assert!(NOTEPAD_DIRECTIVE.contains("issues.md"));
        assert!(NOTEPAD_DIRECTIVE.contains("decisions.md"));
        assert!(NOTEPAD_DIRECTIVE.contains("problems.md"));
        assert!(NOTEPAD_DIRECTIVE.contains("Plan Location"));
        assert!(NOTEPAD_DIRECTIVE.contains("NEVER MODIFY THE PLAN FILE"));
    }
}