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
        // For now, check if content contains tool call JSON
        // This is a placeholder - will be extended in later tasks
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
        // Test that index_tool_calls function exists
        let messages: Vec<crate::context::Message> = vec![];
        let result = index_tool_calls(&messages);
        assert!(result.ordered_keys.is_empty());
        assert!(result.by_location.is_empty());
    }
}