use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

// Note: extra_types and handler are declared as submodules in config/mod.rs
// to avoid path resolution issues with nested modules

use crate::agents::types::AgentConfig;
#[cfg(feature = "bundle-pmat")]
use crate::agents::types::RateLimit;
pub use crate::config::schema::*;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub server: ServerConfig,
    pub main_agent: MainAgentConfig,
    pub agents: Vec<AgentConfig>,
    pub routing: RoutingConfig,
    pub rate_limiting: RateLimitingConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent_tasks: usize,
}

fn default_max_concurrent() -> usize {
    5
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MainAgentConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub agent_type: String,
    pub session_token_path: Option<String>,
}

// AgentConfig and RateLimit are defined in crate::agents::types

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RoutingConfig {
    #[serde(default)]
    pub tier: RoutingTier,
    pub rules: Vec<RoutingRule>,
}

/// Routing tier determines rule priority
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Default)]
#[serde(rename_all = "lowercase")]
pub enum RoutingTier {
    #[default]
    Default,
    User,
    Admin,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RoutingRule {
    pub task_type: String,
    pub keywords: Vec<String>,
    pub preferred_agents: Vec<String>,
    #[serde(default)]
    pub priority: u16, // 0-999
    #[serde(default)]
    pub enabled: bool,
}

impl Default for RoutingRule {
    fn default() -> Self {
        Self {
            task_type: String::new(),
            keywords: Vec::new(),
            preferred_agents: Vec::new(),
            priority: 0,
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RateLimitingConfig {
    #[serde(default = "default_strategy")]
    pub strategy: String,
    #[serde(default = "default_true")]
    pub track_usage: bool,
    pub usage_db_path: Option<String>,
}

fn default_strategy() -> String {
    "round-robin".to_string()
}
fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default = "default_output")]
    pub output: String,
}

fn default_log_level() -> String {
    "info".to_string()
}
fn default_output() -> String {
    "stdout".to_string()
}

impl Config {
    /// Load config from file path
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read config file: {:?}", path.as_ref()))?;

        #[allow(unused_mut)]
        let mut config: Config = toml::from_str(&content).context("Failed to parse TOML config")?;

        // Inject bundled PMAT agent if feature is enabled
        #[cfg(feature = "bundle-pmat")]
        {
            config.inject_bundled_pmat();
        }

        config.validate()?;
        Ok(config)
    }

    /// Load config from default locations
    pub fn load_default() -> Result<Self> {
        let config_paths = Self::default_config_paths();

        for path in config_paths {
            if path.exists() {
                tracing::info!("Loading config from: {:?}", path);
                return Self::load(&path);
            }
        }

        anyhow::bail!(
            "No config file found. Create ~/.config/bl1nk-agents-manager/config.toml or use --config"
        );
    }

    /// Default config file locations (in order of priority)
    fn default_config_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // Current directory
        paths.push(PathBuf::from("./config.toml"));
        paths.push(PathBuf::from("./.bl1nk-agents-manager.toml"));

        // User config directory
        if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
            let home_path = PathBuf::from(home);
            paths.push(home_path.join(".config/bl1nk-agents-manager/config.toml"));
            paths.push(home_path.join(".bl1nk-agents-manager.toml"));
        }

        paths
    }

    /// Inject bundled PMAT agent
    #[cfg(feature = "bundle-pmat")]
    fn inject_bundled_pmat(&mut self) {
        let pmat_agent = AgentConfig {
            id: "pmat-architect-internal".to_string(),
            name: "PMAT Architect (Bundled)".to_string(),
            agent_type: "internal".to_string(),
            command: None,
            args: None,
            extension_name: None,
            rate_limit: RateLimit {
                requests_per_minute: 120, // Higher limit for internal
                requests_per_day: 5000,
            },
            capabilities: vec![
                "code-analysis".to_string(),
                "context-generation".to_string(),
                "technical-debt".to_string(),
            ],
            priority: 200,
            enabled: true,
            description: Some("Bundled PMAT architect for internal analysis.".to_string()),
            model: None,
            temperature: None,
            max_tokens: None,
            prompt: None,
            color: None,
            permission: None,
            mode: None,
            thinking: None,
            reasoning_effort: None,
            text_verbosity: None,
            skills: None,
        };

        // Check if already exists
        if !self
            .agents
            .iter()
            .any(|a| a.id == "pmat-architect-internal")
        {
            tracing::info!("✨ Injecting bundled PMAT agent");
            self.agents.push(pmat_agent);
        }
    }

    /// Validate configuration
    fn validate(&self) -> Result<()> {
        // Validate server config
        if self.server.max_concurrent_tasks == 0 {
            anyhow::bail!("max_concurrent_tasks must be greater than 0");
        }

        // Validate agents
        if self.agents.is_empty() {
            anyhow::bail!("At least one agent must be configured");
        }

        // Check for duplicate agent IDs
        let mut seen_ids = std::collections::HashSet::new();
        for agent in &self.agents {
            if !seen_ids.insert(&agent.id) {
                anyhow::bail!("Duplicate agent ID: {}", agent.id);
            }
        }

        // Validate routing rules reference valid agents
        let agent_ids: Vec<String> = self.agents.iter().map(|a| a.id.clone()).collect();
        let agent_set: std::collections::HashSet<&str> =
            agent_ids.iter().map(|s| s.as_str()).collect();

        for rule in &self.routing.rules {
            for preferred_agent in &rule.preferred_agents {
                if !agent_set.contains(preferred_agent.as_str()) {
                    tracing::warn!(
                        "⚠️  Routing rule references unknown agent: {} (will be skipped)",
                        preferred_agent
                    );
                }
            }
        }

        // Validate priority ranges
        for rule in &self.routing.rules {
            if rule.priority > 999 {
                anyhow::bail!(
                    "Rule priority must be 0-999, got {} for task_type '{}'",
                    rule.priority,
                    rule.task_type
                );
            }
        }

        Ok(())
    }

    /// Get agent by ID
    pub fn get_agent(&self, id: &str) -> Option<&AgentConfig> {
        self.agents.iter().find(|a| a.id == id && a.enabled)
    }

    /// Get agents by capability
    pub fn get_agents_by_capability(&self, capability: &str) -> Vec<&AgentConfig> {
        self.agents
            .iter()
            .filter(|a| a.enabled && a.capabilities.contains(&capability.to_string()))
            .collect()
    }

    /// Get enabled agents only
    pub fn get_enabled_agents(&self) -> Vec<&AgentConfig> {
        self.agents.iter().filter(|a| a.enabled).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let config_str = r#"
            [server]
            host = "127.0.0.1"
            port = 3000

            [main_agent]
            name = "gemini"
            type = "gemini-cli"

            [[agents]]
            id = "test-agent"
            name = "Test"
            type = "cli"
            command = "test"
            capabilities = ["test"]

            [routing]
            rules = []

            [rate_limiting]
            usage_db_path = "/tmp/test.db"

            [logging]
        "#;

        let config: Config = toml::from_str(config_str).unwrap();
        assert_eq!(config.server.max_concurrent_tasks, 5);
        assert_eq!(config.routing.tier, RoutingTier::Default);
        assert_eq!(config.logging.level, "info");
    }

    #[test]
    fn test_tier_ordering() {
        assert!(RoutingTier::Admin > RoutingTier::User);
        assert!(RoutingTier::User > RoutingTier::Default);
    }
}
