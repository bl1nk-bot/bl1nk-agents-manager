use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashSet;
use chrono::Utc;
use crate::hooks::directory_agents_injector::constants::agents_injector_storage;
use crate::hooks::directory_agents_injector::types::InjectedPathsData;

fn get_storage_path(session_id: &str) -> PathBuf {
    agents_injector_storage().join(format!("{}.json", session_id))
}

pub fn load_injected_paths(session_id: &str) -> HashSet<String> {
    let file_path = get_storage_path(session_id);
    if !file_path.exists() {
        return HashSet::new();
    }

    match fs::read_to_string(file_path) {
        Ok(content) => {
            if let Ok(data) = serde_json::from_str::<InjectedPathsData>(&content) {
                data.injected_paths.into_iter().collect()
            } else {
                HashSet::new()
            }
        }
        Err(_) => HashSet::new(),
    }
}

pub fn save_injected_paths(session_id: &str, paths: &HashSet<String>) -> anyhow::Result<()> {
    let storage_dir = agents_injector_storage();
    if !storage_dir.exists() {
        fs::create_dir_all(storage_dir)?;
    }

    let data = InjectedPathsData {
        session_id: session_id.to_string(),
        injected_paths: paths.iter().cloned().collect(),
        updated_at: Utc::now().timestamp_millis(),
    };

    let content = serde_json::to_string_pretty(&data)?;
    fs::write(get_storage_path(session_id), content)?;
    Ok(())
}

pub fn clear_injected_paths(session_id: &str) {
    let file_path = get_storage_path(session_id);
    if file_path.exists() {
        let _ = fs::remove_file(file_path);
    }
}
