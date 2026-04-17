use crate::agents::register::AgentRegistry;
use crate::registry::RegistryService;
use anyhow::{Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PlanProposal {
    pub task_id: String,
    pub agent_id: String,
    pub agent_name: String,
    pub reasoning: String,
    pub cost_category: CostCategory,
    pub availability: String,
    pub capable_agents: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub enum CostCategory {
    Free,
    Cheap,
    Standard,
    Premium,
}

pub struct AgentRouter {
    config: crate::config::RoutingConfig,
    registry_service: Option<Arc<RegistryService>>,
}

impl AgentRouter {
    pub fn new(config: crate::config::RoutingConfig) -> Self {
        Self {
            config,
            registry_service: None,
        }
    }

    pub fn with_registry(mut self, service: Arc<RegistryService>) -> Self {
        self.registry_service = Some(service);
        self
    }

    pub async fn route_task(&self, registry: &AgentRegistry, task_type: &str, prompt: &str) -> Result<PlanProposal> {
        let mut capable_agents = Vec::new();

        if let Some(service) = &self.registry_service {
            let matches = service.search_agents(prompt, false);
            for m in matches {
                capable_agents.push(m.id);
            }
        }

        let rule_based: Vec<String> = self
            .config
            .rules
            .iter()
            .filter(|r| r.enabled && (r.task_type == task_type || r.task_type == "*"))
            .flat_map(|r| r.preferred_agents.clone())
            .collect();

        capable_agents.extend(rule_based);

        if capable_agents.is_empty() {
            capable_agents = registry.list_agent_ids();
        }

        let selected_id = capable_agents.first().context("No agents available")?;
        let state = registry.get_agent_state(selected_id).context("Agent not found")?;

        Ok(PlanProposal {
            task_id: uuid::Uuid::new_v4().to_string(),
            agent_id: selected_id.clone(),
            agent_name: state.config.name.clone(),
            reasoning: format!(
                "Selected based on task type '{}' and smart capability matching.",
                task_type
            ),
            cost_category: CostCategory::Standard,
            availability: "Ready".to_string(),
            capable_agents,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AgentConfig, AgentToolPermissions, RateLimit, RoutingConfig, RoutingTier};

    #[tokio::test]
    async fn test_router_basic() {
        let agent = AgentConfig {
            id: "test".into(),
            name: "Test Agent".into(),
            description: "desc".into(),
            mode: "subagent".into(),
            agent_type: "general".into(),
            model: "sonnet".into(),
            capabilities: vec!["test".into()],
            priority: 100,
            enabled: true,
            tool: AgentToolPermissions {
                bash: false,
                write: false,
                skill: true,
                ask: true,
            },
            permission: 100,
            permission_policy: serde_json::json!({}),
            command: "true".into(),
            args: None,
            extension_name: None,
            requires: vec![],
            cost: 0,
            rate_limit: RateLimit::default(),
        };

        let registry = AgentRegistry::new(vec![agent], None);
        let router = AgentRouter::new(RoutingConfig {
            rules: vec![],
            tier: RoutingTier::Default,
        });

        let plan = router.route_task(&registry, "general", "test").await.unwrap();
        assert_eq!(plan.agent_id, "test");
    }
}
