use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashSet;
use chrono::Utc;
use crate::hooks::rules_injector::constants::rules_injector_storage;
use crate::hooks::rules_injector::types::InjectedRulesData;

fn get_storage_path(session_id: &str) -> PathBuf {
    rules_injector_storage().join(format!("{}.json", session_id))
}

pub fn load_injected_rules(session_id: &str) -> (HashSet<String>, HashSet<PathBuf>) {
    let file_path = get_storage_path(session_id);
    if !file_path.exists() {
        return (HashSet::new(), HashSet::new());
    }

    match fs::read_to_string(file_path) {
        Ok(content) => {
            if let Ok(data) = serde_json::from_str::<InjectedRulesData>(&content) {
                let hashes = data.injected_hashes.into_iter().collect();
                let paths = data.injected_real_paths.into_iter().map(PathBuf::from).collect();
                (hashes, paths)
            } else {
                (HashSet::new(), HashSet::new())
            }
        }
        Err(_) => (HashSet::new(), HashSet::new()),
    }
}

pub fn save_injected_rules(
    session_id: &str,
    content_hashes: &HashSet<String>,
    real_paths: &HashSet<PathBuf>
) -> anyhow::Result<()> {
    let storage_dir = rules_injector_storage();
    if !storage_dir.exists() {
        fs::create_dir_all(storage_dir)?;
    }

    let data = InjectedRulesData {
        session_id: session_id.to_string(),
        injected_hashes: content_hashes.iter().cloned().collect(),
        injected_real_paths: real_paths.iter().map(|p| p.to_str().unwrap_or("").to_string()).collect(),
        updated_at: Utc::now().timestamp_millis(),
    };

    let content = serde_json::to_string_pretty(&data)?;
    fs::write(get_storage_path(session_id), content)?;
    Ok(())
}

pub fn clear_injected_rules(session_id: &str) {
    let file_path = get_storage_path(session_id);
    if file_path.exists() {
        let _ = fs::remove_file(file_path);
    }
}
