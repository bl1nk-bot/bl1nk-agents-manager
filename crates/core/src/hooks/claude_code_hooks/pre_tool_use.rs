use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio;
use std::time::Instant;

use crate::hooks::claude_code_hooks::types::{PreToolUseInput, PreToolUseOutput, PermissionDecision, ClaudeHooksConfig};
use crate::hooks::claude_code_hooks::config_loader::{is_hook_command_disabled, PluginExtendedConfig};
use crate::hooks::claude_code_hooks::plugin_config::DEFAULT_CONFIG;

#[derive(Debug, Clone)]
pub struct PreToolUseContext {
    pub session_id: String,
    pub tool_name: String,
    pub tool_input: serde_json::Map<String, serde_json::Value>,
    pub cwd: String,
    pub transcript_path: Option<String>,
    pub tool_use_id: Option<String>,
    pub permission_mode: Option<String>, // "default", "plan", "acceptEdits", "bypassPermissions"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreToolUseResult {
    pub decision: PermissionDecision,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_input: Option<serde_json::Map<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elapsed_ms: Option<u128>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hook_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_lines: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continue_execution: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suppress_output: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_message: Option<String>,
}

fn build_input_lines(tool_input: &serde_json::Map<String, serde_json::Value>) -> String {
    let mut lines = Vec::new();
    for (i, (key, val)) in tool_input.iter().take(3).enumerate() {
        let val_str = format!("{}", val);
        let truncated_val = if val_str.len() > 40 {
            format!("{}...", &val_str[..40])
        } else {
            val_str
        };
        lines.push(format!("  {}: {}", key, truncated_val));
    }
    lines.join("\n")
}

pub async fn execute_pre_tool_use_hooks(
    ctx: &PreToolUseContext,
    config: Option<&ClaudeHooksConfig>,
    extended_config: Option<&PluginExtendedConfig>,
) -> PreToolUseResult {
    // ถ้าไม่มี config ให้คืนค่า allow
    let config = match config {
        Some(c) => c,
        None => {
            return PreToolUseResult {
                decision: PermissionDecision::Allow,
                reason: None,
                modified_input: None,
                elapsed_ms: None,
                hook_name: None,
                tool_name: None,
                input_lines: None,
                continue_execution: None,
                stop_reason: None,
                suppress_output: None,
                system_message: None,
            }
        }
    };

    // แปลงชื่อเครื่องมือ
    let transformed_tool_name = transform_tool_name(&ctx.tool_name);
    
    // ค้นหา matchers สำหรับ PreToolUse hook
    let matchers = find_matching_hooks(config, "PreToolUse", &transformed_tool_name);
    if matchers.is_empty() {
        return PreToolUseResult {
            decision: PermissionDecision::Allow,
            reason: None,
            modified_input: None,
            elapsed_ms: None,
            hook_name: None,
            tool_name: None,
            input_lines: None,
            continue_execution: None,
            stop_reason: None,
            suppress_output: None,
            system_message: None,
        };
    }

    // สร้าง input สำหรับ hook
    let stdin_data = PreToolUseInput {
        session_id: ctx.session_id.clone(),
        transcript_path: ctx.transcript_path.clone(),
        cwd: ctx.cwd.clone(),
        permission_mode: None, // ต้องแปลงจาก string ไปเป็น enum ที่เหมาะสม
        hook_event_name: "PreToolUse".to_string(),
        tool_name: transformed_tool_name.clone(),
        tool_input: ctx.tool_input.clone(),
        tool_use_id: ctx.tool_use_id.clone(),
        hook_source: Some(crate::hooks::claude_code_hooks::types::HookSource::OpencodePlugin),
    };

    let start_time = Instant::now();
    let mut first_hook_name: Option<String> = None;
    let input_lines = build_input_lines(&ctx.tool_input);

    // ประมวลผลแต่ละ hook
    for matcher in &matchers {
        for hook in &matcher.hooks {
            if hook.command_type != "command" {
                continue;
            }

            // ตรวจสอบว่า hook ถูกปิดใช้งานหรือไม่
            if is_hook_command_disabled("PreToolUse", &hook.command, extended_config).await {
                log::info!("PreToolUse hook command skipped (disabled by config). command={}, toolName={}", 
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

            // ถ้า exit code เป็น 2 ให้คืนค่า deny
            if result.exit_code == 2 {
                return PreToolUseResult {
                    decision: PermissionDecision::Deny,
                    reason: result.stderr.or(result.stdout).or(Some("Hook blocked the operation".to_string())),
                    elapsed_ms: Some(start_time.elapsed().as_millis()),
                    hook_name: first_hook_name,
                    tool_name: Some(transformed_tool_name),
                    input_lines: Some(input_lines),
                    modified_input: None,
                    continue_execution: None,
                    stop_reason: None,
                    suppress_output: None,
                    system_message: None,
                };
            }

            // ถ้า exit code เป็น 1 ให้คืนค่า ask
            if result.exit_code == 1 {
                return PreToolUseResult {
                    decision: PermissionDecision::Ask,
                    reason: result.stderr.or(result.stdout),
                    elapsed_ms: Some(start_time.elapsed().as_millis()),
                    hook_name: first_hook_name,
                    tool_name: Some(transformed_tool_name),
                    input_lines: Some(input_lines),
                    modified_input: None,
                    continue_execution: None,
                    stop_reason: None,
                    suppress_output: None,
                    system_message: None,
                };
            }

            // จัดการผลลัพธ์ stdout
            if let Some(stdout) = &result.stdout {
                // พยายาม parse เป็น PreToolUseOutput
                if let Ok(output) = serde_json::from_str::<PreToolUseOutput>(stdout) {
                    // จัดการผลลัพธ์จาก hook-specific output
                    let mut decision: Option<PermissionDecision> = None;
                    let mut reason: Option<String> = None;
                    let mut modified_input: Option<serde_json::Map<String, serde_json::Value>> = None;

                    if let Some(ref hook_specific_output) = output.hook_specific_output {
                        decision = Some(hook_specific_output.permission_decision.clone());
                        reason = hook_specific_output.permission_decision_reason.clone();
                        modified_input = hook_specific_output.updated_input.clone();
                    }
                    // จัดการผลลัพธ์แบบเก่า (deprecated)
                    else if let Some(ref legacy_decision) = output.decision {
                        // แปลงค่าเก่าเป็นค่าใหม่
                        decision = match legacy_decision.as_str() {
                            "approve" | "allow" => Some(PermissionDecision::Allow),
                            "block" | "deny" => Some(PermissionDecision::Deny),
                            "ask" => Some(PermissionDecision::Ask),
                            _ => None,
                        };
                        reason = output.reason.clone();
                    }

                    // ตรวจสอบว่ามี common fields หรือไม่
                    let has_common_fields = output.common.continue_execution.is_some() ||
                        output.common.stop_reason.is_some() ||
                        output.common.suppress_output.is_some() ||
                        output.common.system_message.is_some();

                    if decision.is_some() || has_common_fields {
                        return PreToolUseResult {
                            decision: decision.unwrap_or(PermissionDecision::Allow),
                            reason,
                            modified_input,
                            elapsed_ms: Some(start_time.elapsed().as_millis()),
                            hook_name: first_hook_name,
                            tool_name: Some(transformed_tool_name),
                            input_lines: Some(input_lines),
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

    PreToolUseResult {
        decision: PermissionDecision::Allow,
        reason: None,
        modified_input: None,
        elapsed_ms: None,
        hook_name: None,
        tool_name: None,
        input_lines: None,
        continue_execution: None,
        stop_reason: None,
        suppress_output: None,
        system_message: None,
    }
}

// ฟังก์ชันช่วยเหลือ
fn find_matching_hooks(config: &ClaudeHooksConfig, event_type: &str, _pattern: &str) -> Vec<crate::hooks::claude_code_hooks::types::HookMatcher> {
    match event_type {
        "PreToolUse" => config.pre_tool_use.clone().unwrap_or_default(),
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