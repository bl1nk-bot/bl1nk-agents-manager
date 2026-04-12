use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio;
use std::fs;
use std::io::Write;

const TRUNCATION_MESSAGE: &str = 
    "[TOOL RESULT TRUNCATED - Context limit exceeded. Original output was too large and has been truncated to recover the session. Please re-run this tool if you need the full output.]";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredToolPart {
    pub id: String,
    pub session_id: String,
    pub message_id: String,
    #[serde(rename = "type")]
    pub part_type: String,
    pub call_id: String,
    pub tool: String,
    pub state: ToolState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_size: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolState {
    pub status: String, // "pending", "running", "completed", "error"
    pub input: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<ToolTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolTime {
    pub start: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compacted: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResultInfo {
    pub part_path: String,
    pub part_id: String,
    pub message_id: String,
    pub tool_name: String,
    pub output_size: usize,
}

// ฟังก์ชันช่วยเหลือสำหรับการได้รับไดเรกทอรีเก็บข้อมูล
fn get_message_dir(session_id: &str) -> Option<String> {
    let storage_dir = get_opencode_storage_dir();
    let message_storage = format!("{}/message", storage_dir);
    
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

fn get_opencode_storage_dir() -> String {
    // ใช้ไดเรกทอรี home หรือ temp สำหรับจำลอง
    if cfg!(windows) {
        std::env::var("APPDATA").unwrap_or_else(|_| "/tmp".to_string())
    } else {
        std::env::var("HOME").map(|h| format!("{}/.bl1nk", h)).unwrap_or_else(|_| "/tmp/.bl1nk".to_string())
    }
}

fn get_message_ids(session_id: &str) -> Vec<String> {
    let mut message_ids = Vec::new();
    
    if let Some(message_dir) = get_message_dir(session_id) {
        if let Ok(entries) = std::fs::read_dir(&message_dir) {
            for entry in entries.flatten() {
                let file_name = entry.file_name();
                let file_str = file_name.to_string_lossy();
                
                if file_str.ends_with(".json") {
                    let message_id = file_str.strip_suffix(".json").unwrap_or(&file_str).to_string();
                    message_ids.push(message_id);
                }
            }
        }
    }
    
    message_ids
}

pub async fn find_tool_results_by_size(session_id: &str) -> Vec<ToolResultInfo> {
    let message_ids = get_message_ids(session_id);
    let mut results = Vec::new();

    for message_id in message_ids {
        let part_dir = format!(".bl1nk/part/{}", message_id); // ใช้ path จำลอง
        if !std::path::Path::new(&part_dir).exists() {
            continue;
        }

        if let Ok(entries) = std::fs::read_dir(&part_dir) {
            for entry in entries.flatten() {
                let file_name = entry.file_name();
                let file_str = file_name.to_string_lossy();
                
                if file_str.ends_with(".json") {
                    let part_path = format!("{}/{}", part_dir, file_str);
                    
                    if let Ok(content) = tokio::fs::read_to_string(&part_path).await {
                        if let Ok(part) = serde_json::from_str::<StoredToolPart>(&content) {
                            if part.part_type == "tool" && 
                               part.state.output.is_some() && 
                               part.truncated.unwrap_or(false) == false {
                                results.push(ToolResultInfo {
                                    part_path: part_path.clone(),
                                    part_id: part.id.clone(),
                                    message_id: message_id.clone(),
                                    tool_name: part.tool.clone(),
                                    output_size: part.state.output.as_ref().map(|s| s.len()).unwrap_or(0),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // เรียงลำดับตามขนาดของ output จากมากไปน้อย
    results.sort_by(|a, b| b.output_size.cmp(&a.output_size));
    results
}

pub async fn find_largest_tool_result(session_id: &str) -> Option<ToolResultInfo> {
    let results = find_tool_results_by_size(session_id).await;
    results.first().cloned()
}

pub async fn truncate_tool_result(part_path: &str) -> Result<(String, usize), String> {
    if let Ok(content) = tokio::fs::read_to_string(part_path).await {
        if let Ok(mut part) = serde_json::from_str::<StoredToolPart>(&content) {
            if let Some(ref output) = part.state.output {
                let original_size = output.len();
                let tool_name = part.tool.clone();

                part.truncated = Some(true);
                part.original_size = Some(original_size);
                part.state.output = Some(TRUNCATION_MESSAGE.to_string());

                if part.state.time.is_none() {
                    part.state.time = Some(ToolTime {
                        start: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as u64,
                        end: None,
                        compacted: Some(std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as u64),
                    });
                } else {
                    part.state.time.as_mut().unwrap().compacted = Some(std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64);
                }

                let updated_content = serde_json::to_string_pretty(&part).map_err(|e| e.to_string())?;
                
                tokio::fs::write(part_path, updated_content).await.map_err(|e| e.to_string())?;
                
                return Ok((tool_name, original_size));
            }
        }
    }
    
    Err("Failed to truncate tool result".to_string())
}

pub async fn get_total_tool_output_size(session_id: &str) -> usize {
    let results = find_tool_results_by_size(session_id).await;
    results.iter().map(|r| r.output_size).sum()
}

pub async fn count_truncated_results(session_id: &str) -> usize {
    let message_ids = get_message_ids(session_id);
    let mut count = 0;

    for message_id in message_ids {
        let part_dir = format!(".bl1nk/part/{}", message_id); // ใช้ path จำลอง
        if !std::path::Path::new(&part_dir).exists() {
            continue;
        }

        if let Ok(entries) = std::fs::read_dir(&part_dir) {
            for entry in entries.flatten() {
                let file_name = entry.file_name();
                let file_str = file_name.to_string_lossy();
                
                if file_str.ends_with(".json") {
                    let full_path = format!("{}/{}", part_dir, file_str);
                    
                    if let Ok(content) = tokio::fs::read_to_string(&full_path).await {
                        if let Ok(part) = serde_json::from_str::<StoredToolPart>(&content) {
                            if part.truncated == Some(true) {
                                count += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    count
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggressiveTruncateResult {
    pub success: bool,
    pub sufficient: bool,
    pub truncated_count: usize,
    pub total_bytes_removed: usize,
    pub target_bytes_to_remove: usize,
    pub truncated_tools: Vec<TruncatedTool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TruncatedTool {
    pub tool_name: String,
    pub original_size: usize,
}

pub async fn truncate_until_target_tokens(
    session_id: &str,
    current_tokens: usize,
    max_tokens: usize,
    target_ratio: Option<f64>,
    chars_per_token: Option<usize>,
) -> AggressiveTruncateResult {
    let target_ratio = target_ratio.unwrap_or(0.8);
    let chars_per_token = chars_per_token.unwrap_or(4);
    
    let target_tokens = (max_tokens as f64 * target_ratio) as usize;
    let tokens_to_reduce = current_tokens.saturating_sub(target_tokens);
    let chars_to_reduce = tokens_to_reduce * chars_per_token;

    if tokens_to_reduce == 0 {
        return AggressiveTruncateResult {
            success: true,
            sufficient: true,
            truncated_count: 0,
            total_bytes_removed: 0,
            target_bytes_to_remove: 0,
            truncated_tools: vec![],
        };
    }

    let results = find_tool_results_by_size(session_id).await;

    if results.is_empty() {
        return AggressiveTruncateResult {
            success: false,
            sufficient: false,
            truncated_count: 0,
            total_bytes_removed: 0,
            target_bytes_to_remove: chars_to_reduce,
            truncated_tools: vec![],
        };
    }

    let mut total_removed = 0;
    let mut truncated_count = 0;
    let mut truncated_tools = Vec::new();

    for result in results {
        match truncate_tool_result(&result.part_path).await {
            Ok((tool_name, original_size)) => {
                truncated_count += 1;
                total_removed += original_size;
                truncated_tools.push(TruncatedTool {
                    tool_name,
                    original_size,
                });

                if total_removed >= chars_to_reduce {
                    break;
                }
            }
            Err(_) => continue,
        }
    }

    let sufficient = total_removed >= chars_to_reduce;

    AggressiveTruncateResult {
        success: truncated_count > 0,
        sufficient,
        truncated_count,
        total_bytes_removed: total_removed,
        target_bytes_to_remove: chars_to_reduce,
        truncated_tools,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_find_tool_results_by_size() {
        // ต้องมีข้อมูลจริงในระบบไฟล์เพื่อทดสอบ
        // สำหรับตอนนี้เราจะข้ามการทดสอบนี้ไป
    }
}