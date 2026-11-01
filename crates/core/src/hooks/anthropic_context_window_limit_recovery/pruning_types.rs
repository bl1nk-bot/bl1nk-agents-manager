use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallSignature {
    pub tool_name: String,
    pub signature: String,
    pub call_id: String,
    pub turn: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOperation {
    pub call_id: String,
    pub tool: String,
    pub file_path: String,
    pub turn: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErroredToolCall {
    pub call_id: String,
    pub tool_name: String,
    pub turn: u32,
    pub error_age: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PruningResult {
    pub items_pruned: u32,
    pub total_tokens_saved: u32,
    pub strategies: PruningStrategies,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PruningStrategies {
    pub deduplication: u32,
    pub supersede_writes: u32,
    pub purge_errors: u32,
}

#[derive(Debug, Clone)]
pub struct PruningState {
    pub tool_ids_to_prune: HashSet<String>,
    pub current_turn: u32,
    pub file_operations: HashMap<String, Vec<FileOperation>>,
    pub tool_signatures: HashMap<String, Vec<ToolCallSignature>>,
    pub errored_tools: HashMap<String, ErroredToolCall>,
}

pub const CHARS_PER_TOKEN: usize = 4;

pub fn estimate_tokens(text: &str) -> usize {
    (text.len() as f64 / CHARS_PER_TOKEN as f64).ceil() as usize
}

impl PruningState {
    pub fn new() -> Self {
        Self {
            tool_ids_to_prune: HashSet::new(),
            current_turn: 0,
            file_operations: HashMap::new(),
            tool_signatures: HashMap::new(),
            errored_tools: HashMap::new(),
        }
    }

    pub fn add_tool_to_prune(&mut self, tool_id: String) {
        self.tool_ids_to_prune.insert(tool_id);
    }

    pub fn remove_tool_to_prune(&mut self, tool_id: &str) {
        self.tool_ids_to_prune.remove(tool_id);
    }

    pub fn is_tool_to_prune(&self, tool_id: &str) -> bool {
        self.tool_ids_to_prune.contains(tool_id)
    }

    pub fn add_file_operation(&mut self, session_id: String, operation: FileOperation) {
        self.file_operations.entry(session_id).or_insert_with(Vec::new).push(operation);
    }

    pub fn get_file_operations(&self, session_id: &str) -> Option<&Vec<FileOperation>> {
        self.file_operations.get(session_id)
    }

    pub fn add_tool_signature(&mut self, session_id: String, signature: ToolCallSignature) {
        self.tool_signatures.entry(session_id).or_insert_with(Vec::new).push(signature);
    }

    pub fn get_tool_signatures(&self, session_id: &str) -> Option<&Vec<ToolCallSignature>> {
        self.tool_signatures.get(session_id)
    }

    pub fn add_errored_tool(&mut self, session_id: String, tool: ErroredToolCall) {
        self.errored_tools.insert(session_id, tool);
    }

    pub fn get_errored_tool(&self, session_id: &str) -> Option<&ErroredToolCall> {
        self.errored_tools.get(session_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_tokens() {
        assert_eq!(estimate_tokens("hello"), 2); // 5 chars / 4 = 1.25 -> ceil = 2
        assert_eq!(estimate_tokens("hello world"), 3); // 11 chars / 4 = 2.75 -> ceil = 3
        assert_eq!(estimate_tokens(""), 0); // 0 chars / 4 = 0
    }

    #[test]
    fn test_pruning_state() {
        let mut state = PruningState::new();
        
        // ทดสอบการเพิ่ม tool ที่จะถูก prune
        state.add_tool_to_prune("tool1".to_string());
        assert!(state.is_tool_to_prune("tool1"));
        
        // ทดสอบการเพิ่ม file operation
        let operation = FileOperation {
            call_id: "call1".to_string(),
            tool: "edit".to_string(),
            file_path: "/path/file.txt".to_string(),
            turn: 1,
        };
        state.add_file_operation("session1".to_string(), operation);
        assert!(state.get_file_operations("session1").is_some());
        
        // ทดสอบการเพิ่ม tool signature
        let signature = ToolCallSignature {
            tool_name: "edit".to_string(),
            signature: "edit_file".to_string(),
            call_id: "call1".to_string(),
            turn: 1,
        };
        state.add_tool_signature("session1".to_string(), signature);
        assert!(state.get_tool_signatures("session1").is_some());
    }
}