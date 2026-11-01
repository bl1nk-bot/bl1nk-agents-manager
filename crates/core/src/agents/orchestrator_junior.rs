use crate::agents::types::{is_gpt_model, AgentConfig, AgentOverrideConfig};
use std::collections::HashMap;

pub fn create_orchestrator_junior_agent(model: &str) -> AgentConfig {
    create_orchestrator_junior_agent_with_overrides(None, Some(model))
}

const ORCHESTRATOR_JUNIOR_PROMPT: &str = r####"<Role>
Orchestrator-Junior - Focused executor from Bl1nk.
Execute tasks directly. NEVER delegate or spawn other agents.
</Role>

<Critical_Constraints>
BLOCKED ACTIONS (will fail if attempted):
- task tool: BLOCKED
- delegate_task tool: BLOCKED

ALLOWED: call_omo_agent - You CAN spawn explore/librarian agents for research.
You work ALONE for implementation. No delegation of implementation tasks.
</Critical_Constraints>

<Todo_Discipline>
TODO OBSESSION (NON-NEGOTIABLE):
- 2+ steps → todowrite FIRST, atomic breakdown
- Mark in_progress before starting (ONE at a time)
- Mark completed IMMEDIATELY after each step
- NEVER batch completions

No todos on multi-step work = INCOMPLETE WORK.
</Todo_Discipline>

<Verification>
Task NOT complete without:
- lsp_diagnostics clean on changed files
- Build passes (if applicable)
- All todos marked completed
</Verification>

<Style>
- Start immediately. No acknowledgments.
- Match user's communication style.
- Dense > verbose.
</Style>"####;

fn build_orchestrator_junior_prompt(prompt_append: Option<&str>) -> String {
    if let Some(append) = prompt_append {
        format!("{}\n\n{}", ORCHESTRATOR_JUNIOR_PROMPT, append)
    } else {
        ORCHESTRATOR_JUNIOR_PROMPT.to_string()
    }
}

// Core tools that Sisyphus-Junior must NEVER have access to
// Note: call_omo_agent is ALLOWED so subagents can spawn explore/librarian
const BLOCKED_TOOLS: &[&str] = &["task", "delegate_task"];

#[derive(Debug, Clone)]
pub struct OrchestratorJuniorDefaults {
    pub model: &'static str,
    pub temperature: f32,
}

impl Default for OrchestratorJuniorDefaults {
    fn default() -> Self {
        OrchestratorJuniorDefaults {
            model: "anthropic/claude-sonnet-4-5",
            temperature: 0.1,
        }
    }
}

pub fn create_orchestrator_junior_agent_with_overrides(
    override_config: Option<&AgentOverrideConfig>,
    system_default_model: Option<&str>,
) -> AgentConfig {
    let defaults = OrchestratorJuniorDefaults::default();

    let model = override_config
        .and_then(|o| o.partial_agent_config.model.as_ref())
        .map(|m| m.as_str())
        .or(system_default_model)
        .unwrap_or(defaults.model)
        .to_string();

    let temperature = override_config
        .and_then(|o| o.partial_agent_config.temperature)
        .unwrap_or(defaults.temperature);

    let prompt_append = override_config
        .and_then(|o| o.prompt_append.as_ref())
        .map(|s| s.as_str());
    let prompt = build_orchestrator_junior_prompt(prompt_append);

    let base_restrictions = create_agent_tool_restrictions(BLOCKED_TOOLS);

    // Merge permissions
    let mut merged_permissions = HashMap::new();

    // Add user permissions if they exist
    if let Some(override_cfg) = override_config {
        if let Some(ref perm) = override_cfg.partial_agent_config.permission {
            merged_permissions.extend(perm.clone());
        }
    }

    // Ensure blocked tools are denied
    for tool in BLOCKED_TOOLS {
        merged_permissions.insert(tool.to_string(), "deny".to_string());
    }

    // Ensure call_omo_agent is allowed
    merged_permissions.insert("call_omo_agent".to_string(), "allow".to_string());

    // Add base restrictions
    if let Some(ref base_perm) = base_restrictions.permission {
        merged_permissions.extend(base_perm.clone());
    }

    let base = AgentConfig {
        description: Some(
            override_config
                .and_then(|o| o.partial_agent_config.description.as_ref())
                .cloned()
                .unwrap_or_else(|| {
                    "Orchestrator-Junior - Focused task executor. Same discipline, no delegation."
                        .to_string()
                }),
        ),
        mode: Some("subagent".to_string()),
        model: Some(model),
        temperature: Some(temperature),
        max_tokens: Some(64000),
        prompt: Some(prompt),
        color: Some(
            override_config
                .and_then(|o| o.partial_agent_config.color.as_ref())
                .cloned()
                .unwrap_or_else(|| "####20B2AA".to_string()),
        ),
        permission: Some(merged_permissions),
        id: "orchestrator-junior".to_string(),
        name: "Orchestrator-Junior".to_string(),
        agent_type: "omo".to_string(),
        command: None,
        args: None,
        extension_name: None,
        rate_limit: crate::agents::types::RateLimit::default(),
        capabilities: vec!["execution".to_string(), "focused-task".to_string()],
        priority: 75,
        enabled: true,
        thinking: None,
        reasoning_effort: None,
        text_verbosity: None,
        skills: None,
    };

    if is_gpt_model(&base.model.clone().unwrap_or_default()) {
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

fn create_agent_tool_restrictions(blocked_tools: &[&str]) -> AgentConfig {
    let mut permission = HashMap::new();

    // Deny the blocked tools
    for tool in blocked_tools {
        permission.insert(tool.to_string(), "deny".to_string());
    }

    AgentConfig {
        permission: Some(permission),
        ..Default::default()
    }
}
