use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;

pub enum StorageLocation {
    Global,
    Local,
}

pub struct Persistence {
    base_path: PathBuf,
}

impl Persistence {
    pub fn new(location: StorageLocation) -> Result<Self> {
        let base_path = match location {
            StorageLocation::Global => {
                let home = std::env::var("HOME")
                    .or_else(|_| std::env::var("USERPROFILE"))
                    .context("Could not find home directory")?;
                PathBuf::from(home).join(".bl1nk")
            }
            StorageLocation::Local => PathBuf::from("."),
        };
        Ok(Self { base_path })
    }

    fn validate_relative_path(&self, relative_path: &str) -> Result<PathBuf> {
        let path = Path::new(relative_path);
        if path.is_absolute()
            || path
                .components()
                .any(|c| matches!(c, std::path::Component::ParentDir | std::path::Component::Prefix(_)))
        {
            return Err(anyhow::anyhow!(
                "relative_path must not be an absolute path or contain path traversal: {}",
                relative_path
            ));
        }
        Ok(self.base_path.join(relative_path))
    }

    pub async fn save_json<T: Serialize>(&self, relative_path: &str, data: &T) -> Result<()> {
        let path = self.validate_relative_path(relative_path)?;
        let content = serde_json::to_string_pretty(data).context("Failed to serialize to JSON")?;
        self.atomic_write(&path, content).await
    }

    pub async fn load_json<T: DeserializeOwned>(&self, relative_path: &str) -> Result<T> {
        let path = self.validate_relative_path(relative_path)?;
        let content = fs::read_to_string(&path)
            .await
            .with_context(|| format!("Failed to read file: {:?}", path))?;
        serde_json::from_str(&content).with_context(|| format!("Failed to deserialize JSON from {:?}", path))
    }

    pub async fn save_toml<T: Serialize>(&self, relative_path: &str, data: &T) -> Result<()> {
        let path = self.validate_relative_path(relative_path)?;
        let content = toml::to_string_pretty(data).context("Failed to serialize to TOML")?;
        self.atomic_write(&path, content).await
    }

    pub async fn load_toml<T: DeserializeOwned>(&self, relative_path: &str) -> Result<T> {
        let path = self.validate_relative_path(relative_path)?;
        let content = fs::read_to_string(&path)
            .await
            .with_context(|| format!("Failed to read file: {:?}", path))?;
        toml::from_str(&content).with_context(|| format!("Failed to deserialize TOML from {:?}", path))
    }

    async fn atomic_write(&self, path: &Path, content: String) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .with_context(|| format!("Failed to create directory: {:?}", parent))?;
        }

        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let mut temp_name = path.as_os_str().to_os_string();
        temp_name.push(format!(".{}.{}.tmp", std::process::id(), nonce));
        let temp_path = PathBuf::from(temp_name);

        fs::write(&temp_path, content)
            .await
            .with_context(|| format!("Failed to write to temporary file: {:?}", temp_path))?;

        let result = async {
            // On Windows, rename fails if the destination already exists, so remove it first.
            #[cfg(windows)]
            {
                match fs::remove_file(path).await {
                    Ok(()) => {}
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                    Err(e) => {
                        return Err(e).with_context(|| format!("Failed to remove existing file: {:?}", path));
                    }
                }
            }

            fs::rename(&temp_path, path)
                .await
                .with_context(|| format!("Failed to rename {:?} to {:?}", temp_path, path))?;
            Ok(())
        }
        .await;

        if result.is_err() {
            let _ = fs::remove_file(&temp_path).await;
        }

        result
    }
}
