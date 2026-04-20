# Tool Call Compaction Implementation Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Convert TypeScript tool call compaction library to Rust for token optimization in LLM message contexts.

**Status (2026-04-20):**
- ✅ Phase 1: Core data structures (basic implementation exists)
- ❌ Phase 2: Compaction logic (9 functions missing)
- ❌ Phase 3: Integration

**Codebase Comparison:**
| Rust (current) | TypeScript (original) |
|----------------|---------------------|
| `ToolCallIndex` (basic) | Full implementation |
| `PendingCompactionCandidates` | Full implementation |
| `index_tool_calls` (basic parsing) | Full with ContentPart enum |
| `find_pending_compaction_candidates` (basic) | Full |
| ❌ Missing: ContentPart, MessageContent | Full JSON array content |
| ❌ Missing: 9 functions | Full |

---

## Phase 1: Core Data Structures & Indexing [✅ DONE - basic]

### Task 1: Create tool_compaction module [✅]
### Task 2: Implement indexToolCalls function [✅]
### Task 3: Implement findPendingCompactionCandidates [✅]

## Phase 2: Compaction Logic [✅ DONE]

### Task 4: Implement compact functions [✅ DONE - 12 tests]

**Added:**
- `ContentPart` enum (Text, ToolCall, ToolResult)
- `MessageContent` struct with `from_string()` parser
- `is_tool_call_part()`, `is_tool_result_part()`
- `is_compacted_tool_call_part()`, `is_compacted_tool_result_part()`
- `compact_tool_call()`, `compact_tool_result()`
- `estimate_compaction_savings()`, `compact_tool_data()`
- `get_pending_compaction_units()`
- **12 tests pass**

---

**Step 2: Export from mod.rs**

Add to `src/context/mod.rs`:

```rust
pub mod tool_compaction;
pub use tool_compaction::*;
```

**Step 3: Verify compilation**

Run: `cargo check --lib`
Expected: SUCCESS

**Step 4: Commit**

```bash
git add src/context/
git commit -m "feat(context): add tool_compaction module structure"
```

---

### Task 2: Implement indexToolCalls function

**Objective:** Index all tool calls in messages to track them by location

**Files:**
- Modify: `src/context/tool_compaction.rs`

**Step 1: Add message type stubs**

```rust
/// Role of a message sender (re-export from context)
pub use crate::context::MessageRole;

/// Simple message representation for tool indexing
#[derive(Debug, Clone)]
pub struct MessageContent {
    pub role: MessageRole,
    pub content: Vec<ContentPart>,
}

/// Content part - either text or tool call/result
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
```

**Step 2: Add indexToolCalls function**

```rust
/// Index all tool calls in messages, returning their locations and keys
pub fn index_tool_calls(messages: &[MessageContent]) -> ToolCallIndex {
    let mut by_location: HashMap<usize, HashMap<usize, String>> = HashMap::new();
    let mut ordered_keys: Vec<String> = Vec::new();
    let mut anonymous_call_index: usize = 0;

    for (message_index, message) in messages.iter().enumerate() {
        for (part_index, part) in message.content.iter().enumerate() {
            if let ContentPart::ToolCall { tool_call_id, .. } = part {
                let key = match tool_call_id {
                    Some(id) => format!("id:{}", id),
                    None => {
                        let k = format!("anon:{}", anonymous_call_index);
                        anonymous_call_index += 1;
                        k
                    }
                };

                let indexed_parts = by_location.entry(message_index).or_default();
                indexed_parts.insert(part_index, key.clone());
                ordered_keys.push(key);
            }
        }
    }

    ToolCallIndex {
        by_location,
        ordered_keys,
    }
}
```

**Step 3: Add tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_tool_calls_basic() {
        let messages = vec![
            MessageContent {
                role: MessageRole::Assistant,
                content: vec![
                    ContentPart::ToolCall {
                        tool_call_id: Some("call_123".to_string()),
                        tool_name: "bash".to_string(),
                        input: serde_json::json!({"command": "ls"}),
                    },
                ],
            },
        ];

        let index = index_tool_calls(&messages);
        
        assert_eq!(index.ordered_keys.len(), 1);
        assert_eq!(index.ordered_keys[0], "id:call_123");
        assert!(index.by_location.contains_key(&0));
    }

    #[test]
    fn test_index_tool_calls_multiple() {
        let messages = vec![
            MessageContent {
                role: MessageRole::Assistant,
                content: vec![
                    ContentPart::ToolCall {
                        tool_call_id: Some("call_1".to_string()),
                        tool_name: "bash".to_string(),
                        input: serde_json::json!({}),
                    },
                    ContentPart::ToolCall {
                        tool_call_id: None,
                        tool_name: "write".to_string(),
                        input: serde_json::json!({}),
                    },
                ],
            },
        ];

        let index = index_tool_calls(&messages);
        
        assert_eq!(index.ordered_keys.len(), 2);
        assert_eq!(index.ordered_keys[0], "id:call_1");
        assert_eq!(index.ordered_keys[1], "anon:0");
    }

    #[test]
    fn test_index_tool_calls_empty() {
        let messages = vec![
            MessageContent {
                role: MessageRole::User,
                content: vec![
                    ContentPart::Text { text: "Hello".to_string() },
                ],
            },
        ];

        let index = index_tool_calls(&messages);
        
        assert!(index.ordered_keys.is_empty());
        assert!(index.by_location.is_empty());
    }
}
```

**Step 4: Run tests**

Run: `cargo test --lib context::tool_compaction -v`
Expected: PASS (all 3 tests)

**Step 5: Commit**

```bash
git add src/context/tool_compaction.rs
git commit -m "feat(context): implement index_tool_calls function"
```

---

### Task 3: Implement findPendingCompactionCandidates

**Objective:** Find tool calls/results that are not in recent keys and not already compacted

**Files:**
- Modify: `src/context/tool_compaction.rs`

**Step 1: Add helper functions**

```rust
/// Check if a part is a tool call
fn is_tool_call_part(part: &ContentPart) -> bool {
    matches!(part, ContentPart::ToolCall { .. })
}

/// Check if a part is a tool result  
fn is_tool_result_part(part: &ContentPart) -> bool {
    matches!(part, ContentPart::ToolResult { .. })
}

/// Check if a tool call part is already compacted
fn is_compacted_tool_call(part: &ContentPart, compacted_notice: &str) -> bool {
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

/// Check if a tool result part is already compacted
fn is_compacted_tool_result(part: &ContentPart, compacted_notice: &str) -> bool {
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
```

**Step 2: Add find_pending_compaction_candidates function**

```rust
/// Find tool calls/results that are candidates for compaction
pub fn find_pending_compaction_candidates(params: &FindPendingParams) -> PendingCompactionCandidates {
    let mut pending_tool_call_keys: HashSet<String> = HashSet::new();
    let mut pending_anonymous_tool_results: usize = 0;

    for (message_index, message) in params.messages.iter().enumerate() {
        let part_keys = params.tool_call_index.by_location.get(&message_index);

        for (part_index, part) in message.content.iter().enumerate() {
            if is_tool_call_part(part) {
                if let Some(key) = part_keys.and_then(|k| k.get(&part_index)) {
                    if !params.recent_tool_call_keys.contains(key)
                        && !is_compacted_tool_call(part, &params.compacted_notice)
                    {
                        pending_tool_call_keys.insert(key.clone());
                    }
                }
                continue;
            }

            if !is_tool_result_part(part) {
                continue;
            }

            // Tool result handling
            let tool_call_id = if let ContentPart::ToolResult { tool_call_id, .. } = part {
                tool_call_id.clone()
            } else {
                None
            };

            let key = tool_call_id.map(|id| format!("id:{}", id));

            if let Some(ref k) = key {
                if !params.recent_tool_call_keys.contains(k)
                    && !is_compacted_tool_result(part, &params.compacted_notice)
                {
                    pending_tool_call_keys.insert(k.clone());
                }
            } else if !is_compacted_tool_result(part, &params.compacted_notice) {
                pending_anonymous_tool_results += 1;
            }
        }
    }

    PendingCompactionCandidates {
        pending_tool_call_keys,
        pending_anonymous_tool_results,
    }
}

/// Parameters for find_pending_compaction_candidates
pub struct FindPendingParams<'a> {
    pub messages: &'a [MessageContent],
    pub tool_call_index: &'a ToolCallIndex,
    pub recent_tool_call_keys: &'a HashSet<String>,
    pub compacted_notice: &'a str,
}
```

**Step 3: Add tests**

```rust
#[test]
fn test_find_pending_compaction_candidates_with_recent() {
    let messages = vec![
        MessageContent {
            role: MessageRole::Assistant,
            content: vec![
                ContentPart::ToolCall {
                    tool_call_id: Some("call_1".to_string()),
                    tool_name: "bash".to_string(),
                    input: serde_json::json!({}),
                },
            ],
        },
    ];

    let index = index_tool_calls(&messages);
    let recent_keys: HashSet<String> = ["id:call_1".to_string()].into_iter().collect();

    let params = FindPendingParams {
        messages: &messages,
        tool_call_index: &index,
        recent_tool_call_keys: &recent_keys,
        compacted_notice: "[compacted]",
    };

    let result = find_pending_compaction_candidates(&params);
    
    assert!(result.pending_tool_call_keys.is_empty());
}

#[test]
fn test_find_pending_compaction_candidates_none_recent() {
    let messages = vec![
        MessageContent {
            role: MessageRole::Assistant,
            content: vec![
                ContentPart::ToolCall {
                    tool_call_id: Some("call_old".to_string()),
                    tool_name: "bash".to_string(),
                    input: serde_json::json!({}),
                },
            ],
        },
    ];

    let index = index_tool_calls(&messages);
    let recent_keys: HashSet<String> = HashSet::new();

    let params = FindPendingParams {
        messages: &messages,
        tool_call_index: &index,
        recent_tool_call_keys: &recent_keys,
        compacted_notice: "[compacted]",
    };

    let result = find_pending_compaction_candidates(&params);
    
    assert!(result.pending_tool_call_keys.contains("id:call_old"));
}
```

**Step 4: Run tests**

Run: `cargo test --lib context::tool_compaction -v`
Expected: PASS

**Step 5: Commit**

```bash
git add src/context/tool_compaction.rs
git commit -m "feat(context): implement find_pending_compaction_candidates"
```

---

## Phase 2: Compaction Logic

### Task 4: Implement compact functions

**Objective:** Create functions to compact tool calls and results to minimal representations

**Files:**
- Modify: `src/context/tool_compaction.rs`

**Step 1: Add compact helper functions**

```rust
/// Compact a tool call part to minimal representation
fn compact_tool_call(part: &ContentPart, compacted_notice: &str) -> ContentPart {
    if let ContentPart::ToolCall { tool_call_id, tool_name, input: _ } = part {
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

/// Compact a tool result part to minimal representation
fn compact_tool_result(part: &ContentPart, compacted_notice: &str) -> ContentPart {
    if let ContentPart::ToolResult { tool_call_id, output: _ } = part {
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
```

**Step 2: Add estimate_compaction_savings**

```rust
/// Estimate token savings from compaction (returns estimated tokens saved)
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
                        && !is_compacted_tool_call(part, params.compacted_notice)
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
                        && !is_compacted_tool_result(part, params.compacted_notice)
                    {
                        compacted_part = Some(compact_tool_result(part, params.compacted_notice));
                    }
                } else if params.pending_candidates.pending_anonymous_tool_results > 0
                    && !is_compacted_tool_result(part, params.compacted_notice)
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

pub struct EstimateSavingsParams<'a> {
    pub messages: &'a [MessageContent],
    pub tool_call_index: &'a ToolCallIndex,
    pub pending_candidates: &'a PendingCompactionCandidates,
    pub compacted_notice: &'a str,
}
```

**Step 3: Add compact_tool_data function**

```rust
/// Apply compaction to messages, returning new message list
pub fn compact_tool_data(params: &CompactToolDataParams) -> Vec<MessageContent> {
    params.messages
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
                                && !is_compacted_tool_call(part, params.compacted_notice)
                            {
                                changed = true;
                                return compact_tool_call(part, params.compacted_notice);
                            }
                        }
                    }

                    if is_tool_result_part(part) {
                        let tool_call_id = if let ContentPart::ToolResult { tool_call_id, .. } = part {
                            tool_call_id.clone()
                        } else {
                            None
                        };

                        let key = tool_call_id.map(|id| format!("id:{}", id));

                        if let Some(ref k) = key {
                            if params.pending_candidates.pending_tool_call_keys.contains(k)
                                && !is_compacted_tool_result(part, params.compacted_notice)
                            {
                                changed = true;
                                return compact_tool_result(part, params.compacted_notice);
                            }
                        }

                        if key.is_none()
                            && params.pending_candidates.pending_anonymous_tool_results > 0
                            && !is_compacted_tool_result(part, params.compacted_notice)
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

pub struct CompactToolDataParams<'a> {
    pub messages: &'a [MessageContent],
    pub tool_call_index: &'a ToolCallIndex,
    pub pending_candidates: &'a PendingCompactionCandidates,
    pub compacted_notice: &'a str,
}
```

**Step 4: Add get_pending_compaction_units**

```rust
/// Get total number of pending compaction units
pub fn get_pending_compaction_units(candidates: &PendingCompactionCandidates) -> usize {
    candidates.pending_tool_call_keys.len() + candidates.pending_anonymous_tool_results
}
```

**Step 5: Add integration tests**

```rust
#[test]
fn test_compact_tool_data_full_flow() {
    let messages = vec![
        // Message with tool call
        MessageContent {
            role: MessageRole::Assistant,
            content: vec![
                ContentPart::ToolCall {
                    tool_call_id: Some("call_1".to_string()),
                    tool_name: "bash".to_string(),
                    input: serde_json::json!({"command": "ls -la"}),
                },
            ],
        },
        // Message with tool result
        MessageContent {
            role: MessageRole::System,
            content: vec![
                ContentPart::ToolResult {
                    tool_call_id: Some("call_1".to_string()),
                    output: serde_json::json!({
                        "type": "text",
                        "value": "total 12\ndrwxr-xr-x  5 user  4096 Apr 18 10:00 ."
                    }),
                },
            ],
        },
    ];

    let index = index_tool_calls(&messages);
    let recent_keys: HashSet<String> = HashSet::new(); // No recent keys

    let params = FindPendingParams {
        messages: &messages,
        tool_call_index: &index,
        recent_tool_call_keys: &recent_keys,
        compacted_notice: "[compacted]",
    };

    let candidates = find_pending_compaction_candidates(&params);
    assert!(!candidates.pending_tool_call_keys.is_empty());

    // Now compact
    let compact_params = CompactToolDataParams {
        messages: &messages,
        tool_call_index: &index,
        pending_candidates: &candidates,
        compacted_notice: "[compacted]",
    };

    let compacted = compact_tool_data(&compact_params);
    
    // Verify compaction happened
    if let ContentPart::ToolCall { input, .. } = &compacted[0].content[0] {
        assert_eq!(input.get("compacted"), Some(&serde_json::json!(true)));
    } else {
        panic!("Expected tool call");
    }
}
```

**Step 6: Run tests**

Run: `cargo test --lib context::tool_compaction -v`
Expected: PASS

**Step 7: Commit**

```bash
git add src/context/tool_compaction.rs
git commit -m "feat(context): implement compaction logic and savings estimation"
```

---

## Phase 3: Integration

### Task 5: Add helper to get pending units count

**Objective:** Simple accessor for pending compaction units

**Files:**
- Modify: `src/context/tool_compaction.rs`

**Step 1: Add test for get_pending_compaction_units**

```rust
#[test]
fn test_get_pending_compaction_units() {
    let candidates = PendingCompactionCandidates {
        pending_tool_call_keys: ["id:1".to_string(), "id:2".to_string()].into_iter().collect(),
        pending_anonymous_tool_results: 3,
    };

    let units = get_pending_compaction_units(&candidates);
    assert_eq!(units, 5);
}
```

**Step 2: Run tests and commit**

Run: `cargo test --lib context::tool_compaction -v`
Expected: PASS

Commit: `git add src/context/tool_compaction.rs && git commit -m "feat(context): add get_pending_compaction_units helper"`

---

### Task 6: Verify full integration with context module

**Objective:** Ensure tool_compaction integrates with existing context structures

**Step 1: Run full test suite**

Run: `cargo test --lib`
Expected: ALL PASS (35+ tests)

**Step 2: Run clippy**

Run: `cargo clippy --lib -- -D warnings`
Expected: NO WARNINGS

**Step 3: Final commit**

```bash
git add -A
git commit -m "feat(context): complete tool call compaction implementation (TypeScript to Rust)"
```

---

## Summary

| Phase | Tasks | Description |
|-------|-------|-------------|
| 1 | 2 | Module structure, index_tool_calls, find_pending_compaction_candidates |
| 2 | 1 | Compaction logic (compact_tool_data, estimate_savings) |
| 3 | 2 | Integration tests, final verification |

**Total: 5 tasks**

Each task ~2-5 minutes. Run tests after each task. Commit after each task passes.

---

**Plan complete and saved.** Ready to execute using subagent-driven-development — I'll dispatch a fresh subagent per task with two-stage review (spec compliance then code quality). Shall I proceed?