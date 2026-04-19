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

        // --- 🧠 DYNAMIC WEIGHTING LOGIC ---
        // ดึงคะแนนความเชื่อใจ (Trust Score) มาคำนวณร่วมกับ Base Priority
        let weight_registry = crate::registry::WeightRegistry::load().await.unwrap_or_default();

        let mut scored_agents: Vec<(String, f64)> = capable_agents
            .into_iter()
            .filter_map(|id| {
                registry.get_agent_state(&id).map(|state| {
                    let trust_score = weight_registry.get_trust_score(&id);
                    let effective_priority = state.config.priority as f64 * trust_score;
                    (id, effective_priority)
                })
            })
            .collect();

        // เรียงลำดับตาม Effective Priority (สูงไปต่ำ)
        scored_agents.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let (selected_id, _score) = scored_agents.first().context("No agents available after scoring")?;
        let state = registry.get_agent_state(selected_id).context("Agent not found")?;

        Ok(PlanProposal {
            task_id: uuid::Uuid::new_v4().to_string(),
            agent_id: selected_id.clone(),
            agent_name: state.config.name.clone(),
            reasoning: format!(
                "Selected agent '{}' based on task type '{}' and a dynamic trust score (Effective Priority: {:.2}).",
                state.config.name, task_type, _score
            ),
            cost_category: CostCategory::Standard,
            availability: "Ready".to_string(),
            capable_agents: scored_agents.into_iter().map(|(id, _)| id).collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AgentConfig, PolicyRule, RateLimit, RoutingConfig, RoutingTier};

    #[tokio::test]
    async fn test_router_basic() {
        let agent = AgentConfig {
            id: "test".into(),
            name: "Test Agent".into(),
            description: "desc".into(),
            mode: "subagent".into(),
            agent_type: "general".into(),
            capabilities: vec!["test".into()],
            tier: 2,
            priority: 100,
            policies: vec![PolicyRule {
                tool: "test".to_string(),
                decision: "allow".to_string(),
                modes: vec![],
            }],
            enabled: true,
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
