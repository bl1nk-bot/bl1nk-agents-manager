use crate::config::{AgentConfig, RateLimit};
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

pub struct DiscoveryEngine;

impl DiscoveryEngine {
    pub async fn scan() -> Result<crate::system::discovery::DiscoveryReport> {
        Ok(DiscoveryReport { agents: Vec::new() })
    }

    pub async fn save(_report: &DiscoveryReport) -> Result<()> {
        Ok(())
    }

    pub fn parse_agent_file(path: &Path) -> Result<AgentConfig> {
        let slug = path.file_stem().unwrap().to_str().unwrap();

        let mut default_policies = HashMap::new();
        default_policies.insert("activate_skill".to_string(), "allow".to_string());
        default_policies.insert("ask_user".to_string(), "allow".to_string());
        default_policies.insert("run_shell_command".to_string(), "deny".to_string());

        Ok(AgentConfig {
            id: slug.to_string(),
            name: slug.to_string(),
            version: "1.0.0".to_string(),
            description: "Discovered agent (v1.7.5.1 Standard)".to_string(),
            mode: "subagent".to_string(),
            agent_type: "general".to_string(),
            capabilities: Vec::new(),
            tier: 2,
            priority: 100,
            policies: default_policies,
            enabled: true,
            command: "true".to_string(),
            args: None,
            extension_name: None,
            requires: Vec::new(),
            cost: 0,
            rate_limit: RateLimit::default(),
        })
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DiscoveryReport {
    pub agents: Vec<String>,
}
