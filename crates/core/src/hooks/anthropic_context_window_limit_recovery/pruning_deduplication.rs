use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use tokio;

use crate::hooks::anthropic_context_window_limit_recovery::pruning_types::{PruningState, ToolCallSignature, estimate_tokens};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeduplicationConfig {
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protected_tools: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ToolPart {
    #[serde(rename = "type")]
    part_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    state: Option<ToolState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ToolState {
    #[serde(skip_serializing_if = "Option::is_none")]
    input: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MessagePart {
    #[serde(rename = "type")]
    part_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parts: Option<Vec<ToolPart>>,
}

pub fn create_tool_signature(tool_name: &str, input: &Option<serde_json::Value>) -> String {
    let sorted_input = sort_object(input);
    format!("{}::{}", tool_name, serde_json::to_string(&sorted_input).unwrap_or_else(|_| "{}".to_string()))
}

fn sort_object(obj: &Option<serde_json::Value>) -> serde_json::Value {
    match obj {
        Some(value) => sort_value(value),
        None => serde_json::Value::Null,
    }
}

fn sort_value(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut sorted_map = serde_json::Map::new();
            let mut keys: Vec<String> = map.keys().cloned().collect();
            keys.sort();
            
            for key in keys {
                let sorted_value = sort_value(&map[&key]);
                sorted_map.insert(key, sorted_value);
            }
            serde_json::Value::Object(sorted_map)
        },
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(|v| sort_value(v)).collect())
        },
        _ => value.clone(),
    }
}

fn get_message_dir(session_id: &str) -> Option<String> {
    let message_storage = get_message_storage_path();
    
    if !std::path::Path::new(&message_storage).exists() {
        return None;
    }

    let direct_path = format!("{}/{}", message_storage, session_id);
    if std::path::Path::new(&direct_path).exists() {
        return Some(direct_path);
    }

    // ค้นหาในไดเรกทอรีย่อย
    if let Ok(entries) = std::fs::read_dir(&message_storage) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    let session_path = format!("{}/{}/{}", message_storage, entry.file_name().to_string_lossy(), session_id);
                    if std::path::Path::new(&session_path).exists() {
                        return Some(session_path);
                    }
                }
            }
        }
    }

    None
}

fn get_message_storage_path() -> String {
    // ใช้ path จำลองสำหรับการพัฒนา
    ".bl1nk/message".to_string()
}

async fn read_messages(session_id: &str) -> Vec<MessagePart> {
    let mut messages = Vec::new();
    
    if let Some(message_dir) = get_message_dir(session_id) {
        if let Ok(entries) = tokio::fs::read_dir(&message_dir).await {
            while let Ok(Some(entry)) = entries.recv().await {
                let file_name = entry.file_name();
                let file_str = file_name.to_string_lossy();
                
                if file_str.ends_with(".json") {
                    let file_path = format!("{}/{}", message_dir, file_str);
                    
                    if let Ok(content) = tokio::fs::read_to_string(&file_path).await {
                        if let Ok(data) = serde_json::from_str::<MessagePart>(&content) {
                            if data.parts.is_some() {
                                messages.push(data);
                            }
                        }
                    }
                }
            }
        }
    }
    
    messages
}

pub async fn execute_deduplication(
    session_id: &str,
    state: &mut PruningState,
    config: &DeduplicationConfig,
    protected_tools: &HashSet<String>,
) -> usize {
    if !config.enabled {
        return 0;
    }

    let messages = read_messages(session_id).await;
    let mut signatures: HashMap<String, Vec<ToolCallSignature>> = HashMap::new();

    let mut current_turn = 0;

    for msg in &messages {
        if let Some(parts) = &msg.parts {
            for part in parts {
                if part.part_type == "step-start" {
                    current_turn += 1;
                    continue;
                }

                if part.part_type != "tool" || part.call_id.is_none() || part.tool.is_none() {
                    continue;
                }

                let call_id = part.call_id.as_ref().unwrap();
                let tool_name = part.tool.as_ref().unwrap();

                if protected_tools.contains(tool_name) {
                    continue;
                }

                if let Some(ref protected_tools_list) = config.protected_tools {
                    if protected_tools_list.contains(tool_name) {
                        continue;
                    }
                }

                if state.is_tool_to_prune(call_id) {
                    continue;
                }

                let signature = create_tool_signature(tool_name, &part.state.as_ref().and_then(|s| s.input.as_ref().cloned()));
                
                signatures.entry(signature.clone()).or_insert_with(Vec::new).push(ToolCallSignature {
                    tool_name: tool_name.clone(),
                    signature: signature.clone(),
                    call_id: call_id.clone(),
                    turn: current_turn,
                });

                state.add_tool_signature(session_id.to_string(), ToolCallSignature {
                    tool_name: tool_name.clone(),
                    signature: signature.clone(),
                    call_id: call_id.clone(),
                    turn: current_turn,
                });
            }
        }
    }

    let mut pruned_count = 0;
    let mut tokens_saved = 0;

    for (signature, calls) in &signatures {
        if calls.len() > 1 {
            // เก็บเฉพาะอันสุดท้ายไว้ ตัดอันอื่นทิ้ง
            let to_prune = &calls[..calls.len()-1];

            for call in to_prune {
                state.add_tool_to_prune(call.call_id.clone());
                pruned_count += 1;

                if let Some(output) = find_tool_output(&messages, &call.call_id).await {
                    tokens_saved += estimate_tokens(&output);
                }

                log::info!("[pruning-deduplication] pruned duplicate. tool={}, callID={}, turn={}, signature={}", 
                          &call.tool_name,
                          &call.call_id,
                          call.turn,
                          &signature.chars().take(100).collect::<String>());
            }
        }
    }

    log::info!("[pruning-deduplication] complete. prunedCount={}, tokensSaved={}, uniqueSignatures={}", 
              pruned_count,
              tokens_saved,
              signatures.len());

    pruned_count
}

async fn find_tool_output(messages: &[MessagePart], call_id: &str) -> Option<String> {
    for msg in messages {
        if let Some(parts) = &msg.parts {
            for part in parts {
                if part.part_type == "tool" && 
                   part.call_id.as_ref().map(|id| id == call_id).unwrap_or(false) &&
                   part.state.as_ref().and_then(|s| s.output.as_ref()).is_some() {
                    return part.state.as_ref().unwrap().output.clone();
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_tool_signature() {
        let input = Some(serde_json::json!({"file": "test.txt", "content": "hello"}));
        let signature = create_tool_signature("edit", &input);
        
        // ตรวจสอบว่า signature มีรูปแบบถูกต้อง
        assert!(signature.starts_with("edit::"));
        assert!(signature.contains("content"));
        assert!(signature.contains("hello"));
    }
}