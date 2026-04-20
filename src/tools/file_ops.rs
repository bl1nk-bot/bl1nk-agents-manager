//! File Operations Tools (read, write, replace)
//! พอร์ตจากมาตรฐาน Gemini CLI (v1.7.5.1)

use anyhow::{Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// ข้อมูลนำเข้าสำหรับ read_file
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReadFileArgs {
    pub file_path: String,
    pub offset: Option<usize>,
    pub limit: Option<usize>,
}

/// ข้อมูลนำเข้าสำหรับ write_file
#[derive(Debug, Deserialize, JsonSchema)]
pub struct WriteFileArgs {
    pub file_path: String,
    pub content: String,
}

/// ข้อมูลนำเข้าสำหรับ replace
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReplaceArgs {
    pub file_path: String,
    pub instruction: String,
    pub old_string: String,
    pub new_string: String,
    pub allow_multiple: Option<bool>,
}

pub struct FileOpsTools;

impl FileOpsTools {
    pub fn read_file(args: ReadFileArgs) -> Result<String> {
        let content = fs::read_to_string(&args.file_path)
            .with_context(|| format!("Failed to read file: {}", args.file_path))?;
        
        // จัดการ offset และ limit (การตัดบรรทัด)
        let lines: Vec<&str> = content.lines().collect();
        let start = args.offset.unwrap_or(0);
        let end = args.limit.map(|l| start + l).unwrap_or(lines.len());
        
        let result = lines.get(start..end.min(lines.len()))
            .map(|l| l.join("\n"))
            .unwrap_or_default();
            
        Ok(result)
    }

    pub fn write_file(args: WriteFileArgs) -> Result<String> {
        if let Some(parent) = Path::new(&args.file_path).parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&args.file_path, &args.content)?;
        Ok(format!("Successfully wrote {} bytes to {}", args.content.len(), args.file_path))
    }

    pub fn replace(args: ReplaceArgs) -> Result<String> {
        let content = fs::read_to_string(&args.file_path)?;
        let count = content.matches(&args.old_string).count();
        
        if count == 0 {
            anyhow::bail!("String not found: '{}'", args.old_string);
        }
        
        if count > 1 && !args.allow_multiple.unwrap_or(false) {
            anyhow::bail!("Found multiple occurrences of '{}'. Use allow_multiple=true to replace all.", args.old_string);
        }
        
        let new_content = content.replace(&args.old_string, &args.new_string);
        fs::write(&args.file_path, new_content)?;
        
        Ok(format!("Successfully replaced {} occurrence(s) in {}", count, args.file_path))
    }
}
