use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub server: ServerConfig,
    pub main_agent: MainAgentConfig,
    pub agents: Vec<AgentConfig>,
    pub routing: RoutingConfig,
    pub rate_limiting: RateLimitingConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent_tasks: usize,
}

fn default_max_concurrent() -> usize { 5 }

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MainAgentConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub agent_type: String,
    #[serde(default)]
    pub session_token_path: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub mode: String,
    #[serde(rename = "type")]
    pub agent_type: String,
    #[serde(default = "default_command")]
    pub command: String,
    #[serde(default)]
    pub args: Option<Vec<String>>,
    #[serde(default)]
    pub extension_name: Option<String>,
    #[serde(default)]
    pub rate_limit: RateLimit,
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub priority: u8,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub requires: Vec<String>,
    #[serde(default)]
    pub cost: u16,
    pub tool: AgentToolPermissions,
    pub permission: u32,
    pub permission_policy: serde_json::Value,
}

fn default_command() -> String { "true".to_string() }

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentToolPermissions {
    pub bash: bool,
    pub write: bool,
    pub skill: bool,
    pub ask: bool,
}

fn default_true() -> bool { true }

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RateLimit {
    #[serde(default = "default_rpm")]
    pub requests_per_minute: u32,
    #[serde(default = "default_rpd")]
    pub requests_per_day: u32,
}

fn default_rpm() -> u32 { 60 }
fn default_rpd() -> u32 { 2000 }

impl Default for RateLimit {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            requests_per_day: 2000,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RoutingConfig {
    #[serde(default)]
    pub tier: RoutingTier,
    #[serde(default)]
    pub rules: Vec<RoutingRule>,
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self {
            tier: RoutingTier::Default,
            rules: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum RoutingTier {
    #[default]
    Default,
    User,
    Admin,
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RoutingRule {
    pub task_type: String,
    #[serde(default)]
    pub keywords: Vec<String>,
    pub preferred_agents: Vec<String>,
    #[serde(default)]
    pub priority: u16,
    #[serde(default = "default_true")]
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
    pub usage_db_path: String,
}

fn default_strategy() -> String { "round-robin".to_string() }

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default = "default_output")]
    pub output: String,
}

fn default_log_level() -> String { "info".to_string() }
fn default_output() -> String { "stdout".to_string() }

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            output: "stdout".to_string(),
        }
    }
}

impl Config {
    /// Load config from file path (auto-detect format from extension)
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;
        
        let config = Self::parse_content(&content, path)
            .with_context(|| format!("Failed to parse config from {:?}", path))?;
        
        config.validate()?;
        Ok(config)
    }

    /// Parse config content (auto-detect format)
    fn parse_content(content: &str, path: &Path) -> Result<Self> {
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match extension.as_str() {
            "toml" => Self::parse_toml(content),
            "json" => Self::parse_json(content),
            "yaml" | "yml" => Self::parse_yaml(content),
            _ => {
                // Try to auto-detect from content
                Self::parse_auto(content)
            }
        }
    }

    /// Parse TOML format
    fn parse_toml(content: &str) -> Result<Self> {
        toml::from_str(content)
            .context("Failed to parse TOML config")
    }

    /// Parse JSON format
    fn parse_json(content: &str) -> Result<Self> {
        serde_json::from_str(content)
            .context("Failed to parse JSON config")
    }

    /// Parse YAML format
    fn parse_yaml(content: &str) -> Result<Self> {
        serde_yaml::from_str(content)
            .context("Failed to parse YAML config")
    }

    /// Auto-detect and parse format
    fn parse_auto(content: &str) -> Result<Self> {
        let trimmed = content.trim();
        
        // Check for TOML markers
        if trimmed.starts_with('[') || trimmed.contains("server =") {
            return Self::parse_toml(content);
        }
        
        // Check for JSON markers
        if trimmed.starts_with('{') {
            return Self::parse_json(content);
        }
        
        // Check for YAML markers
        if trimmed.starts_with("server:") || trimmed.starts_with("---") {
            return Self::parse_yaml(content);
        }
        
        // Default to TOML
        Self::parse_toml(content)
    }

    /// Load config from default locations (supports all formats)
    pub fn load_default() -> Result<Self> {
        let config_paths = Self::default_config_paths();
        
        for path in config_paths {
            if path.exists() {
                tracing::info!("Loading config from: {:?}", path);
                return Self::load(&path);
            }
        }
        
        anyhow::bail!(
            "No config file found. Create config.toml/json/yaml in ~/.config/bl1nk-agents-manager/ or use --config"
        );
    }

    /// Default config file locations (in order of priority)
    fn default_config_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();
        let formats = ["toml", "json", "yaml", "yml"];
        let filenames = ["config", ".bl1nk-agents-manager"];
        
        // Current directory
        for fmt in &formats {
            for name in &filenames {
                paths.push(PathBuf::from(format!("{}.{}", name, fmt)));
            }
        }
        
        // User config directory
        if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
            let home_path = PathBuf::from(home);
            for fmt in &formats {
                for name in &filenames {
                    paths.push(home_path.join(format!(".config/bl1nk-agents-manager/config.{}", fmt)));
                    paths.push(home_path.join(format!(".{}.{}", name, fmt)));
                }
            }
        }
        
        paths
    }

    /// Validate configuration
    fn validate(&self) -> Result<()> {
        if self.server.max_concurrent_tasks == 0 {
            anyhow::bail!("max_concurrent_tasks must be greater than 0");
        }

        if self.agents.is_empty() {
            anyhow::bail!("At least one agent must be configured");
        }

        let mut seen_ids = std::collections::HashSet::new();
        for agent in &self.agents {
            if !seen_ids.insert(&agent.id) {
                anyhow::bail!("Duplicate agent ID: {}", agent.id);
            }
        }

        let agent_ids: Vec<String> = self.agents.iter().map(|a| a.id.clone()).collect();
        for rule in &self.routing.rules {
            for preferred_agent in &rule.preferred_agents {
                if !agent_ids.contains(preferred_agent) {
                    tracing::warn!(
                        "Routing rule references unknown agent: {} (will be skipped)",
                        preferred_agent
                    );
                }
            }
        }

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_toml() {
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
        "#;

        let config: Config = toml::from_str(config_str).unwrap();
        assert_eq!(config.server.max_concurrent_tasks, 5);
        assert_eq!(config.routing.tier, RoutingTier::Default);
        assert_eq!(config.logging.level, "info");
    }

    #[test]
    fn test_parse_json() {
        let config_str = r#"{
            "server": {
                "host": "127.0.0.1",
                "port": 3000
            },
            "main_agent": {
                "name": "gemini",
                "type": "gemini-cli"
            },
            "agents": [{
                "id": "test-agent",
                "name": "Test",
                "type": "cli",
                "command": "test",
                "capabilities": ["test"]
            }],
            "routing": {
                "rules": []
            },
            "rate_limiting": {
                "usage_db_path": "/tmp/test.db"
            }
        }"#;

        let config: Config = serde_json::from_str(config_str).unwrap();
        assert_eq!(config.server.port, 3000);
    }

    #[test]
    fn test_tier_ordering() {
        assert!(RoutingTier::Admin > RoutingTier::User);
        assert!(RoutingTier::User > RoutingTier::Default);
    }

    #[test]
    fn test_load_non_existent_file() {
        let result = Config::load("non_existent_file.toml");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_malformed_toml() {
        let content = "invalid = [toml";
        let result = Config::parse_toml(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_empty_agents() {
        let config = Config {
            server: ServerConfig { host: "127.0.0.1".into(), port: 3000, max_concurrent_tasks: 5 },
            main_agent: MainAgentConfig { name: "test".into(), agent_type: "cli".into(), session_token_path: None },
            agents: vec![],
            routing: RoutingConfig { tier: RoutingTier::Default, rules: vec![] },
            rate_limiting: RateLimitingConfig { strategy: "rr".into(), track_usage: true, usage_db_path: "db".into() },
            logging: LoggingConfig::default(),
        };
        assert!(config.validate().is_err());
    }
}
