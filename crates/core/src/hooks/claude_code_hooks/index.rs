use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde_json::Value;

use crate::hooks::claude_code_hooks::config::load_claude_hooks_config;
use crate::hooks::claude_code_hooks::config_loader::{load_plugin_extended_config, PluginExtendedConfig};
use crate::hooks::claude_code_hooks::pre_tool_use::{execute_pre_tool_use_hooks, PreToolUseContext};
use crate::hooks::claude_code_hooks::post_tool_use::{execute_post_tool_use_hooks, PostToolUseContext};
use crate::hooks::claude_code_hooks::user_prompt_submit::{execute_user_prompt_submit_hooks, UserPromptSubmitContext};
use crate::hooks::claude_code_hooks::stop::{execute_stop_hooks, StopContext};
use crate::hooks::claude_code_hooks::pre_compact::{execute_pre_compact_hooks, PreCompactContext};
use crate::hooks::claude_code_hooks::tool_input_cache::{ToolInputCache, get_tool_input};
use crate::hooks::claude_code_hooks::transcript::{record_tool_use, record_tool_result, get_transcript_path, record_user_message};
use crate::hooks::claude_code_hooks::types::PluginConfig;

// สถานะของ session ต่างๆ
lazy_static::lazy_static! {
    static ref SESSION_FIRST_MESSAGE_PROCESSED: Arc<RwLock<std::collections::HashSet<String>>> = 
        Arc::new(RwLock::new(std::collections::HashSet::new()));
    static ref SESSION_ERROR_STATE: Arc<RwLock<HashMap<String, SessionErrorState>>> = 
        Arc::new(RwLock::new(HashMap::new()));
    static ref SESSION_INTERRUPT_STATE: Arc<RwLock<HashMap<String, SessionInterruptState>>> = 
        Arc::new(RwLock::new(HashMap::new()));
}

#[derive(Debug, Clone)]
struct SessionErrorState {
    has_error: bool,
    error_message: Option<String>,
}

#[derive(Debug, Clone)]
struct SessionInterruptState {
    interrupted: bool,
}

#[derive(Debug, Clone)]
pub struct ClaudeCodeHooksHook {
    directory: String,
    config: PluginConfig,
    tool_input_cache: ToolInputCache,
}

impl ClaudeCodeHooksHook {
    pub fn new(directory: String, config: PluginConfig) -> Self {
        Self {
            directory,
            config,
            tool_input_cache: ToolInputCache::default(),
        }
    }

    pub async fn on_experimental_session_compacting(&self, session_id: &str) -> Vec<String> {
        // ตรวจสอบว่า hook ถูกปิดใช้งานหรือไม่
        if is_hook_disabled(&self.config, "PreCompact").await {
            return vec![];
        }

        let claude_config = load_claude_hooks_config(None).await;
        let extended_config = load_plugin_extended_config().await;

        let pre_compact_ctx = PreCompactContext {
            session_id: session_id.to_string(),
            cwd: self.directory.clone(),
        };

        let result = execute_pre_compact_hooks(&pre_compact_ctx, claude_config.as_ref(), Some(&extended_config)).await;

        if !result.context.is_empty() {
            log::info!("PreCompact hooks injecting context. sessionID={}, contextCount={}, hookName={}, elapsedMs={}", 
                      session_id, 
                      result.context.len(),
                      result.hook_name.as_deref().unwrap_or(""),
                      result.elapsed_ms.map(|ms| ms.to_string()).unwrap_or("".to_string()));
        }

        result.context
    }

    pub async fn on_chat_message(&self, session_id: &str, agent: Option<&str>, model: Option<&HashMap<String, String>>, message_id: Option<&str>, 
                                  message: &HashMap<String, Value>, 
                                  parts: &[HashMap<String, Value>]) -> Result<(), String> {
        // ตรวจสอบสถานะการหยุดทำงาน
        {
            let interrupt_state = SESSION_INTERRUPT_STATE.read().await;
            if let Some(state) = interrupt_state.get(session_id) {
                if state.interrupted {
                    log::info!("chat.message hook skipped - session interrupted. sessionID={}", session_id);
                    return Ok(());
                }
            }
        }

        let claude_config = load_claude_hooks_config(None).await;
        let extended_config = load_plugin_extended_config().await;

        // กรอง text parts
        let text_parts: Vec<&HashMap<String, Value>> = parts.iter()
            .filter(|p| {
                if let Some(type_val) = p.get("type") {
                    if let Some(type_str) = type_val.as_str() {
                        if type_str == "text" {
                            return p.contains_key("text") && p.get("text").is_some();
                        }
                    }
                }
                false
            })
            .collect();

        // รวม prompt จาก text parts
        let prompt = text_parts.iter()
            .filter_map(|p| p.get("text"))
            .filter_map(|t| t.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        record_user_message(session_id, &prompt).await.map_err(|e| e.to_string())?;

        // ตรวจสอบสถานะก่อนรัน hooks
        {
            let interrupt_state = SESSION_INTERRUPT_STATE.read().await;
            if let Some(state) = interrupt_state.get(session_id) {
                if state.interrupted {
                    log::info!("chat.message hooks skipped - interrupted during preparation. sessionID={}", session_id);
                    return Ok(());
                }
            }
        }

        // ตรวจสอบว่าเป็นข้อความแรกหรือไม่
        let is_first_message = {
            let mut processed = SESSION_FIRST_MESSAGE_PROCESSED.write().await;
            let was_present = processed.contains(session_id);
            if !was_present {
                processed.insert(session_id.to_string());
            }
            !was_present
        };

        // รัน UserPromptSubmit hooks ถ้าไม่ถูกปิดใช้งาน
        if !is_hook_disabled(&self.config, "UserPromptSubmit").await {
            let parent_session_id = None; // ต้องดึงจาก session API จริง
            
            let user_prompt_ctx = UserPromptSubmitContext {
                session_id: session_id.to_string(),
                parent_session_id,
                prompt: prompt.clone(),
                parts: vec![], // ต้องแปลงจาก HashMap ไปเป็น MessagePart
                cwd: self.directory.clone(),
            };

            let result = execute_user_prompt_submit_hooks(&user_prompt_ctx, claude_config.as_ref(), Some(&extended_config)).await;

            if result.block {
                return Err(result.reason.unwrap_or_else(|| "Hook blocked the prompt".to_string()));
            }

            // ตรวจสอบสถานะหลังรัน hooks
            {
                let interrupt_state = SESSION_INTERRUPT_STATE.read().await;
                if let Some(state) = interrupt_state.get(session_id) {
                    if state.interrupted {
                        log::info!("chat.message injection skipped - interrupted during hooks. sessionID={}", session_id);
                        return Ok(());
                    }
                }
            }

            if !result.messages.is_empty() {
                let hook_content = result.messages.join("\n\n");
                log::info!("[claude-code-hooks] Injecting {} hook messages. sessionID={}, contentLength={}, isFirstMessage={}", 
                           result.messages.len(),
                           session_id, 
                           hook_content.len(),
                           is_first_message);
            }
        }

        Ok(())
    }

    pub async fn on_tool_execute_before(&self, tool: &str, session_id: &str, call_id: &str, args: &mut HashMap<String, Value>) -> Result<(), String> {
        // จัดการ todowrite tool พิเศษ
        if tool == "todowrite" {
            if let Some(Value::String(todos_str)) = args.get("todos") {
                match serde_json::from_str::<Value>(todos_str) {
                    Ok(parsed) => {
                        if parsed.is_array() {
                            args.insert("todos".to_string(), parsed);
                            log::info!("todowrite: parsed todos string to array. sessionID={}", session_id);
                        } else {
                            return Err(format!(
                                "[todowrite ERROR] Parsed JSON is not an array. Received type: {}. Expected: Array of todo objects.",
                                parsed.type_name()
                            ));
                        }
                    }
                    Err(_) => {
                        return Err(format!(
                            "[todowrite ERROR] Failed to parse todos string as JSON. Expected: Valid JSON array."
                        ));
                    }
                }
            }
        }

        let claude_config = load_claude_hooks_config(None).await;
        let extended_config = load_plugin_extended_config().await;

        // บันทึกการใช้เครื่องมือ
        record_tool_use(session_id, tool, args.clone()).await.map_err(|e| e.to_string())?;

        // แคช input ของเครื่องมือ
        self.tool_input_cache.cache_tool_input(session_id, tool, call_id, serde_json::to_value(args.clone()).unwrap()).await;

        // รัน PreToolUse hooks ถ้าไม่ถูกปิดใช้งาน
        if !is_hook_disabled(&self.config, "PreToolUse").await {
            let pre_ctx = PreToolUseContext {
                session_id: session_id.to_string(),
                tool_name: tool.to_string(),
                tool_input: args.clone(),
                cwd: self.directory.clone(),
                tool_use_id: Some(call_id.to_string()),
                transcript_path: None,
                permission_mode: None,
            };

            let result = execute_pre_tool_use_hooks(&pre_ctx, claude_config.as_ref(), Some(&extended_config)).await;

            if matches!(result.decision, crate::hooks::claude_code_hooks::types::PermissionDecision::Deny) {
                // แสดง toast notification (จำลอง)
                log::warn!("PreToolUse Hook Executed - Blocked. sessionID={}, toolName={}, hookName={}, elapsedMs={}", 
                           session_id,
                           result.tool_name.as_deref().unwrap_or(tool),
                           result.hook_name.as_deref().unwrap_or(""),
                           result.elapsed_ms.map(|ms| ms.to_string()).unwrap_or("0".to_string()));

                return Err(result.reason.unwrap_or_else(|| "Hook blocked the operation".to_string()));
            }

            // อัปเดต args ถ้ามีการเปลี่ยนแปลง
            if let Some(modified_input) = result.modified_input {
                for (key, value) in modified_input {
                    args.insert(key, value);
                }
            }
        }

        Ok(())
    }

    pub async fn on_tool_execute_after(&self, tool: &str, session_id: &str, call_id: &str, 
                                       title: &str, output: &str, metadata: Option<&Value>) -> Result<String, String> {
        let claude_config = load_claude_hooks_config(None).await;
        let extended_config = load_plugin_extended_config().await;

        // ดึง input ที่แคชไว้
        let cached_input = self.tool_input_cache.get_tool_input(session_id, tool, call_id).await;
        let cached_input = cached_input.unwrap_or_else(|| serde_json::json!({}));

        // สร้าง tool output
        let tool_output = if let Some(meta) = metadata {
            if !meta.is_null() && (meta.is_object() && !meta.as_object().unwrap().is_empty()) {
                meta.clone()
            } else {
                serde_json::json!({"output": output})
            }
        } else {
            serde_json::json!({"output": output})
        };

        // บันทึกผลลัพธ์ของเครื่องมือ
        record_tool_result(session_id, tool, cached_input.clone(), tool_output.clone()).await.map_err(|e| e.to_string())?;

        // รัน PostToolUse hooks ถ้าไม่ถูกปิดใช้งาน
        if !is_hook_disabled(&self.config, "PostToolUse").await {
            let post_ctx = PostToolUseContext {
                session_id: session_id.to_string(),
                tool_name: tool.to_string(),
                tool_input: cached_input.as_object().unwrap_or(&serde_json::Map::new()).clone(),
                tool_output: tool_output.as_object().unwrap_or(&serde_json::Map::new()).clone(),
                cwd: self.directory.clone(),
                transcript_path: Some(get_transcript_path(session_id)),
                tool_use_id: Some(call_id.to_string()),
                client: None, // ต้องใช้ client จริง
                permission_mode: Some("bypassPermissions".to_string()),
            };

            let result = execute_post_tool_use_hooks(&post_ctx, claude_config.as_ref(), Some(&extended_config)).await;

            if result.block {
                log::warn!("PostToolUse Hook Warning. sessionID={}, reason={}", 
                           session_id,
                           result.reason.as_deref().unwrap_or("Hook returned warning"));
            }

            let mut final_output = output.to_string();
            if let Some(ref warnings) = result.warnings {
                final_output.push_str(&format!("\n\n{}", warnings.join("\n")));
            }
            if let Some(ref message) = result.message {
                final_output.push_str(&format!("\n\n{}", message));
            }

            if let Some(ref hook_name) = result.hook_name {
                log::info!("PostToolUse Hook Executed. sessionID={}, toolName={}, hookName={}, elapsedMs={}", 
                           session_id,
                           result.tool_name.as_deref().unwrap_or(tool),
                           hook_name,
                           result.elapsed_ms.map(|ms| ms.to_string()).unwrap_or("0".to_string()));
            }

            return Ok(final_output);
        }

        Ok(output.to_string())
    }

    pub async fn handle_event(&self, event_type: &str, properties: Option<&HashMap<String, Value>>) -> Result<(), String> {
        match event_type {
            "session.error" => {
                if let Some(props) = properties {
                    if let Some(Value::String(session_id)) = props.get("sessionID") {
                        let error_msg = props.get("error").map(|e| e.to_string()).unwrap_or_else(|| "Unknown error".to_string());
                        let mut error_state = SESSION_ERROR_STATE.write().await;
                        error_state.insert(
                            session_id.clone(), 
                            SessionErrorState { 
                                has_error: true, 
                                error_message: Some(error_msg) 
                            }
                        );
                    }
                }
            }
            "session.deleted" => {
                if let Some(props) = properties {
                    if let Some(info_val) = props.get("info") {
                        if let Some(info_obj) = info_val.as_object() {
                            if let Some(Value::String(session_id)) = info_obj.get("id") {
                                let mut error_state = SESSION_ERROR_STATE.write().await;
                                error_state.remove(session_id);
                                
                                let mut interrupt_state = SESSION_INTERRUPT_STATE.write().await;
                                interrupt_state.remove(session_id);
                                
                                let mut processed = SESSION_FIRST_MESSAGE_PROCESSED.write().await;
                                processed.remove(session_id);
                            }
                        }
                    }
                }
            }
            "session.idle" => {
                if let Some(props) = properties {
                    if let Some(Value::String(session_id)) = props.get("sessionID") {
                        let claude_config = load_claude_hooks_config(None).await;
                        let extended_config = load_plugin_extended_config().await;

                        // ตรวจสอบสถานะก่อนหน้า
                        let error_state_before = {
                            let error_state = SESSION_ERROR_STATE.read().await;
                            error_state.get(session_id).cloned()
                        };
                        
                        let interrupt_state_before = {
                            let interrupt_state = SESSION_INTERRUPT_STATE.read().await;
                            interrupt_state.get(session_id).cloned()
                        };

                        // รัน Stop hooks ถ้าไม่ถูกปิดใช้งาน
                        if !is_hook_disabled(&self.config, "Stop").await {
                            let stop_ctx = StopContext {
                                session_id: session_id.clone(),
                                parent_session_id: None, // ต้องดึงจาก session API จริง
                                cwd: self.directory.clone(),
                                transcript_path: None,
                                permission_mode: None,
                                stop_hook_active: false,
                                todo_path: None,
                            };

                            let stop_result = execute_stop_hooks(&stop_ctx, claude_config.as_ref(), Some(&extended_config)).await;

                            let error_state_after = {
                                let error_state = SESSION_ERROR_STATE.read().await;
                                error_state.get(session_id).cloned()
                            };
                            
                            let interrupt_state_after = {
                                let interrupt_state = SESSION_INTERRUPT_STATE.read().await;
                                interrupt_state.get(session_id).cloned()
                            };

                            let ended_with_error_before = error_state_before.as_ref().map(|s| s.has_error).unwrap_or(false);
                            let ended_with_error_after = error_state_after.as_ref().map(|s| s.has_error).unwrap_or(false);
                            let interrupted_before = interrupt_state_before.as_ref().map(|s| s.interrupted).unwrap_or(false);
                            let interrupted_after = interrupt_state_after.as_ref().map(|s| s.interrupted).unwrap_or(false);

                            let should_bypass = ended_with_error_before || ended_with_error_after || interrupted_before || interrupted_after;

                            if should_bypass && stop_result.block {
                                let interrupted = interrupted_before || interrupted_after;
                                let ended_with_error = ended_with_error_before || ended_with_error_after;
                                log::info!("Stop hook block ignored. sessionID={}, block={}, interrupted={}, endedWithError={}", 
                                          session_id,
                                          stop_result.block,
                                          interrupted,
                                          ended_with_error);
                            } else if stop_result.block && stop_result.inject_prompt.is_some() {
                                log::info!("Stop hook returned block with inject_prompt. sessionID={}", session_id);
                                // ต้องใช้ session API จริงในการ inject prompt
                            } else if stop_result.block {
                                log::info!("Stop hook returned block. sessionID={}, reason={}", 
                                          session_id,
                                          stop_result.reason.as_deref().unwrap_or(""));
                            }
                        }

                        // ล้างสถานะ
                        {
                            let mut error_state = SESSION_ERROR_STATE.write().await;
                            error_state.remove(session_id);
                            
                            let mut interrupt_state = SESSION_INTERRUPT_STATE.write().await;
                            interrupt_state.remove(session_id);
                        }
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }
}

async fn is_hook_disabled(config: &PluginConfig, hook_name: &str) -> bool {
    // ตรวจสอบว่า hook ถูกปิดใช้งานใน config หรือไม่
    if let Some(ref disabled_hooks) = config.disabled_hooks {
        match disabled_hooks {
            crate::hooks::claude_code_hooks::types::DisabledHooks::All(disabled) => *disabled,
            crate::hooks::claude_code_hooks::types::DisabledHooks::Events(events) => {
                // ตรวจสอบว่า hook_name อยู่ในรายการที่ปิดใช้งาน
                events.iter().any(|event| {
                    match event {
                        crate::hooks::claude_code_hooks::types::ClaudeHookEvent::Stop => hook_name == "Stop",
                        crate::hooks::claude_code_hooks::types::ClaudeHookEvent::PreToolUse => hook_name == "PreToolUse",
                        crate::hooks::claude_code_hooks::types::ClaudeHookEvent::PostToolUse => hook_name == "PostToolUse",
                        crate::hooks::claude_code_hooks::types::ClaudeHookEvent::UserPromptSubmit => hook_name == "UserPromptSubmit",
                        crate::hooks::claude_code_hooks::types::ClaudeHookEvent::PreCompact => hook_name == "PreCompact",
                    }
                })
            }
        }
    } else {
        false
    }
}