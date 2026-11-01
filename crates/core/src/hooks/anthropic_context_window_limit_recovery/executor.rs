use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::hooks::anthropic_context_window_limit_recovery::types::{AutoCompactState, ParsedTokenLimitError, RETRY_CONFIG, TRUNCATE_CONFIG};
use crate::hooks::anthropic_context_window_limit_recovery::storage::{find_largest_tool_result, truncate_tool_result, truncate_until_target_tokens, count_truncated_results};
use crate::hooks::session_recovery::storage::{find_empty_messages, find_empty_message_by_index, inject_text_part, replace_empty_text_parts};

const PLACEHOLDER_TEXT: &str = "[user interrupted]";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentalConfig {
    // ต้องกำหนดโครงสร้างตามที่ใช้งานจริง
}

#[derive(Debug, Clone)]
pub struct Client {
    // จำลองโครงสร้าง client สำหรับการใช้งาน
}

fn get_or_create_retry_state<'a>(
    auto_compact_state: &'a mut AutoCompactState,
    session_id: &str,
) -> &'a mut crate::hooks::anthropic_context_window_limit_recovery::types::RetryState {
    auto_compact_state.retry_state_by_session
        .entry(session_id.to_string())
        .or_insert_with(|| crate::hooks::anthropic_context_window_limit_recovery::types::RetryState {
            attempt: 0,
            last_attempt_time: 0,
        })
}

fn get_or_create_truncate_state<'a>(
    auto_compact_state: &'a mut AutoCompactState,
    session_id: &str,
) -> &'a mut crate::hooks::anthropic_context_window_limit_recovery::types::TruncateState {
    auto_compact_state.truncate_state_by_session
        .entry(session_id.to_string())
        .or_insert_with(|| crate::hooks::anthropic_context_window_limit_recovery::types::TruncateState {
            truncate_attempt: 0,
            last_truncated_part_id: None,
        })
}

fn sanitize_empty_messages_before_summarize(session_id: &str) -> usize {
    let empty_message_ids = find_empty_messages(session_id);
    if empty_message_ids.is_empty() {
        return 0;
    }

    let mut fixed_count = 0;
    for message_id in empty_message_ids {
        let replaced = replace_empty_text_parts(&message_id, PLACEHOLDER_TEXT);
        if replaced {
            fixed_count += 1;
        } else {
            let injected = inject_text_part(session_id, &message_id, PLACEHOLDER_TEXT);
            if injected {
                fixed_count += 1;
            }
        }
    }

    if fixed_count > 0 {
        log::info!("[auto-compact] pre-summarize sanitization fixed empty messages. sessionID={}, fixedCount={}, totalEmpty={}", 
                   session_id, fixed_count, empty_message_ids.len());
    }

    fixed_count
}

fn format_bytes(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{}B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1}KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1}MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

pub async fn get_last_assistant(
    session_id: &str,
    _client: &Client,
    _directory: &str,
) -> Option<HashMap<String, serde_json::Value>> {
    // ฟังก์ชันนี้ต้องการการเชื่อมต่อกับ API จริง
    // สำหรับตอนนี้เราจะคืนค่า None
    None
}

fn clear_session_state(
    auto_compact_state: &mut AutoCompactState,
    session_id: &str,
) {
    auto_compact_state.remove_pending_compact(session_id);
    auto_compact_state.error_data_by_session.remove(session_id);
    auto_compact_state.retry_state_by_session.remove(session_id);
    auto_compact_state.truncate_state_by_session.remove(session_id);
    auto_compact_state.empty_content_attempt_by_session.remove(session_id);
    auto_compact_state.remove_compaction_in_progress(session_id);
}

fn get_or_create_empty_content_attempt(
    auto_compact_state: &AutoCompactState,
    session_id: &str,
) -> u32 {
    *auto_compact_state.empty_content_attempt_by_session.get(session_id).unwrap_or(&0)
}

async fn fix_empty_messages(
    session_id: &str,
    auto_compact_state: &mut AutoCompactState,
    _client: &Client,
    message_index: Option<u32>,
) -> bool {
    let attempt = get_or_create_empty_content_attempt(auto_compact_state, session_id);
    auto_compact_state.set_empty_content_attempt(session_id.to_string(), attempt + 1);

    let mut fixed = false;
    let mut fixed_message_ids = Vec::new();

    if let Some(index) = message_index {
        if let Some(target_message_id) = find_empty_message_by_index(session_id, index) {
            let replaced = replace_empty_text_parts(&target_message_id, "[user interrupted]");
            if replaced {
                fixed = true;
                fixed_message_ids.push(target_message_id);
            } else {
                let injected = inject_text_part(session_id, &target_message_id, "[user interrupted]");
                if injected {
                    fixed = true;
                    fixed_message_ids.push(target_message_id);
                }
            }
        }
    }

    if !fixed {
        let empty_message_ids = find_empty_messages(session_id);
        if empty_message_ids.is_empty() {
            log::error!("No empty messages found in storage. Cannot auto-recover.");
            return false;
        }

        for message_id in empty_message_ids {
            let replaced = replace_empty_text_parts(&message_id, "[user interrupted]");
            if replaced {
                fixed = true;
                fixed_message_ids.push(message_id);
            } else {
                let injected = inject_text_part(session_id, &message_id, "[user interrupted]");
                if injected {
                    fixed = true;
                    fixed_message_ids.push(message_id);
                }
            }
        }
    }

    if fixed {
        log::warn!("Session Recovery: {}", format!("Fixed {} empty message(s). Retrying...", fixed_message_ids.len()));
    }

    fixed
}

pub async fn execute_compact(
    session_id: &str,
    msg: &HashMap<String, serde_json::Value>,
    auto_compact_state: &mut AutoCompactState,
    _client: &Client,
    directory: &str,
    _experimental: Option<&ExperimentalConfig>,
) {
    if auto_compact_state.is_compaction_in_progress(session_id) {
        log::warn!("Compact In Progress: Recovery already running. Please wait or start new session if stuck.");
        return;
    }
    
    auto_compact_state.add_compaction_in_progress(session_id.to_string());

    // ใช้ block เพื่อให้สามารถใช้ finally ได้
    let result = async {
        let error_data = auto_compact_state.get_error_data(session_id).cloned();
        let mut truncate_state = get_or_create_truncate_state(auto_compact_state, session_id).clone();

        let is_over_limit = if let Some(ref error_data) = error_data {
            error_data.current_tokens > error_data.max_tokens
        } else {
            false
        };

        // Aggressive Truncation - ลองเสมอเมื่อเกินขีดจำกัด
        if is_over_limit && truncate_state.truncate_attempt < TRUNCATE_CONFIG.max_truncate_attempts as u32 {
            log::info!("[auto-compact] PHASE 2: aggressive truncation triggered. currentTokens={}, maxTokens={}, targetRatio={}", 
                       error_data.as_ref().map(|d| d.current_tokens).unwrap_or(0),
                       error_data.as_ref().map(|d| d.max_tokens).unwrap_or(0),
                       TRUNCATE_CONFIG.target_token_ratio);

            if let Some(ref error_data) = error_data {
                let aggressive_result = truncate_until_target_tokens(
                    session_id,
                    error_data.current_tokens as usize,
                    error_data.max_tokens as usize,
                    Some(TRUNCATE_CONFIG.target_token_ratio),
                    Some(TRUNCATE_CONFIG.chars_per_token),
                ).await;

                if aggressive_result.truncated_count > 0 {
                    truncate_state.truncate_attempt += aggressive_result.truncated_count as u32;

                    let tool_names: Vec<String> = aggressive_result.truncated_tools
                        .iter()
                        .map(|t| t.tool_name.clone())
                        .collect();
                    let status_msg = if aggressive_result.sufficient {
                        format!("Truncated {} outputs ({})", 
                                aggressive_result.truncated_count, 
                                format_bytes(aggressive_result.total_bytes_removed))
                    } else {
                        format!("Truncated {} outputs ({}) - continuing to summarize...", 
                                aggressive_result.truncated_count, 
                                format_bytes(aggressive_result.total_bytes_removed))
                    };

                    log::info!("Truncation result: {} - {} (Tools: {})", 
                               if aggressive_result.sufficient { "Truncation Complete" } else { "Partial Truncation" },
                               &status_msg,
                               &tool_names.join(", "));

                    log::info!("[auto-compact] aggressive truncation completed. result={:?}", aggressive_result);

                    // คืนค่าเร็วถ้า truncation สำเร็จ
                    if aggressive_result.sufficient {
                        clear_session_state(auto_compact_state, session_id);
                        // จำลองการส่ง prompt ใหม่หลังจาก truncation
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                        return Ok(());
                    }
                    
                    log::info!("[auto-compact] truncation insufficient, falling through to summarize. sessionID={}, truncatedCount={}, sufficient={}", 
                               session_id, aggressive_result.truncated_count, aggressive_result.sufficient);
                }
            }
        }

        // PHASE 3: Summarize - ใช้เมื่อ truncation ไม่เพียงพอหรือไม่มีผลลัพธ์ของเครื่องมือ
        let mut retry_state = get_or_create_retry_state(auto_compact_state, session_id).clone();

        if let Some(ref error_data) = error_data {
            if error_data.error_type.contains("non-empty content") {
                let attempt = get_or_create_empty_content_attempt(auto_compact_state, session_id);
                if attempt < 3 {
                    let fixed = fix_empty_messages(
                        session_id,
                        auto_compact_state,
                        _client,
                        error_data.message_index,
                    ).await;
                    if fixed {
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                        // จำลองการเรียก execute_compact ซ้ำ
                        return Ok(());
                    }
                } else {
                    log::error!("Recovery Failed: Max recovery attempts (3) reached for empty content error. Please start a new session.");
                    return Ok(());
                }
            }
        }

        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        if current_time - retry_state.last_attempt_time > 300000 {  // 5 นาที
            retry_state.attempt = 0;
            auto_compact_state.truncate_state_by_session.remove(session_id);
        }

        if retry_state.attempt < RETRY_CONFIG.max_attempts {
            retry_state.attempt += 1;
            retry_state.last_attempt_time = current_time;

            let provider_id = msg.get("providerID").and_then(|v| v.as_str()).unwrap_or("");
            let model_id = msg.get("modelID").and_then(|v| v.as_str()).unwrap_or("");

            if !provider_id.is_empty() && !model_id.is_empty() {
                sanitize_empty_messages_before_summarize(session_id);

                log::warn!("Auto Compact: Summarizing session (attempt {}/{})...", retry_state.attempt, RETRY_CONFIG.max_attempts);

                // จำลองการเรียก summarize API
                // ในระบบจริงจะต้องมีการเรียก API ที่เหมาะสม
                
                // ถ้าเกิด error ให้หน่วงเวลาแล้วลองใหม่
                let delay = (RETRY_CONFIG.initial_delay_ms as f64 * 
                            RETRY_CONFIG.backoff_factor.powf((retry_state.attempt - 1) as f64)) as u64;
                let capped_delay = delay.min(RETRY_CONFIG.max_delay_ms);
                
                if capped_delay > 0 {
                    tokio::time::sleep(tokio::time::Duration::from_millis(capped_delay)).await;
                }
                
                return Ok(());
            } else {
                log::warn!("Summarize Skipped: Missing providerID or modelID.");
            }
        }

        clear_session_state(auto_compact_state, session_id);

        log::error!("Auto Compact Failed: All recovery attempts failed. Please start a new session.");

        Ok(())
    }.await;

    // ทำหน้าที่ finally
    auto_compact_state.remove_compaction_in_progress(session_id);
    
    if let Err(e) = result {
        log::error!("Error during execute_compact: {}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500B");
        assert_eq!(format_bytes(2048), "2.0KB");
        assert_eq!(format_bytes(2097152), "2.0MB");
    }
}