use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio;

use crate::hooks::claude_code_hooks::types::{UserPromptSubmitInput, PostToolUseOutput, ClaudeHooksConfig};
use crate::hooks::claude_code_hooks::config_loader::{is_hook_command_disabled, PluginExtendedConfig};
use crate::hooks::claude_code_hooks::plugin_config::DEFAULT_CONFIG;

const USER_PROMPT_SUBMIT_TAG_OPEN: &str = "<user-prompt-submit-hook>";
const USER_PROMPT_SUBMIT_TAG_CLOSE: &str = "</user-prompt-submit-hook>";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagePart {
    #[serde(rename = "type")]
    part_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct UserPromptSubmitContext {
    pub session_id: String,
    pub parent_session_id: Option<String>,
    pub prompt: String,
    pub parts: Vec<MessagePart>,
    pub cwd: String,
    pub permission_mode: Option<String>, // "default", "acceptEdits", "bypassPermissions"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPromptSubmitResult {
    pub block: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub modified_parts: Vec<MessagePart>,
    pub messages: Vec<String>,
}

pub async fn execute_user_prompt_submit_hooks(
    ctx: &UserPromptSubmitContext,
    config: Option<&ClaudeHooksConfig>,
    extended_config: Option<&PluginExtendedConfig>,
) -> UserPromptSubmitResult {
    let mut modified_parts = ctx.parts.clone();
    let mut messages = Vec::new();

    // ถ้ามี parent session ID ให้ไม่บล็อค
    if ctx.parent_session_id.is_some() {
        return UserPromptSubmitResult {
            block: false,
            reason: None,
            modified_parts,
            messages,
        };
    }

    // ถ้า prompt มีแท็กอยู่แล้ว ให้ไม่บล็อค
    if ctx.prompt.contains(USER_PROMPT_SUBMIT_TAG_OPEN) && 
       ctx.prompt.contains(USER_PROMPT_SUBMIT_TAG_CLOSE) {
        return UserPromptSubmitResult {
            block: false,
            reason: None,
            modified_parts,
            messages,
        };
    }

    // ถ้าไม่มี config ให้ไม่บล็อค
    let config = match config {
        Some(c) => c,
        None => {
            return UserPromptSubmitResult {
                block: false,
                reason: None,
                modified_parts,
                messages,
            }
        }
    };

    // ค้นหา matchers สำหรับ UserPromptSubmit hook
    let matchers = find_matching_hooks(config, "UserPromptSubmit");
    if matchers.is_empty() {
        return UserPromptSubmitResult {
            block: false,
            reason: None,
            modified_parts,
            messages,
        };
    }

    // สร้าง input สำหรับ hook
    let stdin_data = UserPromptSubmitInput {
        session_id: ctx.session_id.clone(),
        cwd: ctx.cwd.clone(),
        permission_mode: None, // ต้องแปลงจาก string ไปเป็น enum ที่เหมาะสม
        hook_event_name: "UserPromptSubmit".to_string(),
        prompt: ctx.prompt.clone(),
        session: Some(crate::hooks::claude_code_hooks::types::SessionInfo {
            id: ctx.session_id.clone(),
        }),
        hook_source: Some(crate::hooks::claude_code_hooks::types::HookSource::OpencodePlugin),
    };

    // ประมวลผลแต่ละ hook
    for matcher in &matchers {
        for hook in &matcher.hooks {
            if hook.command_type != "command" {
                continue;
            }

            // ตรวจสอบว่า hook ถูกปิดใช้งานหรือไม่
            if is_hook_command_disabled("UserPromptSubmit", &hook.command, extended_config).await {
                log::info!("UserPromptSubmit hook command skipped (disabled by config). command={}", hook.command);
                continue;
            }

            // รันคำสั่ง hook
            let result = execute_hook_command(
                &hook.command,
                &serde_json::to_string(&stdin_data).unwrap(),
                &ctx.cwd,
            ).await;

            // จัดการผลลัพธ์ stdout
            if let Some(stdout) = &result.stdout {
                let output = stdout.trim();
                if output.starts_with(USER_PROMPT_SUBMIT_TAG_OPEN) {
                    messages.push(output.to_string());
                } else {
                    let message = format!("{}\n{}\n{}", 
                                         USER_PROMPT_SUBMIT_TAG_OPEN, 
                                         output, 
                                         USER_PROMPT_SUBMIT_TAG_CLOSE);
                    messages.push(message);
                }
            }

            // ตรวจสอบ exit code
            if result.exit_code != 0 {
                // พยายาม parse ผลลัพธ์เป็น PostToolUseOutput
                if let Ok(output) = serde_json::from_str::<PostToolUseOutput>(result.stdout.as_deref().unwrap_or("{}")) {
                    if output.decision.as_ref().map(|d| d == "block").unwrap_or(false) {
                        return UserPromptSubmitResult {
                            block: true,
                            reason: output.reason.or(result.stderr),
                            modified_parts,
                            messages,
                        };
                    }
                }
            }
        }
    }

    UserPromptSubmitResult {
        block: false,
        reason: None,
        modified_parts,
        messages,
    }
}

// ฟังก์ชันช่วยเหลือ
fn find_matching_hooks(config: &ClaudeHooksConfig, event_type: &str) -> Vec<crate::hooks::claude_code_hooks::types::HookMatcher> {
    match event_type {
        "UserPromptSubmit" => config.user_prompt_submit.clone().unwrap_or_default(),
        _ => vec![],
    }
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