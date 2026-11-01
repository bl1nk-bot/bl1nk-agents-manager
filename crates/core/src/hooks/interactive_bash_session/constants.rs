use std::path::PathBuf;

// ค่าคงที่สำหรับไดเรกทอรีจัดเก็บ
pub fn get_opencode_storage_dir() -> String {
    // ใช้ไดเรกทอรี home หรือ temp สำหรับจำลอง
    if cfg!(windows) {
        std::env::var("APPDATA").unwrap_or_else(|_| "/tmp".to_string())
    } else {
        std::env::var("HOME").map(|h| format!("{}/.bl1nk", h)).unwrap_or_else(|_| "/tmp/.bl1nk".to_string())
    }
}

pub fn get_interactive_bash_session_storage() -> String {
    let opencode_storage = get_opencode_storage_dir();
    format!("{}/interactive-bash-session", opencode_storage)
}

pub const OMO_SESSION_PREFIX: &str = "omo-";

pub fn build_session_reminder_message(sessions: &[String]) -> String {
    if sessions.is_empty() {
        return String::new();
    }
    
    let sessions_joined = sessions.join(", ");
    format!("\n\n[System Reminder] Active omo-* tmux sessions: {}", sessions_joined)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_session_reminder_message() {
        let sessions = vec!["session1".to_string(), "session2".to_string()];
        let result = build_session_reminder_message(&sessions);
        assert!(result.contains("session1"));
        assert!(result.contains("session2"));
        assert!(result.contains("System Reminder"));
        
        let empty_sessions: Vec<String> = vec![];
        let result = build_session_reminder_message(&empty_sessions);
        assert_eq!(result, "");
    }

    #[test]
    fn test_omo_session_prefix() {
        assert_eq!(OMO_SESSION_PREFIX, "omo-");
    }
}