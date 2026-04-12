use crate::config::{AgentConfig, RoutingConfig, RoutingRule, RoutingTier};
use crate::agents::register::{AgentState, AgentAvailability};
use crate::registry::RegistryService;
use anyhow::Result;
use std::cmp::Ordering;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CostCategory {
    Free,
    Cheap,
    Expensive,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PlanProposal {
    pub agent_id: String,
    pub agent_name: String,
    pub task_type: String,
    pub cost_category: CostCategory,
    pub reasoning: String,
    pub availability: String,
    pub capable_agents: Vec<String>,
}

pub struct AgentRouter {
    routing_config: RoutingConfig,
    registry_service: Option<Arc<RegistryService>>,
}

/// Routing rule with tier and priority for sorting
#[derive(Debug, Clone)]
struct ScoredRule<'a> {
    rule: &'a RoutingRule,
    tier: RoutingTier,
    priority: u16,
}

impl<'a> ScoredRule<'a> {
    fn new(rule: &'a RoutingRule, tier: RoutingTier) -> Self {
        Self {
            rule,
            tier,
            priority: rule.priority,
        }
    }
}

impl<'a> PartialEq for ScoredRule<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.tier == other.tier && self.priority == other.priority
    }
}

impl<'a> Eq for ScoredRule<'a> {}

impl<'a> PartialOrd for ScoredRule<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for ScoredRule<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        // First compare tier (Admin > User > Default)
        match self.tier.cmp(&other.tier) {
            Ordering::Equal => {
                // If same tier, compare priority (higher is better)
                self.priority.cmp(&other.priority)
            }
            other => other,
        }
    }
}

impl AgentRouter {
    pub fn new(routing_config: RoutingConfig) -> Self {
        Self { 
            routing_config,
            registry_service: None,
        }
    }

    pub fn with_registry(mut self, service: Arc<RegistryService>) -> Self {
        self.registry_service = Some(service);
        self
    }

    /// Create a resource-aware plan proposal
    pub fn create_proposal(
        &self,
        task_type: &str,
        prompt: &str,
        agent_states: &[&AgentState],
    ) -> Result<PlanProposal> {
        let agent_configs: Vec<&AgentConfig> = agent_states.iter().map(|s| &s.config).collect();
        let selected_agent = self.select_agent(task_type, prompt, &agent_configs)?;

        let state = agent_states.iter()
            .find(|s| s.config.id == selected_agent.id)
            .ok_or_else(|| anyhow::anyhow!("Selected agent state not found"))?;

        let ready_agent_configs: Vec<&AgentConfig> = agent_states
            .iter()
            .filter(|s| matches!(s.availability, AgentAvailability::Ready))
            .map(|s| &s.config)
            .collect();

        let capable_agents = self.filter_capable_agents(task_type, &ready_agent_configs)
            .into_iter()
            .map(|a| a.id.clone())
            .collect();

        let cost_category = if selected_agent.cost == 0 {
            CostCategory::Free
        } else if selected_agent.cost < 500 {
            CostCategory::Cheap
        } else {
            CostCategory::Expensive
        };

        let availability = match &state.availability {
            AgentAvailability::Ready => "Ready".to_string(),
            AgentAvailability::MissingTools(tools) => format!("Missing Tools: {}", tools.join(", ")),
        };

        let reasoning = self.determine_reasoning(task_type, prompt, selected_agent, &state.availability);

        Ok(PlanProposal {
            agent_id: selected_agent.id.clone(),
            agent_name: selected_agent.name.clone(),
            task_type: task_type.to_string(),
            cost_category,
            reasoning,
            availability,
            capable_agents,
        })
    }

    fn determine_reasoning(
        &self,
        task_type: &str,
        prompt: &str,
        agent: &AgentConfig,
        availability: &AgentAvailability,
    ) -> String {
        let mut reasons = Vec::new();

        // Check if matched via library search
        if let Some(service) = &self.registry_service {
            let matches = service.search(prompt, None);
            if matches.iter().any(|m| agent.capabilities.contains(&m.id)) {
                reasons.push("Matched via Knowledge Backbone Smart Search".to_string());
            }
        }

        // Check if matched via rule
        let matched_rule = self.routing_config.rules.iter()
            .filter(|r| r.enabled && self.rule_matches(r, task_type, prompt))
            .filter(|r| r.preferred_agents.contains(&agent.id))
            .max_by_key(|r| r.priority);

        if let Some(rule) = matched_rule {
            reasons.push(format!("Matched routing rule for '{}' (priority {})", task_type, rule.priority));
        } else if reasons.is_empty() {
            reasons.push(format!("Selected via priority fallback (priority {})", agent.priority));
        }

        if matches!(availability, AgentAvailability::Ready) {
            reasons.push("Agent is ready".to_string());
        }

        reasons.join(". ")
    }

    /// Select the best agent using tiered priority system + Smart Search
    pub fn select_agent<'a>(
        &self,
        task_type: &str,
        prompt: &str,
        available_agents: &'a [&'a AgentConfig],
    ) -> Result<&'a AgentConfig> {
        // 1. ลองใช้ Smart Search จาก Library ก่อน (Pattern Trigger)
        if let Some(service) = &self.registry_service {
            let results = service.search(prompt, None);
            for res in results {
                // ค้นหา agent ที่มีความสามารถ (capability) ตรงกับ ID ที่เจอใน Registry
                if let Some(agent) = available_agents.iter().find(|a| a.capabilities.contains(&res.id)) {
                    tracing::info!("🎯 Smart Search Trigger: Found agent '{}' for knowledge ID '{}'", agent.id, res.id);
                    return Ok(*agent);
                }
            }
        }

        // 2. ถ้าไม่เจอใน Registry ให้ใช้ Routing Rules เดิม
        let matching_rules: Vec<ScoredRule> = self.routing_config
            .rules
            .iter()
            .filter(|rule| rule.enabled && self.rule_matches(rule, task_type, prompt))
            .map(|rule| ScoredRule::new(rule, self.routing_config.tier.clone()))
            .collect();

        if !matching_rules.is_empty() {
            let mut sorted_rules = matching_rules;
            sorted_rules.sort_by(|a, b| b.cmp(a));

            for scored_rule in sorted_rules {
                for preferred_agent_id in &scored_rule.rule.preferred_agents {
                    if let Some(agent) = available_agents.iter().find(|a| &a.id == preferred_agent_id) {
                        return Ok(*agent);
                    }
                }
            }
        }

        // 3. Fallback สุดท้าย: Priority
        Self::fallback_by_priority(available_agents)
    }

    fn rule_matches(&self, rule: &RoutingRule, task_type: &str, prompt: &str) -> bool {
        if rule.task_type != task_type { return false; }
        if rule.keywords.is_empty() { return true; }
        let prompt_lower = prompt.to_lowercase();
        rule.keywords.iter().any(|keyword| prompt_lower.contains(&keyword.to_lowercase()))
    }

    fn fallback_by_priority<'a>(available_agents: &'a [&'a AgentConfig]) -> Result<&'a AgentConfig> {
        available_agents.iter().max_by_key(|a| a.priority).copied().ok_or_else(|| anyhow::anyhow!("No available agents"))
    }

    pub fn filter_capable_agents<'a>(&self, _task_type: &str, all_agents: &'a [&'a AgentConfig]) -> Vec<&'a AgentConfig> {
        all_agents.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RateLimit;

    fn create_test_agent(id: &str, capabilities: Vec<&str>, priority: u8) -> AgentConfig {
        AgentConfig {
            id: id.to_string(),
            name: id.to_string(),
            description: "Test description".to_string(),
            mode: "all".to_string(),
            agent_type: "cli".to_string(),
            command: "test".to_string(),
            args: None,
            extension_name: None,
            rate_limit: RateLimit::default(),
            capabilities: capabilities.iter().map(|s| s.to_string()).collect(),
            priority,
            enabled: true,
            requires: Vec::new(),
            cost: 0,
            tool: crate::config::AgentToolPermissions { bash: false, write: false, skill: true, ask: false },
            permission: 500,
            permission_policy: serde_json::json!({}),
        }
    }

    #[test]
    fn test_router_tie_breaking_logic() {
        let routing_config = RoutingConfig::default();
        let router = AgentRouter::new(routing_config);
        let agents = vec![
            create_test_agent("agent-a", vec!["test"], 10),
            create_test_agent("agent-b", vec!["test"], 10),
        ];
        let agent_refs: Vec<&AgentConfig> = agents.iter().collect();
        let selected = router.select_agent("test", "run test", &agent_refs).unwrap();
        assert!(selected.id == "agent-a" || selected.id == "agent-b");
    }
}
