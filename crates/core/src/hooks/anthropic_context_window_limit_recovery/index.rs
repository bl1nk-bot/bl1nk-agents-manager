use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde_json::Value;

pub use crate::hooks::anthropic_context_window_limit_recovery::types::{AutoCompactState, ParsedTokenLimitError};
pub use crate::hooks::anthropic_context_window_limit_recovery::parser::parse_anthropic_token_limit_error;
pub use crate::hooks::anthropic_context_window_limit_recovery::executor::{execute_compact, get_last_assistant};

#[derive(Debug, Clone)]
pub struct ExperimentalConfig {
    // ต้องกำหนดโครงสร้างตามที่ใช้งานจริง
}

#[derive(Debug, Clone)]
pub struct AnthropicContextWindowLimitRecoveryHook {
    auto_compact_state: Arc<RwLock<AutoCompactState>>,
    experimental: Option<ExperimentalConfig>,
}

impl AnthropicContextWindowLimitRecoveryHook {
    pub fn new(experimental: Option<ExperimentalConfig>) -> Self {
        Self {
            auto_compact_state: Arc::new(RwLock::new(AutoCompactState::new())),
            experimental,
        }
    }

    pub async fn handle_event(&self, event_type: &str, properties: Option<&HashMap<String, Value>>) {
        let props = if let Some(p) = properties { p } else { return; };

        match event_type {
            "session.deleted" => {
                if let Some(info_val) = props.get("info") {
                    if let Some(info_obj) = info_val.as_object() {
                        if let Some(Value::String(session_id)) = info_obj.get("id") {
                            let mut state = self.auto_compact_state.write().await;
                            state.remove_pending_compact(session_id);
                            state.error_data_by_session.remove(session_id);
                            state.retry_state_by_session.remove(session_id);
                            state.truncate_state_by_session.remove(session_id);
                            state.empty_content_attempt_by_session.remove(session_id);
                            state.remove_compaction_in_progress(session_id);
                        }
                    }
                }
            }
            
            "session.error" => {
                if let Some(Value::String(session_id)) = props.get("sessionID") {
                    log::info!("[auto-compact] session.error received. sessionID={}, error={:?}", 
                               session_id, props.get("error"));

                    let parsed = parse_anthropic_token_limit_error(props.get("error"));
                    log::info!("[auto-compact] parsed result. parsed={:?}, hasError={}", 
                               parsed, props.get("error").is_some());

                    if let Some(parsed_error) = parsed {
                        let mut state = self.auto_compact_state.write().await;
                        state.add_pending_compact(session_id.clone());
                        state.set_error_data(session_id.clone(), parsed_error.clone());

                        if state.is_compaction_in_progress(session_id) {
                            return;
                        }

                        // ดึงข้อมูล assistant คนสุดท้าย
                        // ต้องใช้ client จริงในการดึงข้อมูล
                        let last_assistant = get_last_assistant(session_id, &(), "").await;
                        let provider_id = parsed_error.provider_id.or_else(|| {
                            last_assistant.as_ref().and_then(|la| la.get("providerID").and_then(|v| v.as_str()).map(|s| s.to_string()))
                        });
                        let model_id = parsed_error.model_id.or_else(|| {
                            last_assistant.as_ref().and_then(|la| la.get("modelID").and_then(|v| v.as_str()).map(|s| s.to_string()))
                        });

                        // แสดง toast notification
                        log::warn!("Context Limit Hit: Truncating large tool outputs and recovering...");

                        // เรียก execute_compact หลังจากหน่วงเวลา
                        let state_clone = self.auto_compact_state.clone();
                        let session_id_clone = session_id.clone();
                        let provider_id_clone = provider_id.clone();
                        let model_id_clone = model_id.clone();
                        let experimental_clone = self.experimental.clone();

                        tokio::spawn(async move {
                            tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
                            
                            let msg = {
                                let mut m = HashMap::new();
                                if let Some(pid) = provider_id_clone {
                                    m.insert("providerID".to_string(), Value::String(pid));
                                }
                                if let Some(mid) = model_id_clone {
                                    m.insert("modelID".to_string(), Value::String(mid));
                                }
                                m
                            };
                            
                            let mut state = state_clone.write().await;
                            execute_compact(
                                &session_id_clone,
                                &msg,
                                &mut state,
                                &(), // ต้องใช้ client จริง
                                "", // ต้องใช้ directory จริง
                                experimental_clone.as_ref(),
                            ).await;
                        });
                    }
                }
            }
            
            "message.updated" => {
                if let Some(info_val) = props.get("info") {
                    if let Some(info_obj) = info_val.as_object() {
                        if let Some(Value::String(session_id)) = info_obj.get("sessionID") {
                            if let Some(Value::String(role)) = info_obj.get("role") {
                                if role == "assistant" && info_obj.contains_key("error") {
                                    log::info!("[auto-compact] message.updated with error. sessionID={}, error={:?}", 
                                              session_id, info_obj.get("error"));

                                    let parsed = parse_anthropic_token_limit_error(info_obj.get("error"));
                                    log::info!("[auto-compact] message.updated parsed result. parsed={:?}", parsed);

                                    if let Some(mut parsed_error) = parsed {
                                        if let Some(provider_id) = info_obj.get("providerID").and_then(|v| v.as_str()) {
                                            parsed_error.provider_id = Some(provider_id.to_string());
                                        }
                                        if let Some(model_id) = info_obj.get("modelID").and_then(|v| v.as_str()) {
                                            parsed_error.model_id = Some(model_id.to_string());
                                        }

                                        let mut state = self.auto_compact_state.write().await;
                                        state.add_pending_compact(session_id.clone());
                                        state.set_error_data(session_id.clone(), parsed_error);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            "session.idle" => {
                if let Some(Value::String(session_id)) = props.get("sessionID") {
                    let state = self.auto_compact_state.read().await;
                    if !state.is_pending_compact(session_id) {
                        return;
                    }

                    let error_data = state.get_error_data(session_id).cloned();
                    let last_assistant = get_last_assistant(session_id, &(), "").await;

                    if let Some(ref la) = last_assistant {
                        if let Some(summary_val) = la.get("summary") {
                            if summary_val.as_bool() == Some(true) {
                                let mut state = self.auto_compact_state.write().await;
                                state.remove_pending_compact(session_id);
                                return;
                            }
                        }
                    }

                    let provider_id = error_data.as_ref()
                        .and_then(|ed| ed.provider_id.clone())
                        .or_else(|| last_assistant.as_ref().and_then(|la| la.get("providerID").and_then(|v| v.as_str()).map(|s| s.to_string())));
                    let model_id = error_data.as_ref()
                        .and_then(|ed| ed.model_id.clone())
                        .or_else(|| last_assistant.as_ref().and_then(|la| la.get("modelID").and_then(|v| v.as_str()).map(|s| s.to_string())));

                    log::warn!("Auto Compact: Token limit exceeded. Attempting recovery...");

                    let mut state = self.auto_compact_state.write().await;
                    let msg = {
                        let mut m = HashMap::new();
                        if let Some(pid) = provider_id {
                            m.insert("providerID".to_string(), Value::String(pid));
                        }
                        if let Some(mid) = model_id {
                            m.insert("modelID".to_string(), Value::String(mid));
                        }
                        m
                    };

                    execute_compact(
                        session_id,
                        &msg,
                        &mut state,
                        &(), // ต้องใช้ client จริง
                        "", // ต้องใช้ directory จริง
                        self.experimental.as_ref(),
                    ).await;
                }
            }
            
            _ => {
                // ไม่สนใจ event อื่น
            }
        }
    }
}

// ฟังก์ชัน helper สำหรับสร้าง hook
pub fn create_anthropic_context_window_limit_recovery_hook(
    _ctx: &(), // ต้องใช้ context จริง
    options: Option<AnthropicContextWindowLimitRecoveryOptions>
) -> AnthropicContextWindowLimitRecoveryHook {
    let experimental = options.and_then(|opt| opt.experimental);
    AnthropicContextWindowLimitRecoveryHook::new(experimental)
}

#[derive(Debug, Clone)]
pub struct AnthropicContextWindowLimitRecoveryOptions {
    pub experimental: Option<ExperimentalConfig>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_hook() {
        let hook = AnthropicContextWindowLimitRecoveryHook::new(None);
        assert!(hook.auto_compact_state.read().await.error_data_by_session.is_empty());
    }
}