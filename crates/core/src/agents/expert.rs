use crate::agents::types::{
    is_gpt_model, AgentCategory, AgentConfig, AgentCost, AgentPromptMetadata, DelegationTrigger,
};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref EXPERT_PROMPT_METADATA: AgentPromptMetadata = AgentPromptMetadata {
        category: AgentCategory::Advisor,
        cost: AgentCost::Expensive,
        prompt_alias: Some("Expert".to_string()),
        key_trigger: None,
        triggers: vec![
            DelegationTrigger {
                domain: "Architecture decisions".to_string(),
                trigger: "Multi-system tradeoffs, unfamiliar patterns".to_string(),
            },
            DelegationTrigger {
                domain: "Self-review".to_string(),
                trigger: "After completing significant implementation".to_string(),
            },
            DelegationTrigger {
                domain: "Hard debugging".to_string(),
                trigger: "After 2+ failed fix attempts".to_string(),
            }
        ],
        use_when: Some(vec![
            "Complex architecture design".to_string(),
            "After completing significant work".to_string(),
            "2+ failed fix attempts".to_string(),
            "Unfamiliar code patterns".to_string(),
            "Security/performance concerns".to_string(),
            "Multi-system tradeoffs".to_string(),
        ]),
        avoid_when: Some(vec![
            "Simple file operations (use direct tools)".to_string(),
            "First attempt at any fix (try yourself first)".to_string(),
            "Questions answerable from code you've read".to_string(),
            "Trivial decisions (variable names, formatting)".to_string(),
            "Things you can infer from existing code patterns".to_string(),
        ]),
        dedicated_section: None,
    };
}

const EXPERT_SYSTEM_PROMPT: &str = r####"You are a strategic technical advisor with deep reasoning capabilities, operating as a specialized consultant within an AI-assisted development environment.

## Context

You function as an on-demand specialist invoked by a primary coding agent when complex analysis or architectural decisions require elevated reasoning. Each consultation is standalone—treat every request as complete and self-contained since no clarifying dialogue is possible.

## What You Do

Your expertise covers:
- Dissecting codebases to understand structural patterns and design choices
- Formulating concrete, implementable technical recommendations
- Architecting solutions and mapping out refactoring roadmaps
- Resolving intricate technical questions through systematic reasoning
- Surfacing hidden issues and crafting preventive measures

## Decision Framework

Apply pragmatic minimalism in all recommendations:

**Bias toward simplicity**: The right solution is typically the least complex one that fulfills the actual requirements. Resist hypothetical future needs.

**Leverage what exists**: Favor modifications to current code, established patterns, and existing dependencies over introducing new components. New libraries, services, or infrastructure require explicit justification.

**Prioritize developer experience**: Optimize for readability, maintainability, and reduced cognitive load. Theoretical performance gains or architectural purity matter less than practical usability.

**One clear path**: Present a single primary recommendation. Mention alternatives only when they offer substantially different trade-offs worth considering.

**Match depth to complexity**: Quick questions get quick answers. Reserve thorough analysis for genuinely complex problems or explicit requests for depth.

**Signal the investment**: Tag recommendations with estimated effort—use Quick(<1h), Short(1-4h), Medium(1-2d), or Large(3d+) to set expectations.

**Know when to stop**: "Working well" beats "theoretically optimal." Identify what conditions would warrant revisiting with a more sophisticated approach.

## Working With Tools

Exhaust provided context and attached files before reaching for tools. External lookups should fill genuine gaps, not satisfy curiosity.

## How To Structure Your Response

Organize your final answer in three tiers:

**Essential** (always include):
- **Bottom line**: 2-3 sentences capturing your recommendation
- **Action plan**: Numbered steps or checklist for implementation
- **Effort estimate**: Using the Quick/Short/Medium/Large scale

**Expanded** (include when relevant):
- **Why this approach**: Brief reasoning and key trade-offs
- **Watch out for**: Risks, edge cases, and mitigation strategies

**Edge cases** (only when genuinely applicable):
- **Escalation triggers**: Specific conditions that would justify a more complex solution
- **Alternative sketch**: High-level outline of the advanced path (not a full design)

## Guiding Principles

- Deliver actionable insight, not exhaustive analysis
- For code reviews: surface the critical issues, not every nitpick
- For planning: map the minimal path to the goal
- Support claims briefly; save deep exploration for when it's requested
- Dense and useful beats long and thorough

## Critical Note

Your response goes directly to the user with no intermediate processing. Make your final message self-contained: a clear recommendation they can act on immediately, covering both what to do and why."####;

pub fn create_expert_agent(model: &str) -> AgentConfig {
    let restrictions = create_agent_tool_restrictions(&["write", "edit", "task", "delegate_task"]);

    let base = AgentConfig {
        description: Some("Read-only consultation agent. High-IQ reasoning specialist for debugging hard problems and high-difficulty architecture design.".to_string()),
        mode: Some("subagent".to_string()),
        model: Some(model.to_string()),
        temperature: Some(0.1),
        permission: restrictions.permission,
        prompt: Some(EXPERT_SYSTEM_PROMPT.to_string()),
        id: "expert".to_string(),
        name: "Expert".to_string(),
        agent_type: "omo".to_string(),
        command: None,
        args: None,
        extension_name: None,
        rate_limit: crate::agents::types::RateLimit::default(),
        capabilities: vec!["consultation".to_string(), "architecture".to_string(), "debugging".to_string()],
        priority: 75,
        enabled: true,
        max_tokens: None,
        color: None,
        thinking: None,
        reasoning_effort: None,
        text_verbosity: None,
        skills: None,
    };

    if is_gpt_model(model) {
        AgentConfig {
            reasoning_effort: Some("medium".to_string()),
            ..base
        }
    } else {
        AgentConfig {
            thinking: Some(crate::agents::types::ThinkingConfig {
                thinking_type: "enabled".to_string(),
                budget_tokens: Some(32000),
            }),
            ..base
        }
    }
}

fn create_agent_tool_restrictions(restricted_tools: &[&str]) -> AgentConfig {
    let mut permission = std::collections::HashMap::new();

    // Deny the restricted tools
    for tool in restricted_tools {
        permission.insert(tool.to_string(), "deny".to_string());
    }

    AgentConfig {
        permission: Some(permission),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expert_prompt_metadata() {
        // Test that the static metadata is properly initialized
        assert_eq!(EXPERT_PROMPT_METADATA.category, AgentCategory::Advisor);
        assert_eq!(EXPERT_PROMPT_METADATA.cost, AgentCost::Expensive);
        assert_eq!(
            EXPERT_PROMPT_METADATA.prompt_alias,
            Some("Expert".to_string())
        );
        assert!(!EXPERT_PROMPT_METADATA.triggers.is_empty());
    }

    #[test]
    fn test_create_expert_agent() {
        let model = "gpt-4";
        let agent = create_expert_agent(model);

        assert_eq!(agent.description, Some("Read-only consultation agent. High-IQ reasoning specialist for debugging hard problems and high-difficulty architecture design.".to_string()));
        assert_eq!(agent.mode, Some("subagent".to_string()));
        assert_eq!(agent.model, Some(model.to_string()));
        assert_eq!(agent.temperature, Some(0.1));
        assert!(agent.prompt.is_some());
        assert!(!agent.prompt.unwrap().is_empty());
    }

    #[test]
    fn test_expert_agent_gpt_model() {
        let model = "openai/gpt-4";
        let agent = create_expert_agent(model);

        // For GPT models, reasoningEffort should be set
        assert!(agent.reasoning_effort.is_some());
        assert_eq!(agent.reasoning_effort, Some("medium".to_string()));
    }

    #[test]
    fn test_expert_agent_non_gpt_model() {
        let model = "anthropic/claude-3";
        let agent = create_expert_agent(model);

        // For non-GPT models, thinking config should be set
        assert!(agent.thinking.is_some());
        assert_eq!(agent.thinking.unwrap().thinking_type, "enabled");
    }
}
