use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio;
use std::fs;
use std::collections::HashMap;
use chrono::Utc;

use crate::hooks::claude_code_hooks::types::{TodoFile, TodoItem, ClaudeCodeTodoItem};

// ไดเรกทอรีสำหรับจัดเก็บไฟล์ todo
static TODO_DIR: &str = ".bl1nk/todos";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenCodeTodo {
    pub content: String,
    pub status: String,
    pub priority: String,
    pub id: String,
}

pub fn get_todo_path(session_id: &str) -> String {
    format!("{}/{}-agent-{}.json", TODO_DIR, session_id, session_id)
}

async fn ensure_todo_dir() -> Result<(), std::io::Error> {
    let path = PathBuf::from(TODO_DIR);
    if !path.exists() {
        tokio::fs::create_dir_all(path).await?;
    }
    Ok(())
}

fn to_claude_code_format(item: &OpenCodeTodo) -> ClaudeCodeTodoItem {
    ClaudeCodeTodoItem {
        content: item.content.clone(),
        status: if item.status == "cancelled" { "completed".to_string() } else { item.status.clone() },
        active_form: item.content.clone(),
    }
}

fn todo_item_to_claude_code_format(item: &TodoItem) -> ClaudeCodeTodoItem {
    ClaudeCodeTodoItem {
        content: item.content.clone(),
        status: if item.status == "cancelled" { "completed".to_string() } else { item.status.clone() },
        active_form: item.content.clone(),
    }
}

pub async fn load_todo_file(session_id: &str) -> Option<TodoFile> {
    let path = get_todo_path(session_id);
    let path_buf = PathBuf::from(&path);
    
    if !path_buf.exists() {
        return None;
    }

    match tokio::fs::read_to_string(&path).await {
        Ok(content) => {
            // พยายาม parse เป็น TodoFile ก่อน
            if let Ok(todo_file) = serde_json::from_str::<TodoFile>(&content) {
                return Some(todo_file);
            }

            // ถ้า parse ไม่ได้ ลอง parse เป็น array ของ ClaudeCodeTodoItem
            if let Ok(items) = serde_json::from_str::<Vec<ClaudeCodeTodoItem>>(&content) {
                let todo_items: Vec<TodoItem> = items.iter().enumerate().map(|(idx, item)| {
                    TodoItem {
                        id: idx.to_string(),
                        content: item.content.clone(),
                        status: item.status.clone(),
                        priority: None,
                        created_at: Utc::now().to_rfc3339(),
                        updated_at: Some(Utc::now().to_rfc3339()),
                    }
                }).collect();

                return Some(TodoFile {
                    session_id: session_id.to_string(),
                    items: todo_items,
                    created_at: Utc::now().to_rfc3339(),
                    updated_at: Utc::now().to_rfc3339(),
                });
            }

            None
        }
        Err(_) => None,
    }
}

pub async fn save_todo_file(session_id: &str, file: &TodoFile) -> Result<(), Box<dyn std::error::Error>> {
    ensure_todo_dir().await?;
    let path = get_todo_path(session_id);
    
    // แปลงเป็นรูปแบบ ClaudeCode
    let claude_code_format: Vec<ClaudeCodeTodoItem> = file.items.iter()
        .map(todo_item_to_claude_code_format)
        .collect();
    
    let json_content = serde_json::to_string_pretty(&claude_code_format)?;
    tokio::fs::write(&path, json_content).await?;
    
    Ok(())
}

pub async fn save_open_code_todos(session_id: &str, todos: &[OpenCodeTodo]) -> Result<(), Box<dyn std::error::Error>> {
    ensure_todo_dir().await?;
    let path = get_todo_path(session_id);
    
    // แปลงเป็นรูปแบบ ClaudeCode
    let claude_code_format: Vec<ClaudeCodeTodoItem> = todos.iter()
        .map(to_claude_code_format)
        .collect();
    
    let json_content = serde_json::to_string_pretty(&claude_code_format)?;
    tokio::fs::write(&path, json_content).await?;
    
    Ok(())
}

pub async fn delete_todo_file(session_id: &str) -> Result<(), std::io::Error> {
    let path = get_todo_path(session_id);
    let path_buf = PathBuf::from(&path);
    
    if path_buf.exists() {
        tokio::fs::remove_file(path_buf).await?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_todo_operations() {
        let session_id = "test_session_123";
        
        // สร้าง todo ตัวอย่าง
        let todo_item = OpenCodeTodo {
            content: "Test todo item".to_string(),
            status: "pending".to_string(),
            priority: "high".to_string(),
            id: "1".to_string(),
        };
        
        let todos = vec![todo_item];
        
        // บันทึก
        let result = save_open_code_todos(session_id, &todos).await;
        assert!(result.is_ok());
        
        // โหลด
        let loaded = load_todo_file(session_id).await;
        assert!(loaded.is_some());
        
        // ลบ
        let result = delete_todo_file(session_id).await;
        assert!(result.is_ok());
    }
}