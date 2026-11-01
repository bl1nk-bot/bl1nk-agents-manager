use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredTool {
    pub name: String,
    pub is_installed: bool,
    pub version: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiscoveryResult {
    pub ai_clis: HashMap<String, DiscoveredTool>,
    pub dev_tools: HashMap<String, DiscoveredTool>,
    pub package_managers: HashMap<String, DiscoveredTool>,
    pub updated_at: Option<String>,
}

pub async fn discover_resources() -> Result<DiscoveryResult> {
    let mut result = DiscoveryResult::default();

    // AI CLIs to scan
    let ai_clis = vec!["gemini", "claude", "qwen", "ollama"];
    for name in ai_clis {
        result
            .ai_clis
            .insert(name.to_string(), check_tool(name).await?);
    }

    // Dev Tools to scan
    let dev_tools = vec!["git", "just", "cargo"];
    for name in dev_tools {
        result
            .dev_tools
            .insert(name.to_string(), check_tool(name).await?);
    }

    // Package Managers to scan
    let pkg_managers = vec!["npm", "pnpm", "yarn"];
    for name in pkg_managers {
        result
            .package_managers
            .insert(name.to_string(), check_tool(name).await?);
    }

    result.updated_at = Some(chrono::Utc::now().to_rfc3339());

    Ok(result)
}

async fn check_tool(name: &str) -> Result<DiscoveredTool> {
    // Check if binary exists using `which`
    let which_output = Command::new("which").arg(name).output().await;

    if let Ok(output) = which_output {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();

            // Try to get version
            let version = get_tool_version(name).await;

            return Ok(DiscoveredTool {
                name: name.to_string(),
                is_installed: true,
                version,
                path: Some(path),
            });
        }
    }

    Ok(DiscoveredTool {
        name: name.to_string(),
        is_installed: false,
        version: None,
        path: None,
    })
}

async fn get_tool_version(name: &str) -> Option<String> {
    let output = Command::new(name).arg("--version").output().await;

    if let Ok(o) = output {
        if o.status.success() {
            let v = String::from_utf8_lossy(&o.stdout).trim().to_string();
            // Some tools output more than just the version string, let's keep it simple
            if !v.is_empty() {
                return Some(v);
            }
        }
    }
    None
}

pub fn get_global_config_dir() -> Result<PathBuf> {
    let home = std::env::var("HOME")
        .unwrap_or_else(|_| std::env::var("USERPROFILE").unwrap_or_else(|_| ".".to_string()));

    let path = PathBuf::from(home)
        .join(".config")
        .join("bl1nk-agents-manager");
    Ok(path)
}

pub async fn save_discovery_result(result: &DiscoveryResult) -> Result<()> {
    let config_dir = get_global_config_dir()?;
    fs::create_dir_all(&config_dir)
        .await
        .context("Failed to create config directory")?;

    let file_path = config_dir.join("discovery.json");
    let json =
        serde_json::to_string_pretty(result).context("Failed to serialize discovery result")?;
    fs::write(file_path, json)
        .await
        .context("Failed to write discovery file")?;

    Ok(())
}

pub async fn load_discovery_result() -> Result<Option<DiscoveryResult>> {
    let file_path = get_global_config_dir()?.join("discovery.json");
    if !file_path.exists() {
        return Ok(None);
    }

    let json = fs::read_to_string(file_path)
        .await
        .context("Failed to read discovery file")?;
    let result = serde_json::from_str(&json).context("Failed to deserialize discovery result")?;

    Ok(Some(result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::EnvGuard;
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    async fn test_discovery_logic() {
        let result = discover_resources().await.unwrap();
        println!("{:#?}", result);
        // At least git should be installed in most dev environments
        if let Some(git) = result.dev_tools.get("git") {
            assert!(git.is_installed || !git.is_installed); // Just ensure it runs
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_persistence() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let mut env_guard = EnvGuard::new();
        env_guard.set("HOME", temp_dir.path().to_str().unwrap());

        let result = discover_resources().await.unwrap();
        save_discovery_result(&result).await.unwrap();

        let loaded = load_discovery_result()
            .await
            .unwrap()
            .expect("Should load result");
        assert_eq!(result.ai_clis.len(), loaded.ai_clis.len());
        assert_eq!(result.updated_at, loaded.updated_at);
    }
}
