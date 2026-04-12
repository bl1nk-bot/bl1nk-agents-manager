pub mod constants;
pub mod types;
pub mod storage;

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::fs;
use crate::hooks::directory_readme_injector::constants::README_FILENAME;
use crate::hooks::directory_readme_injector::storage::{load_injected_paths, save_injected_paths, clear_injected_paths};

pub struct DirectoryReadmeInjectorHook {
    session_caches: Arc<RwLock<HashMap<String, HashSet<String>>>>,
    base_directory: PathBuf,
}

impl DirectoryReadmeInjectorHook {
    pub fn new(base_directory: PathBuf) -> Self {
        Self {
            session_caches: Arc::new(RwLock::new(HashMap::new())),
            base_directory,
        }
    }

    async fn get_session_cache(&self, session_id: &str) -> HashSet<String> {
        let mut caches = self.session_caches.write().await;
        if let Some(cache) = caches.remove(session_id) {
            return cache;
        }
        load_injected_paths(session_id)
    }

    fn find_readme_md_up(&self, start_dir: &Path) -> Vec<PathBuf> {
        let mut found = Vec::new();
        let mut current = start_dir;

        loop {
            let readme_path = current.join(README_FILENAME);
            if readme_path.is_file() {
                found.push(readme_path);
            }

            if current == self.base_directory { break; }
            match current.parent() {
                Some(parent) => {
                    if !parent.starts_with(&self.base_directory) { break; }
                    current = parent;
                }
                None => break,
            }
        }
        found.reverse();
        found
    }

    pub async fn process_file_for_injection(
        &self,
        file_path: &str,
        session_id: &str,
        output: &mut String,
    ) -> anyhow::Result<()> {
        let resolved = if file_path.starts_with('/') {
            PathBuf::from(file_path)
        } else {
            self.base_directory.join(file_path)
        };

        if !resolved.exists() { return Ok(()); }

        let dir = resolved.parent().unwrap_or(&self.base_directory);
        let mut cache = self.get_session_cache(session_id).await;
        let readme_paths = self.find_readme_md_up(dir);

        let mut injected_any = false;
        for path in readme_paths {
            let readme_dir = path.parent().unwrap().to_str().unwrap_or("").to_string();
            if cache.contains(&readme_dir) { continue; }

            if let Ok(content) = fs::read_to_string(&path) {
                output.push_str(&format!(
                    "\n\n[Project README: {:?}]\n{}",
                    path,
                    content
                ));
                cache.insert(readme_dir);
                injected_any = true;
            }
        }

        if injected_any {
            save_injected_paths(session_id, &cache)?;
        }
        self.session_caches.write().await.insert(session_id.to_string(), cache);
        Ok(())
    }
}
