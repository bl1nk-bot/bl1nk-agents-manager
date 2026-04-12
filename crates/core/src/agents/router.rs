use crate::agents::types::AgentConfig;
use crate::config::{RoutingConfig, RoutingRule, RoutingTier};
use anyhow::Result;
use std::cmp::Ordering;

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

    /// Select the best agent using tiered priority system
    pub fn select_agent<'a>(
        &self,
        task_type: &str,
        prompt: &str,
        available_agents: &'a [&'a AgentConfig],
    ) -> Result<&'a AgentConfig> {
        tracing::debug!("🔍 Router: Selecting agent for task_type='{}'", task_type);
        tracing::debug!(
            "📝 Prompt: {}",
            prompt.chars().take(100).collect::<String>()
        );

        // Find all matching rules
        let matching_rules: Vec<ScoredRule> = self
            .routing_config
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
        rule.keywords
            .iter()
            .any(|keyword| prompt_lower.contains(&keyword.to_lowercase()))
    }

    /// Fallback: select highest priority available agent
    fn fallback_by_priority<'a>(
        available_agents: &'a [&'a AgentConfig],
    ) -> Result<&'a AgentConfig> {
        let agent = available_agents
            .iter()
            .max_by_key(|a| a.priority)
            .copied()
            .ok_or_else(|| anyhow::anyhow!("No available agents"))?;

        tracing::info!(
            "✅ Selected agent '{}' by priority fallback (priority={})",
            agent.id,
            agent.priority
        );

        Ok(agent)
    }

    /// Get agents that match task requirements
    pub fn filter_capable_agents<'a>(
        &self,
        task_type: &str,
        all_agents: &'a [&'a AgentConfig],
    ) -> Vec<&'a AgentConfig> {
        // Extract capabilities from matching rules
        let required_capabilities: Vec<String> = self
            .routing_config
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
                agent
                    .capabilities
                    .iter()
                    .any(|cap| required_capabilities.contains(cap))
            })
            .copied()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::types::RateLimit;

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
            description: None,
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
            rules: vec![RoutingRule {
                task_type: "code".to_string(),
                keywords: vec!["rust".to_string()],
                preferred_agents: vec!["rust-agent".to_string()],
                priority: 500,
                enabled: true,
            }],
        };

        let router = AgentRouter::new(routing_config);

        let agents = vec![create_test_agent("rust-agent", vec!["code"], 1)];

        let agent_refs: Vec<&AgentConfig> = agents.iter().collect();

        // Should match with "rust" keyword
        let selected = router
            .select_agent("code", "write rust code", &agent_refs)
            .unwrap();
        assert_eq!(selected.id, "rust-agent");

        // Should NOT match without keyword
        let result = router.select_agent("code", "write python code", &agent_refs);

        // Falls back to priority
        assert!(result.is_ok());
    }
}
