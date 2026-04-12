pub mod constants;
pub mod types;
pub mod parser;
pub mod finder;
pub mod matcher;
pub mod storage;

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::fs;
use crate::hooks::rules_injector::finder::{find_project_root, find_rule_files};
use crate::hooks::rules_injector::matcher::{create_content_hash, should_apply_rule};
use crate::hooks::rules_injector::parser::parse_rule_frontmatter;
use crate::hooks::rules_injector::storage::{load_injected_rules, save_injected_rules, clear_injected_rules};

pub struct RulesInjectorHook {
    session_caches: Arc<RwLock<HashMap<String, SessionCache>>>,
    base_directory: PathBuf,
}

struct SessionCache {
    content_hashes: HashSet<String>,
    real_paths: HashSet<PathBuf>,
}

impl RulesInjectorHook {
    pub fn new(base_directory: PathBuf) -> Self {
        Self {
            session_caches: Arc::new(RwLock::new(HashMap::new())),
            base_directory,
        }
    }

    async fn get_session_cache(&self, session_id: &str) -> SessionCache {
        let mut caches = self.session_caches.write().await;
        if let Some(cache) = caches.remove(session_id) {
            return cache;
        }

        let (hashes, paths) = load_injected_rules(session_id);
        SessionCache {
            content_hashes: hashes,
            real_paths: paths,
        }
    }

    pub async fn process_file_for_injection(
        &self,
        file_path: &str,
        session_id: &str,
        output: &mut String
    ) -> anyhow::Result<()> {
        let resolved_path = if file_path.starts_with('/') {
            PathBuf::from(file_path)
        } else {
            self.base_directory.join(file_path)
        };

        if !resolved_path.exists() { return Ok(()); }

        let project_root = find_project_root(&resolved_path);
        let mut cache = self.get_session_cache(session_id).await;
        
        // Use empty path for home if not available
        let home = std::env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/tmp"));

        let rule_candidates = find_rule_files(project_root.as_deref(), &home, &resolved_path);
        let mut injected_any = false;

        for candidate in rule_candidates {
            if cache.real_paths.contains(&candidate.real_path) { continue; }

            if let Ok(raw_content) = fs::read_to_string(&candidate.path) {
                let parsed = parse_rule_frontmatter(&raw_content);
                
                let match_reason = if candidate.is_single_file {
                    "copilot-instructions (always apply)".to_string()
                } else {
                    let res = should_apply_rule(&parsed.metadata, &resolved_path, project_root.as_deref());
                    if !res.applies { continue; }
                    res.reason.unwrap_or_else(|| "matched".to_string())
                };

                let hash = create_content_hash(&parsed.body);
                if cache.content_hashes.contains(&hash) { continue; }

                // Injection
                let rel_path = if let Some(root) = &project_root {
                    candidate.path.strip_prefix(root).unwrap_or(&candidate.path).to_str().unwrap_or("")
                } else {
                    candidate.path.to_str().unwrap_or("")
                };

                output.push_str(&format!(
                    "\n\n[Rule: {}]\n[Match: {}]\n{}",
                    rel_path, match_reason, parsed.body
                ));

                cache.real_paths.insert(candidate.real_path);
                cache.content_hashes.insert(hash);
                injected_any = true;
            }
        }

        if injected_any {
            save_injected_rules(session_id, &cache.content_hashes, &cache.real_paths)?;
        }
        
        self.session_caches.write().await.insert(session_id.to_string(), cache);
        Ok(())
    }
}
