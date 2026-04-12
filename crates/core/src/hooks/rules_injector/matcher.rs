use std::path::Path;
use glob::Pattern;
use sha2::{Sha256, Digest};
use crate::hooks::rules_injector::types::RuleMetadata;

pub struct MatchResult {
    pub applies: bool,
    pub reason: Option<String>,
}

pub fn should_apply_rule(
    metadata: &RuleMetadata,
    current_file_path: &Path,
    project_root: Option<&Path>,
) -> MatchResult {
    if metadata.always_apply == Some(true) {
        return MatchResult { applies: true, reason: Some("alwaysApply".to_string()) };
    }

    if let Some(globs) = &metadata.globs {
        let rel_path = if let Some(root) = project_root {
            current_file_path.strip_prefix(root).unwrap_or(current_file_path)
        } else {
            current_file_path
        };

        let path_str = rel_path.to_str().unwrap_or("");

        for pattern_str in globs {
            if let Ok(pattern) = Pattern::new(pattern_str) {
                if pattern.matches(path_str) {
                    return MatchResult { applies: true, reason: Some(format!("glob: {}", pattern_str)) };
                }
            }
        }
    }

    MatchResult { applies: false, reason: None }
}

pub fn create_content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    let result = hasher.finalize();
    format!("{:x}", result)[..16].to_string()
}
