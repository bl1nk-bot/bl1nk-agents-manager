use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RuleMetadata {
    pub description: Option<String>,
    pub globs: Option<Vec<String>>,
    pub always_apply: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct RuleInfo {
    pub path: PathBuf,
    pub relative_path: PathBuf,
    pub distance: u32,
    pub content: String,
    pub content_hash: String,
    pub metadata: RuleMetadata,
    pub match_reason: String,
    pub real_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct RuleFileCandidate {
    pub path: PathBuf,
    pub real_path: PathBuf,
    pub is_global: bool,
    pub distance: u32,
    pub is_single_file: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InjectedRulesData {
    pub session_id: String,
    pub injected_hashes: Vec<String>,
    pub injected_real_paths: Vec<String>,
    pub updated_at: i64,
}
