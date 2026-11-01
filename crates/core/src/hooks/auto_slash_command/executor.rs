use std::path::{Path, PathBuf};
use std::fs;
use crate::hooks::auto_slash_command::types::ParsedSlashCommand;
use anyhow::{Result, Context};

#[derive(Debug, Clone)]
pub struct CommandInfo {
    pub name: String,
    pub path: Option<PathBuf>,
    pub description: String,
    pub content: String,
    pub scope: String,
}

pub async fn execute_slash_command(
    parsed: &ParsedSlashCommand,
    base_dir: &Path
) -> Result<String> {
    // 1. Find command file (Simple version: check commands/ folder)
    let command_path = base_dir.join("commands").join(format!("{}.md", parsed.command));
    
    if !command_path.exists() {
        anyhow::bail!("Command /{} not found", parsed.command);
    }

    let content = fs::read_to_string(&command_path)?;
    
    // 2. Format Template
    let mut output = String::new();
    output.push_str(&format!("# /{} Command\n\n", parsed.command));
    output.push_str(&format!("**User Arguments**: {}\n", parsed.args));
    output.push_str("---\n## Command Instructions\n\n");
    output.push_str(&content);
    
    if !parsed.args.is_empty() {
        output.push_str("\n\n---\n## User Request\n");
        output.push_str(&parsed.args);
    }

    Ok(output)
}

