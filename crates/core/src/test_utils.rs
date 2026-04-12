use std::env;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

use std::collections::HashMap;

#[cfg(test)]
pub mod builders {
    use crate::search::RecentChat;

    pub struct RecentChatBuilder {
        id: String,
        title: String,
        started_at_iso: String,
        message_count: u32,
        summary: Option<String>,
        last_activity_iso: Option<String>,
        total_tokens: Option<u32>,
        tags: Vec<String>,
    }

    impl RecentChatBuilder {
        pub fn new(id: &str) -> Self {
            Self {
                id: id.to_string(),
                title: "Chat Session".to_string(),
                started_at_iso: "2024-01-01T00:00:00Z".to_string(),
                message_count: 0,
                summary: None,
                last_activity_iso: None,
                total_tokens: None,
                tags: Vec::new(),
            }
        }

        pub fn with_title(mut self, title: &str) -> Self {
            self.title = title.to_string();
            self
        }

        pub fn with_message_count(mut self, count: u32) -> Self {
            self.message_count = count;
            self
        }

        pub fn with_summary(mut self, summary: &str) -> Self {
            self.summary = Some(summary.to_string());
            self
        }

        pub fn with_last_activity(mut self, last_activity_iso: &str) -> Self {
            self.last_activity_iso = Some(last_activity_iso.to_string());
            self
        }

        pub fn with_total_tokens(mut self, total_tokens: u32) -> Self {
            self.total_tokens = Some(total_tokens);
            self
        }

        pub fn with_tags(mut self, tags: Vec<String>) -> Self {
            self.tags = tags;
            self
        }

        pub fn build(self) -> RecentChat {
            RecentChat {
                id: self.id,
                title: self.title,
                started_at_iso: self.started_at_iso,
                message_count: self.message_count,
                summary: self.summary,
                last_activity_iso: self.last_activity_iso,
                total_tokens: self.total_tokens,
                tags: self.tags,
            }
        }
    }
}

pub struct EnvGuard {
    guards: HashMap<String, Option<String>>,
}

impl EnvGuard {
    pub fn new() -> Self {
        Self {
            guards: HashMap::new(),
        }
    }

    pub fn set<V: AsRef<str>>(&mut self, key: &str, value: V) {
        if !self.guards.contains_key(key) {
            self.guards.insert(key.to_string(), env::var(key).ok());
        }
        env::set_var(key, value.as_ref());
    }

    pub fn remove(&mut self, key: &str) {
        if !self.guards.contains_key(key) {
            self.guards.insert(key.to_string(), env::var(key).ok());
        }
        env::remove_var(key);
    }

    pub fn set_temp_home<P: AsRef<Path>>(&mut self, path: P) {
        self.set("HOME", path.as_ref().to_str().expect("Valid path"));
    }
}

impl Default for EnvGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (key, old_value) in self.guards.drain() {
            if let Some(value) = old_value {
                env::set_var(key, value);
            } else {
                env::remove_var(key);
            }
        }
    }
}

pub struct TestDirManager {
    pub root: PathBuf,
    _temp: Option<TempDir>,
}

impl TestDirManager {
    pub fn new() -> std::io::Result<Self> {
        let temp = TempDir::new()?;
        Ok(Self {
            root: temp.path().to_path_buf(),
            _temp: Some(temp),
        })
    }

    pub fn path(&self) -> &Path {
        &self.root
    }

    pub fn create_projects_structure(&self) -> std::io::Result<PathBuf> {
        let projects_dir = self
            .root
            .join(".gemini-cli-desktop")
            .join("projects");
        std::fs::create_dir_all(&projects_dir)?;
        Ok(projects_dir)
    }

    pub fn create_log_file(
        &self,
        project_hash: &str,
        timestamp: u64,
        content: &str,
    ) -> std::io::Result<PathBuf> {
        let projects_dir = self.create_projects_structure()?;
        let project_dir = projects_dir.join(project_hash);
        std::fs::create_dir_all(&project_dir)?;

        let file_name = format!("rpc-log-{timestamp}.log");
        let log_path = project_dir.join(file_name);
        std::fs::write(&log_path, content)?;
        Ok(log_path)
    }

    pub fn create_unique_subdir(&self, prefix: &str) -> std::io::Result<PathBuf> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let dir_path = self.root.join(format!("{prefix}-{unique}"));
        std::fs::create_dir_all(&dir_path)?;
        Ok(dir_path)
    }
}

impl Default for TestDirManager {
    fn default() -> Self {
        Self::new().expect("Failed to create temp dir")
    }
}
