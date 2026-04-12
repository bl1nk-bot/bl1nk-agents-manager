use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio;
use std::time::Instant;

use crate::hooks::claude_code_hooks::types::{PostToolUseInput, PostToolUseOutput, ClaudeHooksConfig};
use crate::hooks::claude_code_hooks::config_loader::{is_hook_command_disabled, PluginExtendedConfig};
use crate::hooks::claude_code_hooks::plugin_config::DEFAULT_CONFIG;
use crate::hooks::claude_code_hooks::transcript::{build_transcript_from_session, delete_temp_transcript};

#[derive(Debug, Clone)]
pub struct PostToolUseClient {
    // จำลอง client structure สำหรับ session messages
    // ในระบบที่แท้จริง ควรใช้ client ที่เหมาะสม
}

#[derive(Debug, Clone)]
pub struct PostToolUseContext {
    pub session_id: String,
    pub tool_name: String,
    pub tool_input: serde_json::Map<String, serde_json::Value>,
    pub tool_output: serde_json::Map<String, serde_json::Value>,
    pub cwd: String,
    pub transcript_path: Option<String>,
    pub tool_use_id: Option<String>,
    pub client: Option<PostToolUseClient>,
    pub permission_mode: Option<String>, // "default", "plan", "acceptEdits", "bypassPermissions"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostToolUseResult {
    pub block: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elapsed_ms: Option<u128>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hook_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continue_execution: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suppress_output: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_message: Option<String>,
}

pub async fn execute_post_tool_use_hooks(
    ctx: &PostToolUseContext,
    config: Option<&ClaudeHooksConfig>,
    extended_config: Option<&PluginExtendedConfig>,
) -> PostToolUseResult {
    // ถ้าไม่มี config ให้คืนค่าไม่บล็อค
    let config = match config {
        Some(c) => c,
        None => {
            return PostToolUseResult {
                block: false,
                reason: None,
                message: None,
                warnings: None,
                elapsed_ms: None,
                hook_name: None,
                tool_name: None,
                additional_context: None,
                continue_execution: None,
                stop_reason: None,
                suppress_output: None,
                system_message: None,
            }
        }
    };

    // แปลงชื่อเครื่องมือ
    let transformed_tool_name = transform_tool_name(&ctx.tool_name);
    
    // ค้นหา matchers สำหรับ PostToolUse hook
    let matchers = find_matching_hooks(config, "PostToolUse", &transformed_tool_name);
    if matchers.is_empty() {
        return PostToolUseResult {
            block: false,
            reason: None,
            message: None,
            warnings: None,
            elapsed_ms: None,
            hook_name: None,
            tool_name: None,
            additional_context: None,
            continue_execution: None,
            stop_reason: None,
            suppress_output: None,
            system_message: None,
        };
    }

    // สร้าง transcript ชั่วคราว (จำลอง)
    let mut temp_transcript_path: Option<String> = None;

    // ใช้ block เพื่อจัดการการคืนค่าอย่างเหมาะสม
    let result = {
        // สร้าง input สำหรับ hook
        let stdin_data = PostToolUseInput {
            session_id: ctx.session_id.clone(),
            transcript_path: temp_transcript_path.clone().or(ctx.transcript_path.clone()),
            cwd: ctx.cwd.clone(),
            permission_mode: None, // ต้องแปลงจาก string ไปเป็น enum ที่เหมาะสม
            hook_event_name: "PostToolUse".to_string(),
            tool_name: transformed_tool_name.clone(),
            tool_input: ctx.tool_input.clone(),
            tool_response: ctx.tool_output.clone(),
            tool_use_id: ctx.tool_use_id.clone(),
            hook_source: Some(crate::hooks::claude_code_hooks::types::HookSource::OpencodePlugin),
        };

        let mut messages = Vec::new();
        let mut warnings = Vec::new();
        let mut first_hook_name: Option<String> = None;

        let start_time = Instant::now();

        // ประมวลผลแต่ละ hook
        for matcher in &matchers {
            for hook in &matcher.hooks {
                if hook.command_type != "command" {
                    continue;
                }

                // ตรวจสอบว่า hook ถูกปิดใช้งานหรือไม่
                if is_hook_command_disabled("PostToolUse", &hook.command, extended_config).await {
                    log::info!("PostToolUse hook command skipped (disabled by config). command={}, toolName={}", 
                              hook.command, ctx.tool_name);
                    continue;
                }

                // ดึงชื่อ hook
                let hook_name = hook.command.split('/').last().unwrap_or(&hook.command).to_string();
                if first_hook_name.is_none() {
                    first_hook_name = Some(hook_name.clone());
                }

                // รันคำสั่ง hook
                let result = execute_hook_command(
                    &hook.command,
                    &serde_json::to_string(&stdin_data).unwrap(),
                    &ctx.cwd,
                ).await;

                if let Some(stdout) = &result.stdout {
                    messages.push(stdout.clone());
                }

                // ถ้า exit code เป็น 2
                if result.exit_code == 2 {
                    if let Some(stderr) = &result.stderr {
                        let warning = format!("[{}]\n{}", hook_name, stderr.trim());
                        warnings.push(warning);
                    }
                    continue;
                }

                // ถ้า exit code เป็น 0 และมี stdout
                if result.exit_code == 0 {
                    if let Some(stdout) = &result.stdout {
                        // พยายาม parse เป็น PostToolUseOutput
                        if let Ok(output) = serde_json::from_str::<PostToolUseOutput>(stdout) {
                            if output.decision.as_ref().map(|d| d == "block").unwrap_or(false) {
                                return PostToolUseResult {
                                    block: true,
                                    reason: output.reason.or(result.stderr),
                                    message: Some(messages.join("\n")),
                                    warnings: if !warnings.is_empty() { Some(warnings) } else { None },
                                    elapsed_ms: Some(start_time.elapsed().as_millis()),
                                    hook_name: first_hook_name,
                                    tool_name: Some(transformed_tool_name),
                                    additional_context: output.hook_specific_output.as_ref()
                                        .and_then(|hso| hso.additional_context.clone()),
                                    continue_execution: output.common.continue_execution,
                                    stop_reason: output.common.stop_reason,
                                    suppress_output: output.common.suppress_output,
                                    system_message: output.common.system_message,
                                };
                            }
                            
                            // ตรวจสอบว่ามี common fields หรือไม่
                            let has_common_fields = output.common.continue_execution.is_some() ||
                                output.common.system_message.is_some() ||
                                output.common.suppress_output.is_some() ||
                                output.common.stop_reason.is_some();
                            
                            if has_common_fields {
                                return PostToolUseResult {
                                    block: false,
                                    message: Some(messages.join("\n")),
                                    warnings: if !warnings.is_empty() { Some(warnings) } else { None },
                                    elapsed_ms: Some(start_time.elapsed().as_millis()),
                                    hook_name: first_hook_name,
                                    tool_name: Some(transformed_tool_name),
                                    additional_context: output.hook_specific_output.as_ref()
                                        .and_then(|hso| hso.additional_context.clone()),
                                    continue_execution: output.common.continue_execution,
                                    stop_reason: output.common.stop_reason,
                                    suppress_output: output.common.suppress_output,
                                    system_message: output.common.system_message,
                                };
                            }
                        }
                    }
                } 
                // ถ้า exit code ไม่ใช่ 0 หรือ 2
                else if result.exit_code != 0 && result.exit_code != 2 {
                    if let Some(stdout) = &result.stdout {
                        // พยายาม parse เป็น PostToolUseOutput
                        if let Ok(output) = serde_json::from_str::<PostToolUseOutput>(stdout) {
                            if output.decision.as_ref().map(|d| d == "block").unwrap_or(false) {
                                return PostToolUseResult {
                                    block: true,
                                    reason: output.reason.or(result.stderr),
                                    message: Some(messages.join("\n")),
                                    warnings: if !warnings.is_empty() { Some(warnings) } else { None },
                                    elapsed_ms: Some(start_time.elapsed().as_millis()),
                                    hook_name: first_hook_name,
                                    tool_name: Some(transformed_tool_name),
                                    additional_context: output.hook_specific_output.as_ref()
                                        .and_then(|hso| hso.additional_context.clone()),
                                    continue_execution: output.common.continue_execution,
                                    stop_reason: output.common.stop_reason,
                                    suppress_output: output.common.suppress_output,
                                    system_message: output.common.system_message,
                                };
                            }
                        }
                    }
                }
            }
        }

        PostToolUseResult {
            block: false,
            message: if !messages.is_empty() { Some(messages.join("\n")) } else { None },
            warnings: if !warnings.is_empty() { Some(warnings) } else { None },
            elapsed_ms: Some(start_time.elapsed().as_millis()),
            hook_name: first_hook_name,
            tool_name: Some(transformed_tool_name),
            additional_context: None,
            continue_execution: None,
            stop_reason: None,
            suppress_output: None,
            system_message: None,
        }
    };

    // ล้างไฟล์ชั่วคราว
    delete_temp_transcript(temp_transcript_path.as_deref());

    result
}

// ฟังก์ชันช่วยเหลือ
fn find_matching_hooks(config: &ClaudeHooksConfig, event_type: &str, _pattern: &str) -> Vec<crate::hooks::claude_code_hooks::types::HookMatcher> {
    match event_type {
        "PostToolUse" => config.post_tool_use.clone().unwrap_or_default(),
        _ => vec![],
    }
}

fn transform_tool_name(tool_name: &str) -> String {
    // แปลงชื่อเครื่องมือ (จำลอง)
    // ในระบบที่แท้จริง ควรใช้ฟังก์ชันจาก shared module
    tool_name.replace("-", "_").to_lowercase()
}

#[derive(Debug)]
struct HookResult {
    exit_code: i32,
    stdout: Option<String>,
    stderr: Option<String>,
}

async fn execute_hook_command(command: &str, input_data: &str, cwd: &str) -> HookResult {
    // แยกคำสั่งและอาร์กิวเมนต์
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return HookResult {
            exit_code: 1,
            stdout: None,
            stderr: Some("Invalid command".to_string()),
        };
    }

    let mut cmd = tokio::process::Command::new(parts[0]);
    
    // เพิ่มอาร์กิวเมนต์
    if parts.len() > 1 {
        cmd.args(&parts[1..]);
    }
    
    // ตั้งค่าไดเรกทอรีทำงาน
    cmd.current_dir(cwd);
    
    // ตั้งค่า stdin
    cmd.stdin(std::process::Stdio::piped());
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    match cmd.spawn() {
        Ok(mut child) => {
            // ส่งข้อมูลไปยัง stdin
            if let Some(mut stdin) = child.stdin.take() {
                use tokio::io::AsyncWriteExt;
                if let Err(_) = stdin.write_all(input_data.as_bytes()).await {
                    // ถ้าส่งข้อมูลล้มเหลว ให้ปิด stdin
                }
            }

            // รอผลลัพธ์
            match child.wait_with_output().await {
                Ok(output) => HookResult {
                    exit_code: output.status.code().unwrap_or(1),
                    stdout: String::from_utf8(output.stdout).ok(),
                    stderr: String::from_utf8(output.stderr).ok(),
                },
                Err(e) => HookResult {
                    exit_code: 1,
                    stdout: None,
                    stderr: Some(e.to_string()),
                },
            }
        },
        Err(e) => HookResult {
            exit_code: 1,
            stdout: None,
            stderr: Some(e.to_string()),
        },
    }
}