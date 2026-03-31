use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use tokio::process::Command;
use std::path::PathBuf;
use anyhow::{Result, Context};
use crate::persistence::{Persistence, StorageLocation};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub version: Option<String>,
    pub available: bool,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryReport {
    pub timestamp: DateTime<Utc>,
    pub ai_clis: Vec<ToolInfo>,
    pub vcs: Vec<ToolInfo>,
    pub package_managers: Vec<ToolInfo>,
}

pub struct DiscoveryEngine;

impl DiscoveryReport {
    pub fn is_tool_available(&self, name: &str) -> bool {
        self.ai_clis.iter().any(|t| t.name == name && t.available)
            || self.vcs.iter().any(|t| t.name == name && t.available)
            || self.package_managers.iter().any(|t| t.name == name && t.available)
    }

    pub async fn load() -> Result<Self> {
        let persistence = Persistence::new(StorageLocation::Global)?;
        persistence.load_json("discovery.json").await
    }
}

impl DiscoveryEngine {
    pub async fn scan() -> Result<DiscoveryReport> {
        let ai_clis = vec!["gemini", "claude", "qwen", "ollama"];
        let vcs = vec!["git"];
        let package_managers = vec!["npm", "pnpm", "yarn", "cargo"];

        let mut report = DiscoveryReport {
            timestamp: Utc::now(),
            ai_clis: Vec::new(),
            vcs: Vec::new(),
            package_managers: Vec::new(),
        };

        for cli in ai_clis {
            report.ai_clis.push(Self::check_tool(cli).await);
        }

        for v in vcs {
            report.vcs.push(Self::check_tool(v).await);
        }

        for pm in package_managers {
            report.package_managers.push(Self::check_tool(pm).await);
        }

        Ok(report)
    }

    pub async fn save(report: &DiscoveryReport) -> Result<()> {
        let persistence = Persistence::new(StorageLocation::Global)?;
        persistence.save_json("discovery.json", report).await?;

        tracing::info!("✅ Discovery report saved to global storage");
        Ok(())
    }

    async fn check_tool(name: &str) -> ToolInfo {
        let path = Self::find_binary(name).await;
        let available = path.is_some();
        let version = if let Some(ref p) = path {
            Self::get_version(p).await
        } else {
            None
        };

        ToolInfo {
            name: name.to_string(),
            version,
            available,
            path: path.map(|p| p.to_string_lossy().to_string()),
        }
    }

    async fn find_binary(name: &str) -> Option<PathBuf> {
        let cmd = if cfg!(windows) { "where" } else { "which" };
        let output = Command::new(cmd).arg(name).output().await.ok()?;

        if output.status.success() {
            let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path_str.is_empty() {
                // 'where' on Windows might return multiple paths, take the first one
                let first_path = path_str.lines().next().unwrap_or(&path_str);
                return Some(PathBuf::from(first_path));
            }
        }
        None
    }

    async fn get_version(path: &PathBuf) -> Option<String> {
        // Most tools support --version
        let output = Command::new(path).arg("--version").output().await.ok()?;

        if output.status.success() {
            let version_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !version_str.is_empty() {
                return Some(version_str);
            }
        }

        // Some tools might use -v
        let output = Command::new(path).arg("-v").output().await.ok()?;
        if output.status.success() {
            let version_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !version_str.is_empty() {
                return Some(version_str);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_serialization() {
        let report = DiscoveryReport {
            timestamp: Utc::now(),
            ai_clis: vec![ToolInfo {
                name: "gemini".to_string(),
                version: Some("1.0.0".to_string()),
                available: true,
                path: Some("/usr/bin/gemini".to_string()),
            }],
            vcs: vec![ToolInfo {
                name: "git".to_string(),
                version: Some("2.34.1".to_string()),
                available: true,
                path: Some("/usr/bin/git".to_string()),
            }],
            package_managers: vec![ToolInfo {
                name: "cargo".to_string(),
                version: Some("1.56.0".to_string()),
                available: true,
                path: Some("/usr/bin/cargo".to_string()),
            }],
        };

        let json = serde_json::to_string(&report).expect("Failed to serialize");
        let deserialized: DiscoveryReport = serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(deserialized.ai_clis.len(), 1);
        assert_eq!(deserialized.ai_clis[0].name, "gemini");
        assert_eq!(deserialized.vcs[0].name, "git");
        assert_eq!(deserialized.package_managers[0].name, "cargo");
    }
}
