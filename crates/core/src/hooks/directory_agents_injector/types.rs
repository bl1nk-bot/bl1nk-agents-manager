use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InjectedPathsData {
    pub session_id: String,
    pub injected_paths: Vec<String>,
    pub updated_at: i64,
}
