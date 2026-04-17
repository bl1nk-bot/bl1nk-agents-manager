use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use crate::system::skill_discovery;

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
}

/// ข้อมูลส่วนหัวจากไฟล์ .md (Frontmatter)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentMdHeader {
    pub name: String,
    pub description: String,
    pub mode: String,
    pub tool: Vec<String>,
}

/// โครงสร้างเอเจนต์สมบูรณ์ (รักษาฟิลด์เดิมไว้เพื่อความปลอดภัย)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub mode: String,
    #[serde(rename = "type")]
    pub agent_type: String,
    pub model: String,
    pub capabilities: Vec<String>,
    pub priority: u8,
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub tool: AgentToolPermissions,
    pub permission: u32,
    pub permission_policy: serde_json::Value,
    
    // ฟิลด์ทางเทคนิคที่จำเป็น (รักษาไว้เพื่อให้โค้ดส่วนอื่นไม่พัง)
    #[serde(default = "default_command")]
    pub command: String,
    #[serde(default)]
    pub args: Option<Vec<String>>,
    #[serde(default)]
    pub extension_name: Option<String>,
    #[serde(default)]
    pub requires: Vec<String>,
    #[serde(default)]
    pub cost: u16,
    #[serde(default)]
    pub rate_limit: RateLimit,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentToolPermissions {
    pub bash: bool,
    pub write: bool,
    pub skill: bool,
    pub ask: bool,
}

fn default_true() -> bool { true }
fn default_command() -> String { "true".to_string() }

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RateLimit {
    pub requests_per_minute: u32,
    pub requests_per_day: u32,
}

impl Default for RateLimit {
    fn default() -> Self { Self { requests_per_minute: 60, requests_per_day: 2000 } }
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(&path)?;
        let mut config: Config = toml::from_str(&content)?;
        
        let runtime = tokio::runtime::Runtime::new()?;
        runtime.block_on(async {
            config.auto_discover_agents().await
        })?;
        
        config.validate()?;
        Ok(config)
    }

    async fn auto_discover_agents(&mut self) -> Result<()> {
        let agent_dirs = vec![PathBuf::from("agents"), PathBuf::from("skills")];
        let discovered = skill_discovery::discover_validated_assets(agent_dirs).await?;
        
        let registry_path = "agents/agents.json";
        let registry: crate::registry::schema::Registry = if Path::new(registry_path).exists() {
            let content = fs::read_to_string(registry_path)?;
            serde_json::from_str(&content).unwrap_or_else(|_| crate::registry::schema::Registry { 
                version: "1.7.0".into(), last_updated: None, agents: vec![] 
            })
        } else {
            crate::registry::schema::Registry { version: "1.7.0".into(), last_updated: None, agents: vec![] }
        };

        for meta in discovered {
            let entry = registry.agents.iter().find(|a| a.name == meta.name);
            
            let agent = if let Some(e) = entry {
                AgentConfig {
                    id: meta.name.clone(),
                    name: meta.name,
                    description: meta.description,
                    mode: meta.options.mode.clone(),
                    agent_type: e.agent_type.clone(),
                    model: e.model.clone(),
                    capabilities: e.capabilities.clone(),
                    priority: if e.permission > 500 { 90 } else { 50 },
                    enabled: true,
                    tool: AgentToolPermissions {
                        bash: e.tool_permissions.bash,
                        write: e.tool_permissions.write,
                        skill: e.tool_permissions.skill,
                        ask: e.tool_permissions.ask,
                    },
                    permission: e.permission,
                    permission_policy: e.permission_policy.clone(),
                    command: "true".into(),
                    args: None,
                    extension_name: None,
                    requires: vec![],
                    cost: 0,
                    rate_limit: RateLimit::default(),
                }
            } else {
                AgentConfig {
                    id: meta.name.clone(),
                    name: meta.name,
                    description: meta.description,
                    mode: meta.options.mode.clone(),
                    agent_type: "general".into(),
                    model: "sonnet".into(),
                    capabilities: vec![meta.filename],
                    priority: 50,
                    enabled: true,
                    tool: AgentToolPermissions { bash: false, write: false, skill: true, ask: true },
                    permission: 100,
                    permission_policy: serde_json::json!({"hierarchy": ["default"]}),
                    command: "true".into(),
                    args: None,
                    extension_name: None,
                    requires: vec![],
                    cost: 0,
                    rate_limit: RateLimit::default(),
                }
            };

            if let Some(pos) = self.agents.iter().position(|a| a.id == agent.id) {
                self.agents[pos] = agent;
            } else {
                self.agents.push(agent);
            }
        }
        Ok(())
    }

    pub fn load_default() -> Result<Self> {
        Self::load("config.toml")
    }

    fn validate(&self) -> Result<()> {
        if self.agents.is_empty() { bail!("At least one agent must be loaded"); }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RoutingConfig {
    #[serde(default)]
    pub tier: RoutingTier,
    pub rules: Vec<RoutingRule>,
}

impl Default for RoutingConfig {
    fn default() -> Self { Self { tier: RoutingTier::Default, rules: vec![] } }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Default)]
#[serde(rename_all = "lowercase")]
pub enum RoutingTier { #[default] Default, User, Admin }

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RoutingRule {
    pub task_type: String,
    pub preferred_agents: Vec<String>,
    pub priority: u16,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RateLimitingConfig {
    pub usage_db_path: String,
    #[serde(default = "default_true")]
    pub track_usage: bool,
    #[serde(default = "default_strategy")]
    pub strategy: String,
}

fn default_strategy() -> String { "round-robin".to_string() }

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct LoggingConfig {
    pub level: String,
    pub output: String,
}
