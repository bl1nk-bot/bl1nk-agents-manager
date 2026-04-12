use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio;
use serde_json::Value;
use regex::Regex;

use crate::hooks::hook_message_injector::{find_nearest_message_with_fields, find_first_message_with_agent, MESSAGE_STORAGE_DIR};
use crate::hooks::claude_code_session_state::get_session_agent;
use crate::hooks::shared::system_directive::{create_system_directive, SystemDirectiveType};
use crate::hooks::shared::agent_display_names::get_agent_display_name;

pub const HOOK_NAME: &str = "prometheus-md-only";

pub const PROMETHEUS_AGENTS: &[&str] = &["prometheus"];

pub const ALLOWED_EXTENSIONS: &[&str] = &[".md"];

pub const ALLOWED_PATH_PREFIX: &str = ".sisyphus";

pub const BLOCKED_TOOLS: &[&str] = &["Write", "Edit", "write", "edit"];

pub const PLANNING_CONSULT_WARNING: &str = "

---

[SYSTEM_DIRECTIVE:prometheus_read_only]

You are being invoked by Prometheus (Prometheus), a READ-ONLY planning agent.

**CRITICAL CONSTRAINTS:**
- DO NOT modify any files (no Write, Edit, or any file mutations)
- DO NOT execute commands that change system state
- DO NOT create, delete, or rename files
- ONLY provide analysis, recommendations, and information

**YOUR ROLE**: Provide consultation, research, and analysis to assist with planning.
Return your findings and recommendations. The actual implementation will be handled separately after planning is complete.

---

";

pub const PROMETHEUS_WORKFLOW_REMINDER: &str = "

---

[SYSTEM_DIRECTIVE:prometheus_read_only]

## PROMETHEUS MANDATORY WORKFLOW REMINDER

**You are writing a work plan. STOP AND VERIFY you completed ALL steps:**

┌─────────────────────────────────────────────────────────────────────┐
│                     PROMETHEUS WORKFLOW                             │
├──────┬──────────────────────────────────────────────────────────────┤
│  1   │ INTERVIEW: Full consultation with user                       │
│      │    - Gather ALL requirements                                 │
│      │    - Clarify ambiguities                                     │
│      │    - Record decisions to .sisyphus/drafts/                   │
├──────┼──────────────────────────────────────────────────────────────┤
│  2   │ METIS CONSULTATION: Pre-generation gap analysis              │
│      │    - delegate_task(agent=\"Metis (Plan Consultant)\", ...) │
│      │    - Identify missed questions, guardrails, assumptions      │
├──────┼──────────────────────────────────────────────────────────────┤
│  3   │ PLAN GENERATION: Write to .sisyphus/plans/*.md               │
│      │    <- YOU ARE HERE                                           │
├──────┼──────────────────────────────────────────────────────────────┤
│  4   │ MOMUS REVIEW (if high accuracy requested)                    │
│      │    - delegate_task(agent=\"Momus (Plan Reviewer)\", ...)     │
│      │    - Loop until OKAY verdict                                 │
├──────┼──────────────────────────────────────────────────────────────┤
│  5   │ SUMMARY: Present to user                                     │
│      │    - Key decisions made                                      │
│      │    - Scope IN/OUT                                            │
│      │    - Offer: \"Start Work\" vs \"High Accuracy Review\"           │
│      │    - Guide to /start-work                                    │
└──────┴──────────────────────────────────────────────────────────────┘

**DID YOU COMPLETE STEPS 1-2 BEFORE WRITING THIS PLAN?**
**AFTER WRITING, WILL YOU DO STEPS 4-5?**

If you skipped steps, STOP NOW. Go back and complete them.

---

";

fn is_allowed_file_path(file_path: &str, workspace_root: &str) -> bool {
    // 1. Resolve to absolute path
    let resolved = std::path::absolute(Path::new(workspace_root).join(file_path))
        .unwrap_or_else(|_| PathBuf::from(file_path));

    // 2. Get relative path from workspace root
    let rel = pathdiff::diff_paths(&resolved, workspace_root)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| file_path.to_string());

    // 3. Reject if escapes root (starts with ".." or is absolute)
    if rel.starts_with("..") || Path::new(&rel).is_absolute() {
        return false;
    }

    // 4. Check if .sisyphus/ or .sisyphus\ exists anywhere in the path (case-insensitive)
    let rel_lower = rel.to_lowercase();
    if !rel_lower.contains(".sisyphus/") && !rel_lower.contains(".sisyphus\\") {
        return false;
    }

    // 5. Check extension matches one of ALLOWED_EXTENSIONS (case-insensitive)
    if let Some(ext) = resolved.extension() {
        let ext_lower = ext.to_string_lossy().to_lowercase();
        if !ALLOWED_EXTENSIONS.iter().any(|&allowed_ext| allowed_ext == &format!(".{}", ext_lower)) {
            return false;
        }
    } else {
        return false; // No extension, not allowed
    }

    true
}

fn get_message_dir(session_id: &str) -> Option<String> {
    if !std::path::Path::new(MESSAGE_STORAGE_DIR).exists() {
        return None;
    }

    let direct_path = format!("{}/{}", MESSAGE_STORAGE_DIR, session_id);
    if std::path::Path::new(&direct_path).exists() {
        return Some(direct_path);
    }

    if let Ok(entries) = std::fs::read_dir(MESSAGE_STORAGE_DIR) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    let session_path = format!("{}/{}/{}", MESSAGE_STORAGE_DIR, entry.file_name().to_string_lossy(), session_id);
                    if std::path::Path::new(&session_path).exists() {
                        return Some(session_path);
                    }
                }
            }
        }
    }

    None
}

const TASK_TOOLS: &[&str] = &["delegate_task", "task", "call_omo_agent"];

fn get_agent_from_message_files(session_id: &str) -> Option<String> {
    if let Some(message_dir) = get_message_dir(session_id) {
        if let Some(agent) = find_first_message_with_agent(&message_dir) {
            return Some(agent);
        }
        if let Some(nearest_msg) = find_nearest_message_with_fields(&message_dir) {
            return nearest_msg.agent;
        }
    }
    None
}

fn get_agent_from_session(session_id: &str) -> Option<String> {
    get_session_agent(session_id).or_else(|| get_agent_from_message_files(session_id))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecuteInput {
    pub tool: String,
    pub session_id: String,
    pub call_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecuteOutput {
    pub args: HashMap<String, serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

pub struct PrometheusMdOnlyHook {
    workspace_root: String,
}

impl PrometheusMdOnlyHook {
    pub fn new(workspace_root: String) -> Self {
        Self {
            workspace_root,
        }
    }

    pub async fn on_tool_execute_before(
        &self,
        input: &ToolExecuteInput,
        output: &mut ToolExecuteOutput,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let agent_name = get_agent_from_session(&input.session_id);

        if let Some(ref agent) = agent_name {
            if !PROMETHEUS_AGENTS.contains(&agent.as_str()) {
                return Ok(());
            }
        } else {
            return Ok(());
        }

        let tool_name = &input.tool;

        // Inject read-only warning for task tools called by Prometheus
        if TASK_TOOLS.contains(&tool_name.as_str()) {
            if let Some(Value::String(prompt)) = output.args.get_mut("prompt") {
                if !prompt.contains("[SYSTEM_DIRECTIVE:") {
                    let new_prompt = format!("{}{}", PLANNING_CONSULT_WARNING, prompt);
                    output.args.insert("prompt".to_string(), Value::String(new_prompt));
                    
                    log::info!("[{}] Injected read-only planning warning to {}", 
                              HOOK_NAME, tool_name);
                }
            }
            return Ok(());
        }

        if !BLOCKED_TOOLS.contains(&tool_name.as_str()) {
            return Ok(());
        }

        let file_path = output.args.get("filePath")
            .or_else(|| output.args.get("path"))
            .or_else(|| output.args.get("file"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        if file_path.is_none() {
            return Ok(());
        }

        let file_path = file_path.unwrap();
        if !is_allowed_file_path(&file_path, &self.workspace_root) {
            log::warn!("Blocked: Prometheus can only write to .sisyphus/*.md. sessionID={}, tool={}, filePath={}, agent={}", 
                      input.session_id,
                      tool_name,
                      file_path,
                      agent_name.as_deref().unwrap_or("unknown"));

            let error_msg = format!(
                "[{}] {} can only write/edit .md files inside .sisyphus/ directory. Attempted to modify: {}. {} is a READ-ONLY planner. Use /start-work to execute the plan. APOLOGIZE TO THE USER, REMIND OF YOUR PLAN WRITING PROCESSES, TELL USER WHAT YOU WILL GOING TO DO AS THE PROCESS, WRITE THE PLAN",
                HOOK_NAME,
                get_agent_display_name("prometheus"),
                file_path,
                get_agent_display_name("prometheus")
            );
            
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, error_msg)));
        }

        let normalized_path = file_path.to_lowercase().replace('\\', "/");
        if normalized_path.contains(".sisyphus/plans/") {
            log::info!("[{}] Injecting workflow reminder for plan write. sessionID={}, tool={}, filePath={}, agent={}",
                      HOOK_NAME,
                      input.session_id,
                      tool_name,
                      file_path,
                      agent_name.as_deref().unwrap_or("unknown"));
            
            let current_message = output.message.as_ref().unwrap_or(&String::new()).clone();
            output.message = Some(format!("{}{}", current_message, PROMETHEUS_WORKFLOW_REMINDER));
        }

        log::info!("[{}] Allowed: .sisyphus/*.md write permitted. sessionID={}, tool={}, filePath={}, agent={}",
                  HOOK_NAME,
                  input.session_id,
                  tool_name,
                  file_path,
                  agent_name.as_deref().unwrap_or("unknown"));

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_allowed_file_path() {
        let temp_dir = tempdir::TempDir::new("test").unwrap();
        let root = temp_dir.path().to_str().unwrap();

        // Create test files
        std::fs::create_dir_all(format!("{}/.sisyphus", root)).unwrap();
        std::fs::write(format!("{}/.sisyphus/test.md", root), "test").unwrap();

        // Valid path
        assert!(is_allowed_file_path(".sisyphus/test.md", root));

        // Invalid extension
        assert!(!is_allowed_file_path(".sisyphus/test.txt", root));

        // Invalid path (not in .sisyphus)
        assert!(!is_allowed_file_path("test.md", root));

        // Path traversal attempt
        assert!(!is_allowed_file_path("../../outside.md", root));
    }
}