//! การบีบอัดข้อมูลการเรียกใช้เครื่องมือ (Tool Call Compaction) - ปรับปรุงจำนวน Token สำหรับบริบทข้อความของ LLM
//!
//! แปลงการเรียกใช้เครื่องมือให้เป็นรูปแบบย่อเพื่อประหยัดจำนวน Token โดยยังคงรักษา
//! ความสามารถในการระบุว่าการเรียกใดถูกประมวลผลไปแล้ว

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::context::MessageRole;

/// ส่วนประกอบของเนื้อหาข้อความ - ทั้ง text หรือ tool call/result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool-call")]
    ToolCall {
        tool_call_id: Option<String>,
        tool_name: String,
        input: serde_json::Value,
    },
    #[serde(rename = "tool-result")]
    ToolResult {
        tool_call_id: Option<String>,
        output: serde_json::Value,
    },
}

/// การแทนข้อความในรูปแบบ array (เทียบเท่า ModelMessage[])
#[derive(Debug, Clone)]
pub struct MessageContent {
    pub role: MessageRole,
    pub content: Vec<ContentPart>,
}

impl MessageContent {
    /// สร้างจาก string content (parse JSON หรือ text)
    pub fn from_string(role: MessageRole, content: &str) -> Self {
        // ลอง parse เป็น JSON array
        if let Ok(parsed) = serde_json::from_str::<Vec<ContentPart>>(content) {
            return Self {
                role,
                content: parsed,
            };
        }

        // Fallback เป็น text
        Self {
            role,
            content: vec![ContentPart::Text {
                text: content.to_string(),
            }],
        }
    }
}

/// ดัชนีระบุตำแหน่งของข้อความ/ส่วนประกอบไปยังคีย์การเรียกใช้เครื่องมือ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallIndex {
    pub by_location: HashMap<usize, HashMap<usize, String>>,
    pub ordered_keys: Vec<String>,
}

/// รายการที่รอการบีบอัดจากการเรียกใช้เครื่องมือหรือผลลัพธ์
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingCompactionCandidates {
    pub pending_tool_call_keys: HashSet<String>,
    pub pending_anonymous_tool_results: usize,
}

// ========== ภารกิจที่ 2: ฟังก์ชัน index_tool_calls ==========

/// สร้างดัชนีการเรียกใช้เครื่องมือทั้งหมดในข้อความ เพื่อติดตามตำแหน่งและคีย์
pub fn index_tool_calls(messages: &[crate::context::Message]) -> ToolCallIndex {
    let mut by_location: HashMap<usize, HashMap<usize, String>> = HashMap::new();
    let mut ordered_keys: Vec<String> = Vec::new();

    for (msg_idx, msg) in messages.iter().enumerate() {
        // ตรวจสอบเบื้องต้นว่าข้อความมีการเรียกใช้เครื่องมือหรือไม่
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

// ========== ภารกิจที่ 3: ฟังก์ชัน find_pending_compaction_candidates ==========

/// พารามิเตอร์สำหรับ find_pending_compaction_candidates
pub struct FindPendingParams<'a> {
    pub messages: &'a [crate::context::Message],
    pub tool_call_index: &'a ToolCallIndex,
    pub recent_tool_call_keys: &'a HashSet<String>,
    pub compacted_notice: &'a str,
}

/// ค้นหาการเรียกใช้เครื่องมือที่สามารถบีบอัดได้ (ไม่อยู่ในรายการคีย์ล่าสุด)
pub fn find_pending_compaction_candidates(params: &FindPendingParams) -> PendingCompactionCandidates {
    let mut pending_tool_call_keys: HashSet<String> = HashSet::new();
    let pending_anonymous_tool_results: usize = 0;

    // ค้นหาการเรียกใช้เครื่องมือที่ไม่อยู่ในคีย์ล่าสุด
    for (msg_idx, _) in params.messages.iter().enumerate() {
        if let Some(parts) = params.tool_call_index.by_location.get(&msg_idx) {
            for key in parts.values() {
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
        let messages = vec![crate::context::Message {
            role: MessageRole::Assistant,
            content: r#"{"type":"tool-call","tool_call_id":"call_123","tool_name":"bash","input":{"command":"ls"}}"#
                .to_string(),
            timestamp: chrono::Utc::now(),
        }];
        let result = index_tool_calls(&messages);

        assert_eq!(result.ordered_keys.len(), 1);
        assert!(result.by_location.contains_key(&0));
    }

    #[test]
    fn test_index_tool_calls_multiple_in_one_message() {
        // Multiple tool calls in single message
        let messages = vec![crate::context::Message {
            role: MessageRole::Assistant,
            content: r#"[{"type":"tool-call","tool_call_id":"call_1"},{"type":"tool-call","tool_call_id":"call_2"}]"#
                .to_string(),
            timestamp: chrono::Utc::now(),
        }];
        let result = index_tool_calls(&messages);

        // Should detect at least one tool call
        assert!(!result.ordered_keys.is_empty());
    }

    // ========== Phase 2: New Tests ==========

    #[test]
    fn test_content_part_parse() {
        let json = r#"[
            {"type": "text", "text": "Hello"},
            {"type": "tool-call", "tool_call_id": "call_1", "tool_name": "bash", "input": {"command": "ls"}}
        ]"#;

        let content = MessageContent::from_string(MessageRole::Assistant, json);
        assert_eq!(content.content.len(), 2);
    }

    #[test]
    fn test_is_tool_call_part() {
        let part = ContentPart::ToolCall {
            tool_call_id: Some("call_1".to_string()),
            tool_name: "bash".to_string(),
            input: serde_json::json!({}),
        };
        assert!(is_tool_call_part(&part));
    }

    #[test]
    fn test_is_tool_result_part() {
        let part = ContentPart::ToolResult {
            tool_call_id: Some("call_1".to_string()),
            output: serde_json::json!({"type": "text", "value": "result"}),
        };
        assert!(is_tool_result_part(&part));
    }

    #[test]
    fn test_compact_tool_call() {
        let part = ContentPart::ToolCall {
            tool_call_id: Some("call_1".to_string()),
            tool_name: "bash".to_string(),
            input: serde_json::json!({"command": "ls -la"}),
        };
        let compacted = compact_tool_call(&part, "[compacted]");

        if let ContentPart::ToolCall { input, .. } = compacted {
            assert_eq!(input.get("compacted"), Some(&serde_json::json!(true)));
            assert_eq!(input.get("message"), Some(&serde_json::json!("[compacted]")));
        } else {
            panic!("Expected ToolCall");
        }
    }

    #[test]
    fn test_compact_tool_result() {
        let part = ContentPart::ToolResult {
            tool_call_id: Some("call_1".to_string()),
            output: serde_json::json!({"type": "text", "value": "long output here"}),
        };
        let compacted = compact_tool_result(&part, "[compacted]");

        if let ContentPart::ToolResult { output, .. } = compacted {
            assert_eq!(output.get("type"), Some(&serde_json::json!("text")));
            assert_eq!(output.get("value"), Some(&serde_json::json!("[compacted]")));
        } else {
            panic!("Expected ToolResult");
        }
    }

    #[test]
    fn test_is_compacted_tool_call_part() {
        let part = ContentPart::ToolCall {
            tool_call_id: Some("call_1".to_string()),
            tool_name: "bash".to_string(),
            input: serde_json::json!({"compacted": true, "message": "[compacted]"}),
        };
        assert!(is_compacted_tool_call_part(&part, "[compacted]"));
    }

    #[test]
    fn test_get_pending_compaction_units() {
        let mut keys = HashSet::new();
        keys.insert("id:call_1".to_string());
        keys.insert("id:call_2".to_string());

        let candidates = PendingCompactionCandidates {
            pending_tool_call_keys: keys,
            pending_anonymous_tool_results: 3,
        };

        let units = get_pending_compaction_units(&candidates);
        assert_eq!(units, 5); // 2 keys + 3 anonymous
    }
}

// ========== Phase 2: Helper Functions ==========

/// ตรวจสอบว่า part เป็น tool call หรือไม่
pub fn is_tool_call_part(part: &ContentPart) -> bool {
    matches!(part, ContentPart::ToolCall { .. })
}

/// ตรวจสอบว่า part เป็น tool result หรือไม่
pub fn is_tool_result_part(part: &ContentPart) -> bool {
    matches!(part, ContentPart::ToolResult { .. })
}

/// ตรวจสอบว่า tool call part ถูกบีบอัดแล้วหรือไม่
pub fn is_compacted_tool_call_part(part: &ContentPart, compacted_notice: &str) -> bool {
    if let ContentPart::ToolCall { input, .. } = part {
        if let Some(obj) = input.as_object() {
            if let Some(compacted) = obj.get("compacted") {
                if compacted.as_bool() == Some(true) {
                    if let Some(msg) = obj.get("message") {
                        return msg.as_str() == Some(compacted_notice);
                    }
                }
            }
        }
    }
    false
}

/// ตรวจสอบว่า tool result part ถูกบีบอัดแล้วหรือไม่
pub fn is_compacted_tool_result_part(part: &ContentPart, compacted_notice: &str) -> bool {
    if let ContentPart::ToolResult { output, .. } = part {
        if let Some(obj) = output.as_object() {
            if let Some(t) = obj.get("type") {
                if t.as_str() == Some("text") {
                    if let Some(v) = obj.get("value") {
                        return v.as_str() == Some(compacted_notice);
                    }
                }
            }
        }
    }
    false
}

/// บีบอัด tool call part เป็นรูปแบบย่อ
pub fn compact_tool_call(part: &ContentPart, compacted_notice: &str) -> ContentPart {
    if let ContentPart::ToolCall {
        tool_call_id,
        tool_name,
        input: _,
    } = part
    {
        ContentPart::ToolCall {
            tool_call_id: tool_call_id.clone(),
            tool_name: tool_name.clone(),
            input: serde_json::json!({
                "compacted": true,
                "message": compacted_notice,
            }),
        }
    } else {
        part.clone()
    }
}

/// บีบอัด tool result part เป็นรูปแบบย่อ
pub fn compact_tool_result(part: &ContentPart, compacted_notice: &str) -> ContentPart {
    if let ContentPart::ToolResult {
        tool_call_id,
        output: _,
    } = part
    {
        ContentPart::ToolResult {
            tool_call_id: tool_call_id.clone(),
            output: serde_json::json!({
                "type": "text",
                "value": compacted_notice,
            }),
        }
    } else {
        part.clone()
    }
}

/// Parameters for estimate_compaction_savings
pub struct EstimateSavingsParams<'a> {
    pub messages: &'a [MessageContent],
    pub tool_call_index: &'a ToolCallIndex,
    pub pending_candidates: &'a PendingCompactionCandidates,
    pub compacted_notice: &'a str,
}

/// ประมาณการประหยัด token จากการบีบอัด (คืนค่า estimated tokens ที่ประหยัดได้)
pub fn estimate_compaction_savings(params: &EstimateSavingsParams) -> usize {
    let mut savings_chars: isize = 0;

    for (message_index, message) in params.messages.iter().enumerate() {
        let part_keys = params.tool_call_index.by_location.get(&message_index);

        for (part_index, part) in message.content.iter().enumerate() {
            let old_length = serde_json::to_string(part).map(|s| s.len()).unwrap_or(0);
            let mut compacted_part: Option<ContentPart> = None;

            if is_tool_call_part(part) {
                if let Some(key) = part_keys.and_then(|k| k.get(&part_index)) {
                    if params.pending_candidates.pending_tool_call_keys.contains(key)
                        && !is_compacted_tool_call_part(part, params.compacted_notice)
                    {
                        compacted_part = Some(compact_tool_call(part, params.compacted_notice));
                    }
                }
            } else if is_tool_result_part(part) {
                let tool_call_id = if let ContentPart::ToolResult { tool_call_id, .. } = part {
                    tool_call_id.clone()
                } else {
                    None
                };

                let key = tool_call_id.map(|id| format!("id:{}", id));

                if let Some(ref k) = key {
                    if params.pending_candidates.pending_tool_call_keys.contains(k)
                        && !is_compacted_tool_result_part(part, params.compacted_notice)
                    {
                        compacted_part = Some(compact_tool_result(part, params.compacted_notice));
                    }
                } else if params.pending_candidates.pending_anonymous_tool_results > 0
                    && !is_compacted_tool_result_part(part, params.compacted_notice)
                {
                    compacted_part = Some(compact_tool_result(part, params.compacted_notice));
                }
            }

            if let Some(new_part) = compacted_part {
                let new_length = serde_json::to_string(&new_part).map(|s| s.len()).unwrap_or(0);
                let delta = old_length as isize - new_length as isize;
                if delta > 0 {
                    savings_chars += delta;
                }
            }
        }
    }

    // Convert chars to estimated tokens (roughly 4 chars per token)
    ((savings_chars as f64) / 4.0).ceil() as usize
}

/// Parameters for compact_tool_data
pub struct CompactToolDataParams<'a> {
    pub messages: &'a [MessageContent],
    pub tool_call_index: &'a ToolCallIndex,
    pub pending_candidates: &'a PendingCompactionCandidates,
    pub compacted_notice: &'a str,
}

/// บีบอัดข้อมูล tool calls/results ใน messages คืน message ใหม่
pub fn compact_tool_data(params: &CompactToolDataParams) -> Vec<MessageContent> {
    params
        .messages
        .iter()
        .enumerate()
        .map(|(message_index, message)| {
            let part_keys = params.tool_call_index.by_location.get(&message_index);

            let mut changed = false;
            let compacted_content: Vec<ContentPart> = message
                .content
                .iter()
                .enumerate()
                .map(|(part_index, part)| {
                    if is_tool_call_part(part) {
                        if let Some(key) = part_keys.and_then(|k| k.get(&part_index)) {
                            if params.pending_candidates.pending_tool_call_keys.contains(key)
                                && !is_compacted_tool_call_part(part, params.compacted_notice)
                            {
                                changed = true;
                                return compact_tool_call(part, params.compacted_notice);
                            }
                        }
                    }

                    if is_tool_result_part(part) {
                        let tool_call_id = if let ContentPart::ToolResult { tool_call_id, .. } =
                            part
                        {
                            tool_call_id.clone()
                        } else {
                            None
                        };

                        let key = tool_call_id.map(|id| format!("id:{}", id));

                        if let Some(ref k) = key {
                            if params.pending_candidates.pending_tool_call_keys.contains(k)
                                && !is_compacted_tool_result_part(part, params.compacted_notice)
                            {
                                changed = true;
                                return compact_tool_result(part, params.compacted_notice);
                            }
                        }

                        if key.is_none()
                            && params.pending_candidates.pending_anonymous_tool_results > 0
                            && !is_compacted_tool_result_part(part, params.compacted_notice)
                        {
                            changed = true;
                            return compact_tool_result(part, params.compacted_notice);
                        }
                    }

                    part.clone()
                })
                .collect();

            if !changed {
                return message.clone();
            }

            MessageContent {
                role: message.role.clone(),
                content: compacted_content,
            }
        })
        .collect()
}

/// รวมจำนวน pending compaction units ทั้งหมด
pub fn get_pending_compaction_units(candidates: &PendingCompactionCandidates) -> usize {
    candidates.pending_tool_call_keys.len() + candidates.pending_anonymous_tool_results
}

// ═══════════════════════════════════════════════════════════════════════════════════
// PHASE 3: Context Management - Token Budget, History, Offload
// ═══════════════════════════════════════════════════════════════════════════════════

// Phase 3 imports
use std::collections::HashMap;
use std::path::PathBuf;

use crate::context::Message;

/// Token budget configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBudget {
    pub max_tokens: u32,
    pub warning_threshold_percent: u8,  // e.g., 80 = warn at 80%
    pub eviction_strategy: EvictionStrategy,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EvictionStrategy {
    /// Keep oldest messages (FIFO)
    OldestFirst,
    /// Keep newest messages (LIFO)
    NewestFirst,
    /// Keep messages with important roles (system, user > assistant > tool)
    PriorityBased,
    /// Keep recent + summary of older messages
    SlidingWindowWithSummary,
}

impl Default for TokenBudget {
    fn default() -> Self {
        Self {
            max_tokens: 100_000,  // ~400k tokens = 100k
            warning_threshold_percent: 80,
            eviction_strategy: EvictionStrategy::PriorityBased,
        }
    }
}

/// Check if token budget needs compaction
pub fn needs_compaction(current_tokens: u32, budget: &TokenBudget) -> bool {
    current_tokens > budget.max_tokens
}

/// Get current usage percentage
pub fn token_usage_percent(current_tokens: u32, budget: &TokenBudget) -> u8 {
    ((current_tokens as f64 / budget.max_tokens as f64) * 100.0) as u8
}

/// Check if should warn user
pub fn should_warn_token_limit(current_tokens: u32, budget: &TokenBudget) -> bool {
    token_usage_percent(current_tokens, budget) >= budget.warning_threshold_percent
}

/// Tool use history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUseHistoryEntry {
    pub tool_name: String,
    pub call_count: u32,
    pub last_called_timestamp: i64,
    pub cached_result_available: bool,
}

/// Tool use history
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolUseHistory {
    entries: HashMap<String, ToolUseHistoryEntry>,
}

impl ToolUseHistory {
    pub fn new() -> Self {
        Self { entries: HashMap::new() }
    }

    /// Record a tool call
    pub fn record_call(&mut self, tool_name: &str, timestamp: i64) {
        let entry = self.entries.entry(tool_name.to_string()).or_insert(
            ToolUseHistoryEntry {
                tool_name: tool_name.to_string(),
                call_count: 0,
                last_called_timestamp: timestamp,
                cached_result_available: false,
            }
        );
        entry.call_count += 1;
        entry.last_called_timestamp = timestamp;
    }

    /// Mark cached result available
    pub fn mark_cached(&mut self, tool_name: &str) {
        if let Some(entry) = self.entries.get_mut(tool_name) {
            entry.cached_result_available = true;
        }
    }

    /// Get entry
    pub fn get(&self, tool_name: &str) -> Option<&ToolUseHistoryEntry> {
        self.entries.get(tool_name)
    }

    /// Get all entries sorted by call count
    pub fn sorted_by_frequency(&self) -> Vec<&ToolUseHistoryEntry> {
        let mut entries: Vec<_> = self.entries.values().collect();
        entries.sort_by(|a, b| b.call_count.cmp(&a.call_count));
        entries
    }
}

/// Context offload config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextOffloadConfig {
    pub enabled: bool,
    pub offload_threshold_percent: u8,
    pub archive_directory: PathBuf,
    pub max_archived_messages: usize,
}

impl Default for ContextOffloadConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            offload_threshold_percent: 90,
            archive_directory: PathBuf::from(".bl1nk/archive"),
            max_archived_messages: 100,
        }
    }
}

/// Offloaded message archive entry
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ArchivedMessage {
    pub role: String,
    pub content: String,
    pub original_index: usize,
    pub archived_at_timestamp: i64,
}

/// Calculate estimated tokens (simple word-based approximation)
pub fn estimate_tokens(text: &str) -> u32 {
    // Rough approximation: ~4 characters per token
    (text.len() as f64 / 4.0) as u32
}

/// Compact messages based on budget (select important ones to keep)
pub fn compact_messages_by_budget(
    messages: &[Message],
    budget: &TokenBudget,
    now_timestamp: i64,
) -> Vec<Message> {
    if messages.is_empty() {
        return vec![];
    }

    // Score each message by importance
    let mut scored: Vec<(usize, i32)> = messages.iter().enumerate().map(|(i, msg)| {
        let importance = match msg.role {
            MessageRole::System => 100,
            MessageRole::User => 80,
            MessageRole::Assistant => 60,
        };
        // Prefer recent messages (higher score = more recent)
        let recency = (i as i32) / (messages.len() as i32) * 20;
        (i, importance + recency)
    }).collect();

    // Sort by importance descending
    scored.sort_by(|a, b| b.1.cmp(&a.1));

    // Calculate token budget for messages (reserve some for metadata)
    let message_budget = (budget.max_tokens as f64 * 0.8) as u32;

    // Select messages until we fill budget
    let mut selected = Vec::new();
    let mut total_tokens = 0;

    for (original_idx, _) in scored {
        let msg = &messages[original_idx];
        let msg_tokens = estimate_tokens(&msg.content);

        if total_tokens + msg_tokens > message_budget {
            break;
        }
        total_tokens += msg_tokens;
        selected.push((original_idx, msg.clone()));
    }

    // Sort back to original order
    selected.sort_by(|a, b| a.0.cmp(&b.0));
    selected.into_iter().map(|(_, msg)| msg).collect()
}

/// Offload excess messages to file
pub fn offload_messages_to_archive(
    messages: &[Message],
    config: &ContextOffloadConfig,
    now_timestamp: i64,
) -> (Vec<Message>, Vec<ArchivedMessage>) {
    if !config.enabled || messages.len() <= config.max_archived_messages {
        return (messages.to_vec(), vec![]);
    }

    // Keep recent messages, archive older ones
    let keep_count = messages.len() - config.max_archived_messages;
    let (keep, archive_raw) = messages.split_at(keep_count);

    let archived: Vec<ArchivedMessage> = archive_raw.iter().enumerate().map(|(i, msg)| {
        let role_str = match msg.role {
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant", 
            MessageRole::System => "system",
        };
        ArchivedMessage {
            role: role_str.to_string(),
            content: msg.content.clone(),
            original_index: keep_count + i,
            archived_at_timestamp: now_timestamp,
        }
    }).collect();

    (keep.to_vec(), archived)
}

#[cfg(test)]
mod phase3_tests {
    use super::*;

    // Token Budget tests
    #[test]
    fn test_needs_compaction() {
        let budget = TokenBudget::default();
        assert!(!needs_compaction(50_000, &budget));
        assert!(needs_compaction(150_000, &budget));
    }

    #[test]
    fn test_token_usage_percent() {
        let budget = TokenBudget {
            max_tokens: 100_000,
            warning_threshold_percent: 80,
            eviction_strategy: EvictionStrategy::PriorityBased,
        };
        assert_eq!(token_usage_percent(80_000, &budget), 80);
        assert_eq!(token_usage_percent(50_000, &budget), 50);
    }

    #[test]
    fn test_should_warn_token_limit() {
        let budget = TokenBudget {
            max_tokens: 100_000,
            warning_threshold_percent: 80,
            eviction_strategy: EvictionStrategy::PriorityBased,
        };
        assert!(should_warn_token_limit(90_000, &budget));
        assert!(!should_warn_token_limit(70_000, &budget));
    }

    // Tool Use History tests
    #[test]
    fn test_tool_use_history_record() {
        let mut history = ToolUseHistory::new();
        history.record_call("read_file", 1000);
        history.record_call("read_file", 2000);
        history.record_call("write_file", 1500);

        let entry = history.get("read_file").unwrap();
        assert_eq!(entry.call_count, 2);
        assert_eq!(entry.last_called_timestamp, 2000);
    }

    #[test]
    fn test_tool_use_history_sorted_by_frequency() {
        let mut history = ToolUseHistory::new();
        history.record_call("terminal", 1000);
        history.record_call("terminal", 2000);
        history.record_call("terminal", 3000);
        history.record_call("read_file", 1500);
        history.record_call("read_file", 2500);

        let sorted = history.sorted_by_frequency();
        assert_eq!(sorted[0].tool_name, "terminal");
        assert_eq!(sorted[1].tool_name, "read_file");
    }

    // Token estimation test
    #[test]
    fn test_estimate_tokens() {
        let text = "Hello world this is a test string";
        // ~7 words / 4 = ~2 tokens approx
        let tokens = estimate_tokens(text);
        assert!(tokens > 0);
    }

    // Offload config test
    #[test]
    fn test_context_offload_config_default() {
        let config = ContextOffloadConfig::default();
        assert!(config.enabled);
        assert_eq!(config.offload_threshold_percent, 90);
    }

    // Archived message test
    #[test]
    fn test_archived_message_serialization() {
        let archived = ArchivedMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
            original_index: 5,
            archived_at_timestamp: 1000,
        };
        let json = serde_json::to_string(&archived).unwrap();
        let parsed: ArchivedMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.role, "user");
        assert_eq!(parsed.original_index, 5);
    }

    // Token budget serialization test
    #[test]
    fn test_token_budget_serialization() {
        let budget = TokenBudget::default();
        let json = serde_json::to_string(&budget).unwrap();
        let parsed: TokenBudget = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.max_tokens, 100_000);
    }
}
