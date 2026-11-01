use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio;
use std::collections::HashMap;

use crate::hooks::start_work::types::{BoulderState, PlanProgress};
use crate::hooks::start_work::constants::{PROMETHEUS_PLANS_DIR, BOULDER_STATE_FILE};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedBoulderState {
    pub plan_name: String,
    pub active_plan: String,
    pub session_ids: Vec<String>,
    pub started_at: String,
    pub updated_at: String,
}

pub async fn read_boulder_state(directory: &str) -> Option<BoulderState> {
    let state_path = format!("{}/{}", directory, BOULDER_STATE_FILE);
    
    if !tokio::fs::metadata(&state_path).await.is_ok() {
        return None;
    }

    match tokio::fs::read_to_string(&state_path).await {
        Ok(content) => {
            if let Ok(state) = serde_json::from_str::<SerializedBoulderState>(&content) {
                Some(BoulderState {
                    plan_name: state.plan_name,
                    active_plan: state.active_plan,
                    session_ids: state.session_ids,
                    started_at: state.started_at,
                    updated_at: state.updated_at,
                })
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

pub async fn write_boulder_state(directory: &str, state: &BoulderState) -> Result<(), std::io::Error> {
    let state_path = format!("{}/{}", directory, BOULDER_STATE_FILE);
    let serialized = SerializedBoulderState {
        plan_name: state.plan_name.clone(),
        active_plan: state.active_plan.clone(),
        session_ids: state.session_ids.clone(),
        started_at: state.started_at.clone(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };
    
    let json_content = serde_json::to_string_pretty(&serialized)?;
    tokio::fs::write(&state_path, json_content).await?;
    Ok(())
}

pub async fn append_session_id(directory: &str, session_id: &str) -> Result<(), std::io::Error> {
    if let Some(mut state) = read_boulder_state(directory).await {
        if !state.session_ids.contains(&session_id.to_string()) {
            state.session_ids.push(session_id.to_string());
            state.updated_at = chrono::Utc::now().to_rfc3339();
            write_boulder_state(directory, &state).await?;
        }
    }
    Ok(())
}

pub async fn find_prometheus_plans(directory: &str) -> Vec<String> {
    let plans_dir = format!("{}/{}", directory, PROMETHEUS_PLANS_DIR);
    let mut plans = Vec::new();
    
    if let Ok(mut entries) = tokio::fs::read_dir(&plans_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if entry.file_type().await.map(|ft| ft.is_file()).unwrap_or(false) {
                if let Some(ext) = entry.path().extension() {
                    if ext == "md" {
                        if let Some(path_str) = entry.path().to_str() {
                            plans.push(path_str.to_string());
                        }
                    }
                }
            }
        }
    }
    
    plans
}

pub async fn get_plan_progress(plan_path: &str) -> PlanProgress {
    // ฟังก์ชันจำลอง - ในระบบจริงจะต้องวิเคราะห์เนื้อหาของไฟล์แผน
    // เพื่อหาจำนวนงานที่ทำแล้วและทั้งหมด
    PlanProgress {
        completed: 0,
        total: 10, // ค่าจำลอง
        is_complete: false,
    }
}

pub async fn create_boulder_state(plan_path: &str, session_id: &str) -> BoulderState {
    let now = chrono::Utc::now().to_rfc3339();
    
    // ดึงชื่อแผนจาก path
    let plan_name = std::path::Path::new(plan_path)
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("")
        .to_string();
    
    BoulderState {
        plan_name,
        active_plan: plan_path.to_string(),
        session_ids: vec![session_id.to_string()],
        started_at: now.clone(),
        updated_at: now,
    }
}

pub async fn get_plan_name(plan_path: &str) -> String {
    std::path::Path::new(plan_path)
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("")
        .to_string()
}

pub async fn clear_boulder_state(directory: &str) -> Result<(), std::io::Error> {
    let state_path = format!("{}/{}", directory, BOULDER_STATE_FILE);
    if tokio::fs::metadata(&state_path).await.is_ok() {
        tokio::fs::remove_file(&state_path).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_boulder_state_operations() {
        let temp_dir = TempDir::new().unwrap();
        let dir = temp_dir.path().to_str().unwrap();

        // สร้างสถานะใหม่
        let plan_path = format!("{}/test_plan.md", dir);
        tokio::fs::create_dir_all(format!("{}/.sisyphus/plans", dir)).await.unwrap();
        tokio::fs::write(&plan_path, "# Test Plan\n- [ ] Task 1\n- [x] Task 2").await.unwrap();

        let session_id = "test_session_123";
        let state = create_boulder_state(&plan_path, session_id).await;

        assert_eq!(state.plan_name, "test_plan");
        assert_eq!(state.active_plan, plan_path);
        assert_eq!(state.session_ids, vec![session_id.to_string()]);
        assert!(!state.session_ids.is_empty());

        // บันทึกสถานะ
        let result = write_boulder_state(dir, &state).await;
        assert!(result.is_ok());

        // อ่านสถานะ
        let loaded_state = read_boulder_state(dir).await;
        assert!(loaded_state.is_some());
        let loaded = loaded_state.unwrap();
        assert_eq!(loaded.plan_name, "test_plan");
        assert_eq!(loaded.session_ids, vec![session_id.to_string()]);

        // เพิ่ม session ID
        let new_session_id = "test_session_456";
        let result = append_session_id(dir, new_session_id).await;
        assert!(result.is_ok());

        let updated_state = read_boulder_state(dir).await;
        assert!(updated_state.is_some());
        let updated = updated_state.unwrap();
        assert_eq!(updated.session_ids.len(), 2);
        assert!(updated.session_ids.contains(&session_id.to_string()));
        assert!(updated.session_ids.contains(&new_session_id.to_string()));

        // ล้างสถานะ
        let result = clear_boulder_state(dir).await;
        assert!(result.is_ok());

        let cleared_state = read_boulder_state(dir).await;
        assert!(cleared_state.is_none());
    }

    #[tokio::test]
    async fn test_find_prometheus_plans() {
        let temp_dir = TempDir::new().unwrap();
        let dir = temp_dir.path().to_str().unwrap();

        // สร้างไดเรกทอรีและไฟล์แผน
        let plans_dir = format!("{}/.sisyphus/plans", dir);
        tokio::fs::create_dir_all(&plans_dir).await.unwrap();
        
        let plan1_path = format!("{}/plan1.md", plans_dir);
        let plan2_path = format!("{}/plan2.md", plans_dir);
        let non_plan_path = format!("{}/not_a_plan.txt", plans_dir);
        
        tokio::fs::write(&plan1_path, "# Plan 1").await.unwrap();
        tokio::fs::write(&plan2_path, "# Plan 2").await.unwrap();
        tokio::fs::write(&non_plan_path, "Not a plan").await.unwrap();

        let plans = find_prometheus_plans(dir).await;
        assert_eq!(plans.len(), 2);
        assert!(plans.contains(&plan1_path));
        assert!(plans.contains(&plan2_path));
    }
}