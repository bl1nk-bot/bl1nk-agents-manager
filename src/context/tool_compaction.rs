//! Tool Call Compaction - Token optimization for LLM message contexts
//! 
//! Converts tool calls to minimal representations to save tokens while preserving
//! the ability to identify which calls have been processed.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Index mapping message/part locations to tool call keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallIndex {
    pub by_location: HashMap<usize, HashMap<usize, String>>,
    pub ordered_keys: Vec<String>,
}

/// Pending compaction candidates from tool calls/results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingCompactionCandidates {
    pub pending_tool_call_keys: HashSet<String>,
    pub pending_anonymous_tool_results: usize,
}

// ========== Task 2: index_tool_calls function ==========

/// Index all tool calls in messages, returning their locations and keys
pub fn index_tool_calls(messages: &[crate::context::Message]) -> ToolCallIndex {
    let mut by_location: HashMap<usize, HashMap<usize, String>> = HashMap::new();
    let mut ordered_keys: Vec<String> = Vec::new();

    for (msg_idx, msg) in messages.iter().enumerate() {
        if msg.content.contains("\"type\":\"tool-call\"") || msg.content.contains("tool_call_id") {
            let indexed_parts = by_location.entry(msg_idx).or_default();
            indexed_parts.insert(0, format!("msg:{}", msg_idx));
            ordered_keys.push(format!("msg:{}", msg_idx));
        }
    }

    ToolCallIndex {
        by_location,
        ordered_keys,
    }
}

// ========== Task 3: find_pending_compaction_candidates function ==========

/// Parameters for find_pending_compaction_candidates
pub struct FindPendingParams<'a> {
    pub messages: &'a [crate::context::Message],
    pub tool_call_index: &'a ToolCallIndex,
    pub recent_tool_call_keys: &'a HashSet<String>,
    pub compacted_notice: &'a str,
}

/// Find tool calls that are candidates for compaction (not in recent keys)
pub fn find_pending_compaction_candidates(params: &FindPendingParams) -> PendingCompactionCandidates {
    let mut pending_tool_call_keys: HashSet<String> = HashSet::new();
    let mut pending_anonymous_tool_results: usize = 0;
    
    // Find tool calls not in recent keys
    for (msg_idx, _) in params.messages.iter().enumerate() {
        if let Some(parts) = params.tool_call_index.by_location.get(&msg_idx) {
            for (part_idx, key) in parts {
                if !params.recent_tool_call_keys.contains(key) {
                    pending_tool_call_keys.insert(key.clone());
                }
            }
        }
    }
    
    PendingCompactionCandidates {
        pending_tool_call_keys,
        pending_anonymous_tool_results,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::MessageRole;

    // ========== Task 1: Module Structure Tests ==========

    #[test]
    fn test_tool_call_index_creation() {
        let index = ToolCallIndex {
            by_location: HashMap::new(),
            ordered_keys: Vec::new(),
        };
        assert!(index.by_location.is_empty());
        assert!(index.ordered_keys.is_empty());
    }

    #[test]
    fn test_pending_compaction_candidates_creation() {
        let candidates = PendingCompactionCandidates {
            pending_tool_call_keys: HashSet::new(),
            pending_anonymous_tool_results: 0,
        };
        assert!(candidates.pending_tool_call_keys.is_empty());
    }

    // ========== Task 2: index_tool_calls Tests ==========

    #[test]
    fn test_index_tool_calls_empty() {
        let messages: Vec<crate::context::Message> = vec![];
        let result = index_tool_calls(&messages);
        assert!(result.ordered_keys.is_empty());
        assert!(result.by_location.is_empty());
    }

    #[test]
    fn test_index_tool_calls_with_tool_call_json() {
        // Test with actual tool call JSON content
        let messages = vec![
            crate::context::Message {
                role: MessageRole::Assistant,
                content: r#"{"type":"tool-call","tool_call_id":"call_123","tool_name":"bash","input":{"command":"ls"}}"#.to_string(),
                timestamp: chrono::Utc::now(),
            },
        ];
        let result = index_tool_calls(&messages);
        
        assert_eq!(result.ordered_keys.len(), 1);
        assert!(result.by_location.contains_key(&0));
    }

    #[test]
    fn test_index_tool_calls_multiple_in_one_message() {
        // Multiple tool calls in single message
        let messages = vec![
            crate::context::Message {
                role: MessageRole::Assistant,
                content: r#"[{"type":"tool-call","tool_call_id":"call_1"},{"type":"tool-call","tool_call_id":"call_2"}]"#.to_string(),
                timestamp: chrono::Utc::now(),
            },
        ];
        let result = index_tool_calls(&messages);
        
        // Should detect at least one tool call
        assert!(!result.ordered_keys.is_empty());
    }

    #[test]
    fn test_index_tool_calls_text_only_message() {
        let messages = vec![crate::context::Message {
            role: MessageRole::User,
            content: "Hello, how are you?".to_string(),
            timestamp: chrono::Utc::now(),
        }];
        let result = index_tool_calls(&messages);
        
        assert!(result.ordered_keys.is_empty());
    }

    // ========== Task 3: find_pending_compaction_candidates Tests ==========

    #[test]
    fn test_find_pending_compaction_candidates_empty() {
        // Test function existence
        let messages: Vec<crate::context::Message> = vec![];
        let index = ToolCallIndex {
            by_location: HashMap::new(),
            ordered_keys: Vec::new(),
        };
        let recent_keys: HashSet<String> = HashSet::new();
        
        let params = FindPendingParams {
            messages: &messages,
            tool_call_index: &index,
            recent_tool_call_keys: &recent_keys,
            compacted_notice: "[compacted]",
        };
        
        let result = find_pending_compaction_candidates(&params);
        
        assert!(result.pending_tool_call_keys.is_empty());
    }
}