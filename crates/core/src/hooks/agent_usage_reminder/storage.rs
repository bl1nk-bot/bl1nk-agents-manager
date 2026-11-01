use std::fs;
use std::path::PathBuf;
use crate::hooks::agent_usage_reminder::constants::agent_usage_reminder_storage;
use crate::hooks::agent_usage_reminder::types::AgentUsageState;

fn get_storage_path(session_id: &str) -> PathBuf {
    agent_usage_reminder_storage().join(format!("{}.json", session_id))
}

pub fn load_agent_usage_state(session_id: &str) -> Option<AgentUsageState> {
    let file_path = get_storage_path(session_id);
    if !file_path.exists() {
        return None;
    }

    match fs::read_to_string(file_path) {
        Ok(content) => serde_json::from_str(&content).ok(),
        Err(_) => None,
    }
}

pub fn save_agent_usage_state(state: &AgentUsageState) -> anyhow::Result<()> {
    let storage_dir = agent_usage_reminder_storage();
    if !storage_dir.exists() {
        fs::create_dir_all(storage_dir)?;
    }

    let file_path = get_storage_path(&state.session_id);
    let content = serde_json::to_string_pretty(state)?;
    fs::write(file_path, content)?;
    Ok(())
}

pub fn clear_agent_usage_state(session_id: &str) {
    let file_path = get_storage_path(session_id);
    if file_path.exists() {
        let _ = fs::remove_file(file_path);
    }
}
