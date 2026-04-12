use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryState {
    pub attempt: u32,
    pub last_attempt_time: u64, // timestamp in milliseconds
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TruncateState {
    pub truncate_attempt: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_truncated_part_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AutoCompactState {
    pub pending_compact: HashSet<String>,
    pub error_data_by_session: HashMap<String, ParsedTokenLimitError>,
    pub retry_state_by_session: HashMap<String, RetryState>,
    pub truncate_state_by_session: HashMap<String, TruncateState>,
    pub empty_content_attempt_by_session: HashMap<String, u32>,
    pub compaction_in_progress: HashSet<String>,
}

// ค่าคงที่สำหรับการ retry
pub const RETRY_CONFIG: RetryConfig = RetryConfig {
    max_attempts: 2,
    initial_delay_ms: 2000,
    backoff_factor: 2.0,
    max_delay_ms: 30000,
};

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub backoff_factor: f64,
    pub max_delay_ms: u64,
}

// ค่าคงที่สำหรับการ truncate
pub const TRUNCATE_CONFIG: TruncateConfig = TruncateConfig {
    max_truncate_attempts: 20,
    min_output_size_to_truncate: 500,
    target_token_ratio: 0.5,
    chars_per_token: 4,
};

#[derive(Debug, Clone)]
pub struct TruncateConfig {
    pub max_truncate_attempts: u32,
    pub min_output_size_to_truncate: usize,
    pub target_token_ratio: f64,
    pub chars_per_token: usize,
}

impl AutoCompactState {
    pub fn new() -> Self {
        Self {
            pending_compact: HashSet::new(),
            error_data_by_session: HashMap::new(),
            retry_state_by_session: HashMap::new(),
            truncate_state_by_session: HashMap::new(),
            empty_content_attempt_by_session: HashMap::new(),
            compaction_in_progress: HashSet::new(),
        }
    }

    pub fn add_pending_compact(&mut self, session_id: String) {
        self.pending_compact.insert(session_id);
    }

    pub fn remove_pending_compact(&mut self, session_id: &str) {
        self.pending_compact.remove(session_id);
    }

    pub fn is_pending_compact(&self, session_id: &str) -> bool {
        self.pending_compact.contains(session_id)
    }

    pub fn set_error_data(&mut self, session_id: String, error: ParsedTokenLimitError) {
        self.error_data_by_session.insert(session_id, error);
    }

    pub fn get_error_data(&self, session_id: &str) -> Option<&ParsedTokenLimitError> {
        self.error_data_by_session.get(session_id)
    }

    pub fn set_retry_state(&mut self, session_id: String, retry_state: RetryState) {
        self.retry_state_by_session.insert(session_id, retry_state);
    }

    pub fn get_retry_state(&self, session_id: &str) -> Option<&RetryState> {
        self.retry_state_by_session.get(session_id)
    }

    pub fn set_truncate_state(&mut self, session_id: String, truncate_state: TruncateState) {
        self.truncate_state_by_session.insert(session_id, truncate_state);
    }

    pub fn get_truncate_state(&self, session_id: &str) -> Option<&TruncateState> {
        self.truncate_state_by_session.get(session_id)
    }

    pub fn set_empty_content_attempt(&mut self, session_id: String, attempt: u32) {
        self.empty_content_attempt_by_session.insert(session_id, attempt);
    }

    pub fn get_empty_content_attempt(&self, session_id: &str) -> Option<u32> {
        self.empty_content_attempt_by_session.get(session_id).copied()
    }

    pub fn add_compaction_in_progress(&mut self, session_id: String) {
        self.compaction_in_progress.insert(session_id);
    }

    pub fn remove_compaction_in_progress(&mut self, session_id: &str) {
        self.compaction_in_progress.remove(session_id);
    }

    pub fn is_compaction_in_progress(&self, session_id: &str) -> bool {
        self.compaction_in_progress.contains(session_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_compact_state() {
        let mut state = AutoCompactState::new();
        
        // ทดสอบการเพิ่ม session ที่รอ compact
        state.add_pending_compact("session1".to_string());
        assert!(state.is_pending_compact("session1"));
        
        // ทดสอบการลบ session ที่รอ compact
        state.remove_pending_compact("session1");
        assert!(!state.is_pending_compact("session1"));
        
        // ทดสอบการตั้งค่าและรับข้อมูล error
        let error = ParsedTokenLimitError {
            current_tokens: 1000,
            max_tokens: 2000,
            request_id: Some("req123".to_string()),
            error_type: "token_limit".to_string(),
            provider_id: Some("anthropic".to_string()),
            model_id: Some("claude-3".to_string()),
            message_index: Some(1),
        };
        
        state.set_error_data("session1".to_string(), error.clone());
        assert_eq!(state.get_error_data("session1").unwrap().current_tokens, 1000);
    }
}