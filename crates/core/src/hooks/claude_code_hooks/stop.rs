use std::collections::HashMap;
use tokio::process::Command;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::hooks::claude_code_hooks::types::{StopInput, StopOutput, ClaudeHooksConfig, PermissionMode};
use crate::hooks::claude_code_hooks::config_loader::{is_hook_command_disabled, PluginExtendedConfig};
use crate::hooks::claude_code_hooks::todo::get_todo_path;
use crate::hooks::claude_code_hooks::plugin_config::DEFAULT_CONFIG;

// สถานะระดับโมดูลเพื่อติดตาม stop_hook_active ต่อ session
lazy_static::lazy_static! {
    static ref STOP_HOOK_ACTIVE_STATE: Arc<RwLock<HashMap<String, bool>>> = 
        Arc::new(RwLock::new(HashMap::new()));
}

pub async fn set_stop_hook_active(session_id: &str, active: bool) {
    let mut state = STOP_HOOK_ACTIVE_STATE.write().await;
    state.insert(session_id.to_string(), active);
}

pub async fn get_stop_hook_active(session_id: &str) -> bool {
    let state = STOP_HOOK_ACTIVE_STATE.read().await;
    *state.get(session_id).unwrap_or(&false)
}

#[derive(Debug, Clone)]
pub struct StopContext {
    pub session_id: String,
    pub parent_session_id: Option<String>,
    pub cwd: String,
    pub transcript_path: Option<String>,
    pub permission_mode: Option<PermissionMode>,
    pub stop_hook_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopResult {
    pub block: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_hook_active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_mode: Option<PermissionMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inject_prompt: Option<String>,
}

pub async fn execute_stop_hooks(
    ctx: &StopContext,
    config: Option<&ClaudeHooksConfig>,
    extended_config: Option<&PluginExtendedConfig>,
) -> StopResult {
    // ถ้ามี parent session ID ให้ไม่บล็อค
    if ctx.parent_session_id.is_some() {
        return StopResult {
            block: false,
            reason: None,
            stop_hook_active: None,
            permission_mode: None,
            inject_prompt: None,
        }
    }

    // ถ้าไม่มี config ให้ไม่บล็อค
    let config = match config {
        Some(c) => c,
        None => {
            return StopResult {
                block: false,
                reason: None,
                stop_hook_active: None,
                permission_mode: None,
                inject_prompt: None,
            }
        }
    };

    // ค้นหา matchers สำหรับ Stop hook
    let matchers = find_matching_hooks(config, "Stop");
    if matchers.is_empty() {
        return StopResult {
            block: false,
            reason: None,
            stop_hook_active: None,
            permission_mode: None,
            inject_prompt: None,
        }
    }

    // สร้าง input สำหรับ hook
    let stdin_data = StopInput {
        session_id: ctx.session_id.clone(),
        transcript_path: ctx.transcript_path.clone(),
        cwd: ctx.cwd.clone(),
        permission_mode: ctx.permission_mode.clone().unwrap_or(PermissionMode::BypassPermissions),
        hook_event_name: "Stop".to_string(),
        stop_hook_active: get_stop_hook_active(&ctx.session_id).await,
        todo_path: get_todo_path(&ctx.session_id).await,
        hook_source: Some(crate::hooks::claude_code_hooks::types::HookSource::OpencodePlugin),
    };

    // ประมวลผลแต่ละ hook
    for matcher in &matchers {
        for hook in &matcher.hooks {
            if hook.command_type != "command" {
                continue;
            }

            // ตรวจสอบว่า hook ถูกปิดใช้งานหรือไม่
            if is_hook_command_disabled("Stop", &hook.command, extended_config).await {
                log::info!("Stop hook command skipped (disabled by config). command={}", hook.command);
                continue;
            }

            // รันคำสั่ง hook
            let result = execute_hook_command(
                &hook.command,
                &serde_json::to_string(&stdin_data).unwrap(),
                &ctx.cwd,
            ).await;

            // ตรวจสอบ exit code - exit code 2 หมายถึงบล็อค
            if result.exit_code == 2 {
                let reason = result.stderr.clone().unwrap_or_else(|| 
                    result.stdout.clone().unwrap_or_else(|| "Blocked by stop hook".to_string()));
                return StopResult {
                    block: true,
                    reason: Some(reason.clone()),
                    inject_prompt: Some(reason),
                    stop_hook_active: None,
                    permission_mode: None,
                };
            }

            // ประมวลผลผลลัพธ์ stdout
            if let Some(stdout) = &result.stdout {
                if let Ok(output) = serde_json::from_str::<StopOutput>(stdout) {
                    // อัปเดตสถานะ stop hook
                    if let Some(active) = output.stop_hook_active {
                        set_stop_hook_active(&ctx.session_id, active).await;
                    }

                    let is_block = output.decision.as_ref().map(|d| d == "block").unwrap_or(false);
                    let inject_prompt = output.inject_prompt.clone().or_else(|| {
                        if is_block && output.reason.is_some() {
                            output.reason.clone()
                        } else {
                            None
                        }
                    });

                    return StopResult {
                        block: is_block,
                        reason: output.reason,
                        stop_hook_active: output.stop_hook_active,
                        permission_mode: output.permission_mode.clone(),
                        inject_prompt,
                    };
                }
            }
        }
    }

    StopResult {
        block: false,
        reason: None,
        stop_hook_active: None,
        permission_mode: None,
        inject_prompt: None,
    }
}

// ฟังก์ชันช่วยเหลือ
fn find_matching_hooks(config: &ClaudeHooksConfig, event_type: &str) -> Vec<crate::hooks::claude_code_hooks::types::HookMatcher> {
    match event_type {
        "Stop" => config.stop.clone().unwrap_or_default(),
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

    let mut cmd = Command::new(parts[0]);
    
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

// ต้องเพิ่ม derive macro สำหรับ lazy_static
// ซึ่งควรเพิ่มใน Cargo.toml ภายหลัง