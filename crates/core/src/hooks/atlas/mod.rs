use std::path::{Path, PathBuf};
use regex::Regex;
use std::sync::OnceLock;

pub const HOOK_NAME: &str = "atlas";

pub fn is_sisyphus_path(path: &str) -> bool {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"\.sisyphus[/\]").unwrap());
    re.is_match(path)
}

pub struct AtlasHook {
    base_directory: PathBuf,
}

impl AtlasHook {
    pub fn new(base_directory: PathBuf) -> Self {
        Self { base_directory }
    }

    pub fn build_verification_reminder(&self, session_id: &str) -> String {
        format!(
            "**MANDATORY: VERIFY NOW**\n1. Run tests\n2. Check diagnostics\n3. Read code\n\nIf fails: `delegate_task(session_id=\"{}\", prompt=\"fix: ...\")`",
            session_id
        )
    }

    pub async fn on_tool_execute_before(&self, tool: &str, args: &serde_json::Value) -> Option<String> {
        let tool_lower = tool.to_lowercase();
        if tool_lower == "write" || tool_lower == "edit" {
            let path = args.get("path").or(args.get("filePath")).and_then(|v| v.as_str())?;
            if !is_sisyphus_path(path) {
                return Some(format!("WARNING: Direct modification of {} is forbidden outside .sisyphus/", path));
            }
        }
        None
    }
}

