use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio;
use std::fs;
use std::io::Write;
use std::time::SystemTime;
use uuid::Uuid;
use chrono::Utc;

use crate::hooks::claude_code_hooks::types::TranscriptEntry;

// ไดเรกทอรีสำหรับจัดเก็บ transcript
static TRANSCRIPT_DIR: &str = ".bl1nk/transcripts";

pub fn get_transcript_path(session_id: &str) -> String {
    format!("{}/{}.jsonl", TRANSCRIPT_DIR, session_id)
}

async fn ensure_transcript_dir() -> Result<(), std::io::Error> {
    let path = PathBuf::from(TRANSCRIPT_DIR);
    if !path.exists() {
        tokio::fs::create_dir_all(path).await?;
    }
    Ok(())
}

pub async fn append_transcript_entry(session_id: &str, entry: &TranscriptEntry) -> Result<(), Box<dyn std::error::Error>> {
    ensure_transcript_dir().await?;
    let path = get_transcript_path(session_id);
    
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .await?;
    
    let line = format!("{}\n", serde_json::to_string(entry)?);
    file.write_all(line.as_bytes()).await?;
    
    Ok(())
}

pub async fn record_tool_use(
    session_id: &str,
    tool_name: &str,
    tool_input: serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    let entry = TranscriptEntry {
        entry_type: "tool_use".to_string(),
        timestamp: Utc::now().to_rfc3339(),
        tool_name: Some(tool_name.to_string()),
        tool_input: Some(tool_input.as_object().unwrap_or(&std::collections::HashMap::new()).clone()),
        tool_output: None,
        content: None,
    };
    
    append_transcript_entry(session_id, &entry).await?;
    Ok(())
}

pub async fn record_tool_result(
    session_id: &str,
    tool_name: &str,
    tool_input: serde_json::Value,
    tool_output: serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    let entry = TranscriptEntry {
        entry_type: "tool_result".to_string(),
        timestamp: Utc::now().to_rfc3339(),
        tool_name: Some(tool_name.to_string()),
        tool_input: Some(tool_input.as_object().unwrap_or(&std::collections::HashMap::new()).clone()),
        tool_output: Some(tool_output.as_object().unwrap_or(&std::collections::HashMap::new()).clone()),
        content: None,
    };
    
    append_transcript_entry(session_id, &entry).await?;
    Ok(())
}

pub async fn record_user_message(
    session_id: &str,
    content: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let entry = TranscriptEntry {
        entry_type: "user".to_string(),
        timestamp: Utc::now().to_rfc3339(),
        tool_name: None,
        tool_input: None,
        tool_output: None,
        content: Some(content.to_string()),
    };
    
    append_transcript_entry(session_id, &entry).await?;
    Ok(())
}

pub async fn record_assistant_message(
    session_id: &str,
    content: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let entry = TranscriptEntry {
        entry_type: "assistant".to_string(),
        timestamp: Utc::now().to_rfc3339(),
        tool_name: None,
        tool_input: None,
        tool_output: None,
        content: Some(content.to_string()),
    };
    
    append_transcript_entry(session_id, &entry).await?;
    Ok(())
}

// ============================================================================
// ฟังก์ชันสำหรับสร้าง transcript จาก session (จำลอง)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenCodeMessagePart {
    #[serde(rename = "type")]
    part_type: String,
    tool: Option<String>,
    state: Option<OpenCodeState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenCodeState {
    status: Option<String>,
    input: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenCodeMessage {
    info: Option<OpenCodeInfo>,
    parts: Option<Vec<OpenCodeMessagePart>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenCodeInfo {
    role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DisabledTranscriptEntry {
    #[serde(rename = "type")]
    entry_type: String,
    message: DisabledMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DisabledMessage {
    role: String,
    content: Vec<DisabledContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DisabledContent {
    #[serde(rename = "type")]
    content_type: String,
    name: String,
    input: serde_json::Value,
}

/// สร้าง transcript จาก session (จำลอง)
pub async fn build_transcript_from_session(
    session_id: &str,
    directory: &str,
    current_tool_name: &str,
    current_tool_input: serde_json::Value,
) -> Option<String> {
    // สร้างรายการ entries
    let mut entries = Vec::new();
    
    // ลองดึงข้อมูลจาก session (จำลอง)
    // ในระบบที่แท้จริง ควรเรียก API เพื่อดึงข้อมูล session
    
    // เพิ่ม entry สำหรับเครื่องมือปัจจุบันเสมอ
    let current_entry = DisabledTranscriptEntry {
        entry_type: "assistant".to_string(),
        message: DisabledMessage {
            role: "assistant".to_string(),
            content: vec![DisabledContent {
                content_type: "tool_use".to_string(),
                name: transform_tool_name(current_tool_name),
                input: current_tool_input,
            }],
        },
    };
    
    entries.push(serde_json::to_string(&current_entry).unwrap());
    
    // เขียนไปยังไฟล์ชั่วคราว
    let temp_path = std::env::temp_dir()
        .join(format!("opencode-transcript-{}-{}.jsonl", 
                     session_id, 
                     Uuid::new_v4().to_string()));
    
    let temp_path_str = temp_path.to_string_lossy().to_string();
    
    if let Ok(transcript_content) = std::fs::File::create(&temp_path_str) {
        let mut file = std::io::BufWriter::new(transcript_content);
        for entry in entries {
            writeln!(file, "{}", entry).ok()?;
        }
    } else {
        return None;
    }
    
    Some(temp_path_str)
}

/// ลบไฟล์ transcript ชั่วคราว
pub fn delete_temp_transcript(path: Option<&str>) {
    if let Some(path_str) = path {
        let path = PathBuf::from(path_str);
        if path.exists() {
            let _ = std::fs::remove_file(path);
        }
    }
}

// ฟังก์ชันช่วยเหลือ
fn transform_tool_name(tool_name: &str) -> String {
    // แปลงชื่อเครื่องมือ (จำลอง)
    // ในระบบที่แท้จริง ควรใช้ฟังก์ชันจาก shared module
    tool_name.replace("-", "_").to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_transcript_functions() {
        let session_id = "test_session_123";
        
        // ทดสอบการบันทึก tool use
        let tool_input = serde_json::json!({"param": "value"});
        let result = record_tool_use(session_id, "test_tool", tool_input).await;
        assert!(result.is_ok());
        
        // ทดสอบการบันทึก tool result
        let tool_output = serde_json::json!({"result": "success"});
        let result = record_tool_result(session_id, "test_tool", 
                                       serde_json::json!({"param": "value"}), 
                                       tool_output).await;
        assert!(result.is_ok());
        
        // ทดสอบการบันทึกข้อความผู้ใช้
        let result = record_user_message(session_id, "Hello").await;
        assert!(result.is_ok());
        
        // ทดสอบการบันทึกข้อความผู้ช่วย
        let result = record_assistant_message(session_id, "Hi there").await;
        assert!(result.is_ok());
    }
}