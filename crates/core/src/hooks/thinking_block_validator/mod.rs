use serde::{Deserialize, Serialize};
use crate::mcp::protocol::{Message, Part};

pub struct ThinkingBlockValidatorHook;

impl ThinkingBlockValidatorHook {
    pub fn new() -> Self { Self }

    pub fn is_extended_thinking_model(model_id: &str) -> bool {
        let lower = model_id.to_lowercase();
        lower.contains("thinking") || 
        lower.ends_with("-high") ||
        lower.contains("claude-sonnet-4") ||
        lower.contains("claude-opus-4") ||
        lower.contains("claude-3")
    }

    pub fn has_content_parts(parts: &[Part]) -> bool {
        parts.iter().any(|p| match p {
            Part::ToolUse { .. } | Part::Text { .. } => true,
            _ => false,
        })
    }

    pub fn starts_with_thinking_block(parts: &[Part]) -> bool {
        matches!(parts.get(0), Some(Part::Thinking { .. }) | Some(Part::Reasoning { .. }))
    }

    pub fn find_previous_thinking_content(messages: &[MessageWithParts], current_index: usize) -> String {
        for i in (0..current_index).rev() {
            let msg = &messages[i];
            if msg.role != "assistant" { continue; }

            for part in &msg.parts {
                if let Part::Thinking { thinking, .. } = part {
                    if !thinking.trim().is_empty() {
                        return thinking.clone();
                    }
                }
            }
        }
        String::new()
    }

    pub fn process_messages(&self, model_id: &str, messages: &mut Vec<MessageWithParts>) {
        if !Self::is_extended_thinking_model(model_id) { return; }

        for i in 0..messages.len() {
            // We need a way to check assistant messages and modify them
            // This is a simplified version of the logic
            if messages[i].role == "assistant" {
                if Self::has_content_parts(&messages[i].parts) && !Self::startsWith_thinking_block(&messages[i].parts) {
                    let prev_thinking = Self::find_previous_thinking_content(messages, i);
                    let content = if prev_thinking.is_empty() {
                        "[Continuing from previous reasoning]".to_string()
                    } else {
                        prev_thinking
                    };

                    // Prepend thinking part
                    messages[i].parts.insert(0, Part::Thinking {
                        thinking: content,
                        signature: None, // Simplified
                    });
                }
            }
        }
    }
}

// Mocking the structure needed for transformation
pub struct MessageWithParts {
    pub role: String,
    pub parts: Vec<Part>,
}
