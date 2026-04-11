use crate::config::{AgentConfig, RoutingConfig, RoutingRule, RoutingTier};
use crate::agents::register::{AgentState, AgentAvailability};
use anyhow::Result;
use std::cmp::Ordering;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

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
        Self { routing_config }
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

        // Check if matched via rule
        let matched_rule = self.routing_config.rules.iter()
            .filter(|r| r.enabled && self.rule_matches(r, task_type, prompt))
            .filter(|r| r.preferred_agents.contains(&agent.id))
            .max_by_key(|r| r.priority);

        if let Some(rule) = matched_rule {
            reasons.push(format!("Matched routing rule for '{}' (priority {})", task_type, rule.priority));
        } else {
            reasons.push(format!("Selected via priority fallback (priority {})", agent.priority));
        }

        if matches!(availability, AgentAvailability::Ready) {
            reasons.push("Agent is ready with all required tools".to_string());
        } else {
            reasons.push("NOTE: Agent is missing some tools but was still the best choice".to_string());
        }

        if agent.cost == 0 {
            reasons.push("Chosen for zero cost".to_string());
        }

        reasons.join(". ")
    }

    /// Select the best agent using tiered priority system
    pub fn select_agent<'a>(
        &self,
        task_type: &str,
        prompt: &str,
        available_agents: &'a [&'a AgentConfig],
    ) -> Result<&'a AgentConfig> {
        tracing::debug!("🔍 Router: Selecting agent for task_type='{}'", task_type);
        tracing::debug!("📝 Prompt: {}", prompt.chars().take(100).collect::<String>());

        // Find all matching rules
        let matching_rules: Vec<ScoredRule> = self.routing_config
            .rules
            .iter()
            .filter(|rule| rule.enabled && self.rule_matches(rule, task_type, prompt))
            .map(|rule| ScoredRule::new(rule, self.routing_config.tier.clone()))
            .collect();

        tracing::debug!("✅ Found {} matching rules", matching_rules.len());

        if matching_rules.is_empty() {
            tracing::debug!("⚠️  No matching rules, falling back to agent priority");
            return Self::fallback_by_priority(available_agents);
        }

        // Sort by tier (Admin > User > Default) then priority (high > low)
        let mut sorted_rules = matching_rules;
        sorted_rules.sort_by(|a, b| b.cmp(a)); // Reverse for highest first

        // Try each rule's preferred agents in order
        for scored_rule in sorted_rules {
            tracing::debug!(
                "🎯 Trying rule: tier={:?}, priority={}, task_type='{}'",
                scored_rule.tier,
                scored_rule.priority,
                scored_rule.rule.task_type
            );

            for preferred_agent_id in &scored_rule.rule.preferred_agents {
                if let Some(agent) = available_agents
                    .iter()
                    .find(|a| &a.id == preferred_agent_id)
                {
                    tracing::info!(
                        "✅ Selected agent '{}' via rule (tier={:?}, priority={})",
                        agent.id,
                        scored_rule.tier,
                        scored_rule.priority
                    );
                    return Ok(agent);
                } else {
                    tracing::debug!(
                        "⏭️  Preferred agent '{}' not available, trying next",
                        preferred_agent_id
                    );
                }
            }
        }

        // No rule found an available agent, fallback to priority
        tracing::debug!("⚠️  No rule matched available agents, falling back");
        Self::fallback_by_priority(available_agents)
    }

    /// Check if a rule matches the task
    fn rule_matches(&self, rule: &RoutingRule, task_type: &str, prompt: &str) -> bool {
        // Check task_type
        if rule.task_type != task_type {
            return false;
        }

        // Check keywords (if any)
        if rule.keywords.is_empty() {
            return true;
        }

        let prompt_lower = prompt.to_lowercase();
        rule.keywords.iter().any(|keyword| {
            prompt_lower.contains(&keyword.to_lowercase())
        })
    }

    /// Fallback: select highest priority available agent
    fn fallback_by_priority<'a>(
        available_agents: &'a [&'a AgentConfig]
    ) -> Result<&'a AgentConfig> {
        available_agents
            .iter()
            .max_by_key(|a| a.priority)
            .copied()
            .ok_or_else(|| anyhow::anyhow!("No available agents"))
            .inspect(|agent| {
                tracing::info!(
                    "✅ Selected agent '{}' by priority fallback (priority={})",
                    agent.id,
                    agent.priority
                );
            })
    }

    /// Get agents that match task requirements
    pub fn filter_capable_agents<'a>(
        &self,
        task_type: &str,
        all_agents: &'a [&'a AgentConfig],
    ) -> Vec<&'a AgentConfig> {
        // Extract capabilities from matching rules
        let required_capabilities: Vec<String> = self.routing_config
            .rules
            .iter()
            .filter(|rule| rule.enabled && rule.task_type == task_type)
            .flat_map(|rule| {
                rule.preferred_agents.iter().filter_map(|agent_id| {
                    all_agents
                        .iter()
                        .find(|a| &a.id == agent_id)
                        .and_then(|a| a.capabilities.first().cloned())
                })
            })
            .collect();

        if required_capabilities.is_empty() {
            return all_agents.to_vec();
        }

        all_agents
            .iter()
            .filter(|agent| {
                agent.capabilities.iter().any(|cap| {
                    required_capabilities.contains(cap)
                })
            })
            .copied()
            .collect()
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
            agent_type: "cli".to_string(),
            command: Some("test".to_string()),
            args: None,
            extension_name: None,
            rate_limit: RateLimit::default(),
            capabilities: capabilities.iter().map(|s| s.to_string()).collect(),
            priority,
            enabled: true,
            requires: Vec::new(),
            cost: 0,
        }
    }

    #[test]
    fn test_tiered_priority() {
        let routing_config = RoutingConfig {
            tier: RoutingTier::User,
            rules: vec![
                RoutingRule {
                    task_type: "test".to_string(),
                    keywords: vec![],
                    preferred_agents: vec!["high-priority".to_string()],
                    priority: 900,
                    enabled: true,
                },
                RoutingRule {
                    task_type: "test".to_string(),
                    keywords: vec![],
                    preferred_agents: vec!["low-priority".to_string()],
                    priority: 100,
                    enabled: true,
                },
            ],
        };

        let router = AgentRouter::new(routing_config);

        let agents = vec![
            create_test_agent("high-priority", vec!["test"], 1),
            create_test_agent("low-priority", vec!["test"], 2),
        ];

        let agent_refs: Vec<&AgentConfig> = agents.iter().collect();

        let selected = router
            .select_agent("test", "any prompt", &agent_refs)
            .unwrap();

        // Should select high-priority rule first
        assert_eq!(selected.id, "high-priority");
    }

    #[test]
    fn test_keyword_matching() {
        let routing_config = RoutingConfig {
            tier: RoutingTier::Default,
            rules: vec![
                RoutingRule {
                    task_type: "code".to_string(),
                    keywords: vec!["rust".to_string()],
                    preferred_agents: vec!["rust-agent".to_string()],
                    priority: 500,
                    enabled: true,
                },
            ],
        };

        let router = AgentRouter::new(routing_config);

        let agents = vec![
            create_test_agent("rust-agent", vec!["code"], 1),
        ];

        let agent_refs: Vec<&AgentConfig> = agents.iter().collect();

        // Should match with "rust" keyword
        let selected = router
            .select_agent("code", "write rust code", &agent_refs)
            .unwrap();
        assert_eq!(selected.id, "rust-agent");

        // Should NOT match without keyword
        let result = router
            .select_agent("code", "write python code", &agent_refs);
        
        // Falls back to priority
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_proposal() {
        let routing_config = RoutingConfig {
            tier: RoutingTier::Default,
            rules: vec![],
        };
        let router = AgentRouter::new(routing_config);

        let agent = create_test_agent("test-agent", vec!["test"], 1);
        let state = AgentState {
            config: agent.clone(),
            availability: AgentAvailability::Ready,
        };

        let states = vec![&state];
        let proposal = router.create_proposal("test", "hello", &states).unwrap();

        assert_eq!(proposal.agent_id, "test-agent");
        assert_eq!(proposal.cost_category, CostCategory::Free);
        assert!(proposal.reasoning.contains("Selected via priority fallback"));
        assert_eq!(proposal.availability, "Ready");
    }
}
