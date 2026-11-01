use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio;
use std::time::Instant;

use crate::hooks::claude_code_hooks::types::{PreCompactInput, PreCompactOutput, ClaudeHooksConfig};
use crate::hooks::claude_code_hooks::config_loader::{is_hook_command_disabled, PluginExtendedConfig};
use crate::hooks::claude_code_hooks::plugin_config::DEFAULT_CONFIG;

#[derive(Debug, Clone)]
pub struct PreCompactContext {
    pub session_id: String,
    pub cwd: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreCompactResult {
    pub context: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elapsed_ms: Option<u128>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hook_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continue_execution: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suppress_output: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_message: Option<String>,
}

pub async fn execute_pre_compact_hooks(
    ctx: &PreCompactContext,
    config: Option<&ClaudeHooksConfig>,
    extended_config: Option<&PluginExtendedConfig>,
) -> PreCompactResult {
    // ถ้าไม่มี config ให้คืนค่าเปล่า
    let config = match config {
        Some(c) => c,
        None => {
            return PreCompactResult {
                context: vec![],
                elapsed_ms: None,
                hook_name: None,
                continue_execution: None,
                stop_reason: None,
                suppress_output: None,
                system_message: None,
            }
        }
    };

    // ค้นหา matchers สำหรับ PreCompact hook
    let matchers = find_matching_hooks(config, "PreCompact", "*");
    if matchers.is_empty() {
        return PreCompactResult {
            context: vec![],
            elapsed_ms: None,
            hook_name: None,
            continue_execution: None,
            stop_reason: None,
            suppress_output: None,
            system_message: None,
        };
    }

    // สร้าง input สำหรับ hook
    let stdin_data = PreCompactInput {
        session_id: ctx.session_id.clone(),
        cwd: ctx.cwd.clone(),
        hook_event_name: "PreCompact".to_string(),
        hook_source: Some(crate::hooks::claude_code_hooks::types::HookSource::OpencodePlugin),
    };

    let start_time = Instant::now();
    let mut first_hook_name: Option<String> = None;
    let mut collected_context = Vec::new();

    // ประมวลผลแต่ละ hook
    for matcher in &matchers {
        for hook in &matcher.hooks {
            if hook.command_type != "command" {
                continue;
            }

            // ตรวจสอบว่า hook ถูกปิดใช้งานหรือไม่
            if is_hook_command_disabled("PreCompact", &hook.command, extended_config).await {
                log::info!("PreCompact hook command skipped (disabled by config). command={}", hook.command);
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

            // ถ้า exit code เป็น 2 ให้ข้าม hook นี้
            if result.exit_code == 2 {
                log::warn!("PreCompact hook blocked. hookName={}, stderr={}", 
                          hook_name, result.stderr.as_deref().unwrap_or(""));
                continue;
            }

            // จัดการผลลัพธ์ stdout
            if let Some(stdout) = &result.stdout {
                // พยายาม parse เป็น PreCompactOutput
                if let Ok(output) = serde_json::from_str::<PreCompactOutput>(stdout) {
                    // เพิ่ม context จาก hook-specific output
                    if let Some(ref hook_specific_output) = output.hook_specific_output {
                        if let Some(ref additional_context) = hook_specific_output.additional_context {
                            collected_context.extend(additional_context.clone());
                        }
                    }
                    // เพิ่ม context จาก output ทั่วไป
                    else if let Some(ref context) = output.context {
                        collected_context.extend(context.clone());
                    }

                    // ตรวจสอบว่าควรหยุดต่อหรือไม่
                    if output.common.continue_execution == Some(false) {
                        return PreCompactResult {
                            context: collected_context,
                            elapsed_ms: Some(start_time.elapsed().as_millis()),
                            hook_name: first_hook_name,
                            continue_execution: output.common.continue_execution,
                            stop_reason: output.common.stop_reason,
                            suppress_output: output.common.suppress_output,
                            system_message: output.common.system_message,
                        };
                    }
                } else {
                    // ถ้า parse ไม่ได้ ให้เพิ่ม stdout เข้าไปโดยตรง
                    if !stdout.trim().is_empty() {
                        collected_context.push(stdout.trim().to_string());
                    }
                }
            }
        }
    }

    PreCompactResult {
        context: collected_context,
        elapsed_ms: Some(start_time.elapsed().as_millis()),
        hook_name: first_hook_name,
        continue_execution: None,
        stop_reason: None,
        suppress_output: None,
        system_message: None,
    }
}

// ฟังก์ชันช่วยเหลือ
fn find_matching_hooks(config: &ClaudeHooksConfig, event_type: &str, _pattern: &str) -> Vec<crate::hooks::claude_code_hooks::types::HookMatcher> {
    match event_type {
        "PreCompact" => config.pre_compact.clone().unwrap_or_default(),
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