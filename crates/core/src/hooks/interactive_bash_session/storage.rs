use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;
use tokio;

use crate::hooks::interactive_bash_session::types::{InteractiveBashSessionState, SerializedInteractiveBashSessionState};
use crate::hooks::interactive_bash_session::constants::get_interactive_bash_session_storage;

fn get_storage_path(session_id: &str) -> String {
    let storage_dir = get_interactive_bash_session_storage();
    format!("{}/{}.json", storage_dir, session_id)
}

pub async fn load_interactive_bash_session_state(
    session_id: &str,
) -> Option<InteractiveBashSessionState> {
    let file_path = get_storage_path(session_id);
    
    if !std::path::Path::new(&file_path).exists() {
        return None;
    }

    match tokio::fs::read_to_string(&file_path).await {
        Ok(content) => {
            if let Ok(serialized) = serde_json::from_str::<SerializedInteractiveBashSessionState>(&content) {
                let mut tmux_sessions = HashSet::new();
                for session in serialized.tmux_sessions {
                    tmux_sessions.insert(session);
                }
                
                Some(InteractiveBashSessionState {
                    session_id: serialized.session_id,
                    tmux_sessions,
                    updated_at: serialized.updated_at,
                })
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

pub async fn save_interactive_bash_session_state(
    state: &InteractiveBashSessionState,
) -> Result<(), Box<dyn std::error::Error>> {
    let storage_dir = get_interactive_bash_session_storage();
    
    // สร้างไดเรกทอรีถ้ายังไม่มี
    if !std::path::Path::new(&storage_dir).exists() {
        tokio::fs::create_dir_all(&storage_dir).await?;
    }

    let file_path = get_storage_path(&state.session_id);
    let serialized = SerializedInteractiveBashSessionState {
        session_id: state.session_id.clone(),
        tmux_sessions: state.tmux_sessions.iter().cloned().collect(),
        updated_at: state.updated_at,
    };
    
    let json_content = serde_json::to_string_pretty(&serialized)?;
    tokio::fs::write(&file_path, json_content).await?;
    
    Ok(())
}

pub async fn clear_interactive_bash_session_state(session_id: &str) -> Result<(), std::io::Error> {
    let file_path = get_storage_path(session_id);
    
    if std::path::Path::new(&file_path).exists() {
        tokio::fs::remove_file(&file_path).await?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[tokio::test]
    async fn test_save_and_load_session_state() {
        let mut tmux_sessions = HashSet::new();
        tmux_sessions.insert("session1".to_string());
        tmux_sessions.insert("session2".to_string());
        
        let state = InteractiveBashSessionState {
            session_id: "test_session".to_string(),
            tmux_sessions,
            updated_at: 1234567890,
        };

        // บันทึกสถานะ
        let result = save_interactive_bash_session_state(&state).await;
        assert!(result.is_ok());

        // โหลดสถานะ
        let loaded_state = load_interactive_bash_session_state("test_session").await;
        assert!(loaded_state.is_some());
        
        let loaded = loaded_state.unwrap();
        assert_eq!(loaded.session_id, "test_session");
        assert_eq!(loaded.updated_at, 1234567890);
        assert_eq!(loaded.tmux_sessions.len(), 2);
        assert!(loaded.tmux_sessions.contains("session1"));
        assert!(loaded.tmux_sessions.contains("session2"));

        // ล้างสถานะ
        let result = clear_interactive_bash_session_state("test_session").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_clear_nonexistent_session() {
        // ลบ session ที่ไม่มีอยู่ควรจะไม่เกิด error
        let result = clear_interactive_bash_session_state("nonexistent_session").await;
        assert!(result.is_ok());
    }
}