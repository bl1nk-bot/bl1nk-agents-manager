use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartWorkInput {
    pub session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartWorkOutput {
    pub parts: Vec<MessagePart>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagePart {
    #[serde(rename = "type")]
    pub part_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartWorkContext {
    pub directory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanProgress {
    pub completed: u32,
    pub total: u32,
    pub is_complete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoulderState {
    pub plan_name: String,
    pub active_plan: String,
    pub session_ids: Vec<String>,
    pub started_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedTokenLimitError {
    pub current_tokens: u32,
    pub max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    pub error_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_index: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_struct_definitions() {
        // Test that structs can be instantiated
        let input = StartWorkInput {
            session_id: "test_session".to_string(),
            message_id: Some("test_message".to_string()),
        };
        assert_eq!(input.session_id, "test_session");

        let mut extra_fields = HashMap::new();
        extra_fields.insert("test_key".to_string(), serde_json::Value::String("test_value".to_string()));
        
        let part = MessagePart {
            part_type: "text".to_string(),
            text: Some("Hello, world!".to_string()),
            extra: extra_fields,
        };
        assert_eq!(part.part_type, "text");
        assert_eq!(part.text, Some("Hello, world!".to_string()));

        let ctx = StartWorkContext {
            directory: "/tmp/test".to_string(),
        };
        assert_eq!(ctx.directory, "/tmp/test");

        let progress = PlanProgress {
            completed: 5,
            total: 10,
            is_complete: false,
        };
        assert_eq!(progress.completed, 5);
        assert!(!progress.is_complete);

        let state = BoulderState {
            plan_name: "test_plan".to_string(),
            active_plan: "/path/to/plan.md".to_string(),
            session_ids: vec!["session1".to_string()],
            started_at: "2023-01-01T00:00:00Z".to_string(),
            updated_at: "2023-01-01T00:00:00Z".to_string(),
        };
        assert_eq!(state.plan_name, "test_plan");
        assert_eq!(state.session_ids.len(), 1);
    }
}