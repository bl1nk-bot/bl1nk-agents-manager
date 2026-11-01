use std::collections::HashMap;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegateTaskErrorPattern {
    pub pattern: String,
    pub error_type: String,
    pub fix_hint: String,
}

pub const DELEGATE_TASK_ERROR_PATTERNS: &[DelegateTaskErrorPattern] = &[
    DelegateTaskErrorPattern {
        pattern: "run_in_background".to_string(),
        error_type: "missing_run_in_background".to_string(),
        fix_hint: "Add run_in_background=false (for delegation) or run_in_background=true (for parallel exploration)".to_string(),
    },
    DelegateTaskErrorPattern {
        pattern: "load_skills".to_string(),
        error_type: "missing_load_skills".to_string(),
        fix_hint: "Add load_skills=[] parameter (empty array if no skills needed). Note: Calling Skill tool does NOT populate this.".to_string(),
    },
    DelegateTaskErrorPattern {
        pattern: "category OR subagent_type".to_string(),
        error_type: "mutual_exclusion".to_string(),
        fix_hint: "Provide ONLY one of: category (e.g., 'general', 'quick') OR subagent_type (e.g., 'oracle', 'explore')".to_string(),
    },
    DelegateTaskErrorPattern {
        pattern: "Must provide either category or subagent_type".to_string(),
        error_type: "missing_category_or_agent".to_string(),
        fix_hint: "Add either category='general' OR subagent_type='explore'".to_string(),
    },
    DelegateTaskErrorPattern {
        pattern: "Unknown category".to_string(),
        error_type: "unknown_category".to_string(),
        fix_hint: "Use a valid category from the Available list in the error message".to_string(),
    },
    DelegateTaskErrorPattern {
        pattern: "Agent name cannot be empty".to_string(),
        error_type: "empty_agent".to_string(),
        fix_hint: "Provide a non-empty subagent_type value".to_string(),
    },
    DelegateTaskErrorPattern {
        pattern: "Unknown agent".to_string(),
        error_type: "unknown_agent".to_string(),
        fix_hint: "Use a valid agent from the Available agents list in the error message".to_string(),
    },
    DelegateTaskErrorPattern {
        pattern: "Cannot call primary agent".to_string(),
        error_type: "primary_agent".to_string(),
        fix_hint: "Primary agents cannot be called via delegate_task. Use a subagent like 'explore', 'oracle', or 'librarian'".to_string(),
    },
    DelegateTaskErrorPattern {
        pattern: "Skills not found".to_string(),
        error_type: "unknown_skills".to_string(),
        fix_hint: "Use valid skill names from the Available list in the error message".to_string(),
    },
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedError {
    pub error_type: String,
    pub original_output: String,
}

pub fn detect_delegate_task_error(output: &str) -> Option<DetectedError> {
    if !output.contains("[ERROR]") && !output.contains("Invalid arguments") {
        return None;
    }

    for error_pattern in DELEGATE_TASK_ERROR_PATTERNS {
        if output.contains(&error_pattern.pattern) {
            return Some(DetectedError {
                error_type: error_pattern.error_type.clone(),
                original_output: output.to_string(),
            });
        }
    }

    None
}

fn extract_available_list(output: &str) -> Option<String> {
    let re = Regex::new(r"(?m)Available[^:]*:\s*(.+)$").unwrap();
    if let Some(caps) = re.captures(output) {
        if let Some(matched) = caps.get(1) {
            return Some(matched.as_str().trim().to_string());
        }
    }
    None
}

pub fn build_retry_guidance(error_info: &DetectedError) -> String {
    let pattern = DELEGATE_TASK_ERROR_PATTERNS.iter()
        .find(|p| p.error_type == error_info.error_type);

    let mut guidance = if let Some(pattern) = pattern {
        format!("\n[delegate_task CALL FAILED - IMMEDIATE RETRY REQUIRED]\n\n**Error Type**: {}\n**Fix**: {}\n", 
                error_info.error_type, pattern.fix_hint)
    } else {
        "[delegate_task ERROR] Fix the error and retry with correct parameters.\n".to_string()
    };

    if let Some(available_list) = extract_available_list(&error_info.original_output) {
        guidance.push_str(&format!("\n**Available Options**: {}\n", available_list));
    }

    guidance.push_str("\n**Action**: Retry delegate_task NOW with corrected parameters.\n\nExample of CORRECT call:\n```\ndelegate_task(\n  description=\"Task description\",\n  prompt=\"Detailed prompt...\",\n  category=\"unspecified-low\",  // OR subagent_type=\"explore\"\n  run_in_background=false,\n  load_skills=[]\n)\n```\n");

    guidance
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

pub struct DelegateTaskRetryHook;

impl DelegateTaskRetryHook {
    pub fn new() -> Self {
        Self
    }

    pub async fn on_tool_execute_after(
        &self,
        input: &ToolExecuteInput,
        output: &mut ToolExecuteOutput,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if input.tool.to_lowercase() != "delegate_task" {
            return Ok(());
        }

        if let Some(error_info) = detect_delegate_task_error(&output.output) {
            let guidance = build_retry_guidance(&error_info);
            output.output.push_str(&guidance);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_delegate_task_error() {
        let output1 = "[ERROR] run_in_background parameter is missing";
        let result1 = detect_delegate_task_error(output1);
        assert!(result1.is_some());
        assert_eq!(result1.unwrap().error_type, "missing_run_in_background");

        let output2 = "Invalid arguments: Must provide either category or subagent_type";
        let result2 = detect_delegate_task_error(output2);
        assert!(result2.is_some());
        assert_eq!(result2.unwrap().error_type, "missing_category_or_agent");

        let output3 = "Success: Task completed";
        let result3 = detect_delegate_task_error(output3);
        assert!(result3.is_none());
    }

    #[test]
    fn test_extract_available_list() {
        let output = "Unknown category: invalid\nAvailable categories: general, quick, research\nMore details...";
        let result = extract_available_list(output);
        assert_eq!(result, Some("general, quick, research".to_string()));
    }

    #[tokio::test]
    async fn test_on_tool_execute_after() {
        let hook = DelegateTaskRetryHook::new();
        let input = ToolExecuteInput {
            tool: "delegate_task".to_string(),
            session_id: "session123".to_string(),
            call_id: "call123".to_string(),
        };

        let mut output = ToolExecuteOutput {
            title: "Test".to_string(),
            output: "[ERROR] run_in_background parameter is missing".to_string(),
            metadata: None,
        };

        let result = hook.on_tool_execute_after(&input, &mut output).await;
        assert!(result.is_ok());
        assert!(output.output.contains("[delegate_task CALL FAILED - IMMEDIATE RETRY REQUIRED]"));
        assert!(output.output.contains("Add run_in_background=false"));

        // Test with non-delegate_task tool
        let input2 = ToolExecuteInput {
            tool: "other_tool".to_string(),
            session_id: "session123".to_string(),
            call_id: "call123".to_string(),
        };

        let mut output2 = ToolExecuteOutput {
            title: "Test".to_string(),
            output: "[ERROR] run_in_background parameter is missing".to_string(),
            metadata: None,
        };

        let result2 = hook.on_tool_execute_after(&input2, &mut output2).await;
        assert!(result2.is_ok());
        assert!(!output2.output.contains("[delegate_task CALL FAILED"));
    }
}