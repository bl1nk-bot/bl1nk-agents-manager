use crate::agents::auditor::{create_auditor_agent, AUDITOR_PROMPT_METADATA};
use crate::agents::consultant::{create_consultant_agent, CONSULTANT_PROMPT_METADATA};
use crate::agents::expert::{create_expert_agent, EXPERT_PROMPT_METADATA};
use crate::agents::explorer::{create_explorer_agent, EXPLORER_PROMPT_METADATA};
use crate::agents::manager::{create_manager_agent, MANAGER_PROMPT_METADATA};
use crate::agents::observer::{create_observer_agent, OBSERVER_PROMPT_METADATA};
use crate::agents::orchestrator::create_orchestrator_agent;
use crate::agents::orchestrator_junior::create_orchestrator_junior_agent_with_overrides;
use crate::agents::planner::{create_planner_agent, PLANNER_PROMPT_METADATA};
use crate::agents::prompt_builder::AvailableAgent;
use crate::agents::researcher::{create_researcher_agent, RESEARCHER_PROMPT_METADATA};
use crate::agents::types::{
    AgentCategory, AgentConfig, AgentCost, AgentName, AgentOverrideConfig, AgentOverrides,
    AgentPromptMetadata, BuiltinAgentName, DelegationTrigger,
};
use crate::config::extra_types::{CategoriesConfig, GitMasterConfig};
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

type AgentFactory = Box<dyn Fn(&str) -> AgentConfig>;

// Re-using AvailableAgent from prompt_builder

// Agent sources mapping
fn get_agent_sources() -> HashMap<BuiltinAgentName, AgentFactory> {
    let mut sources = HashMap::new();

    sources.insert(
        BuiltinAgentName::Orchestrator,
        Box::new(|model: &str| create_orchestrator_agent(model, None, None, None, None))
            as AgentFactory,
    );
    sources.insert(
        BuiltinAgentName::Expert,
        Box::new(|model: &str| create_expert_agent(model)) as AgentFactory,
    );
    sources.insert(
        BuiltinAgentName::Researcher,
        Box::new(|model: &str| create_researcher_agent(model)) as AgentFactory,
    );
    sources.insert(
        BuiltinAgentName::Explorer,
        Box::new(|model: &str| create_explorer_agent(model)) as AgentFactory,
    );
    sources.insert(
        BuiltinAgentName::Observer,
        Box::new(|model: &str| create_observer_agent(model)) as AgentFactory,
    );
    sources.insert(
        BuiltinAgentName::Consultant,
        Box::new(|model: &str| create_consultant_agent(model)) as AgentFactory,
    );
    sources.insert(
        BuiltinAgentName::Auditor,
        Box::new(|model: &str| create_auditor_agent(model)) as AgentFactory,
    );

    sources.insert(
        BuiltinAgentName::Manager,
        Box::new(|model: &str| {
            let ctx = crate::agents::manager::ManagerContext {
                model: Some(model.to_string()),
                available_agents: None,
                available_skills: None,
                user_categories: None,
            };
            create_manager_agent(&ctx)
        }) as AgentFactory,
    );

    sources.insert(
        BuiltinAgentName::Planner,
        Box::new(|model: &str| create_planner_agent(model)) as AgentFactory,
    );
    sources.insert(
        BuiltinAgentName::OrchestratorJunior,
        Box::new(|model: &str| create_orchestrator_junior_agent_with_overrides(None, Some(model)))
            as AgentFactory,
    );

    sources
}

/// Metadata for each agent, used to build Orchestrator's dynamic prompt sections
/// (Delegation Table, Tool Selection, Key Triggers, etc.)
fn get_agent_metadata() -> HashMap<BuiltinAgentName, AgentPromptMetadata> {
    let mut metadata = HashMap::new();

    metadata.insert(BuiltinAgentName::Expert, EXPERT_PROMPT_METADATA.clone());
    metadata.insert(
        BuiltinAgentName::Researcher,
        RESEARCHER_PROMPT_METADATA.clone(),
    );
    metadata.insert(BuiltinAgentName::Explorer, EXPLORER_PROMPT_METADATA.clone());
    metadata.insert(BuiltinAgentName::Observer, OBSERVER_PROMPT_METADATA.clone());
    metadata.insert(BuiltinAgentName::Planner, PLANNER_PROMPT_METADATA.clone());
    metadata.insert(
        BuiltinAgentName::Consultant,
        CONSULTANT_PROMPT_METADATA.clone(),
    );
    metadata.insert(BuiltinAgentName::Auditor, AUDITOR_PROMPT_METADATA.clone());
    metadata.insert(BuiltinAgentName::Manager, MANAGER_PROMPT_METADATA.clone());
    metadata.insert(
        BuiltinAgentName::OrchestratorJunior,
        AgentPromptMetadata {
            category: AgentCategory::Specialist,
            cost: AgentCost::Cheap,
            triggers: vec![DelegationTrigger {
                domain: "Focused Task Execution".to_string(),
                trigger: "When a task is well-defined and requires no delegation.".to_string(),
            }],
            use_when: Some(vec![
                "Task is atomic".to_string(),
                "No delegation needed".to_string(),
            ]),
            avoid_when: Some(vec!["Task requires multi-agent coordination".to_string()]),
            dedicated_section: None,
            prompt_alias: Some("Orchestrator-Junior".to_string()),
            key_trigger: None,
        },
    );

    metadata
}

pub fn build_agent(
    source: &AgentFactory,
    model: &str,
    _categories: Option<&CategoriesConfig>,
    _git_master_config: Option<&GitMasterConfig>,
) -> AgentConfig {
    // Simplified version - actual implementation would depend on the structure of CategoriesConfig
    // and GitMasterConfig
    source(model)
}

/// Creates OmO-specific environment context (time, timezone, locale).
pub fn create_env_context() -> String {
    use chrono::Local;

    let now = Local::now();
    let timezone = now.offset().to_string();
    let locale = "en-US"; // Simplified

    let date_str = now.format("%a, %e %b %Y").to_string();
    let time_str = now.format("%I:%M:%S %p").to_string();

    format!(
        r####"
<omo-env>
  Current date: {}
  Current time: {}
  Timezone: {}
  Locale: {}
</omo-env>"####,
        date_str, time_str, timezone, locale
    )
}

fn merge_agent_config(base: &AgentConfig, override_config: &AgentOverrideConfig) -> AgentConfig {
    let mut merged = base.clone();

    if let Some(ref desc) = override_config.partial_agent_config.description {
        merged.description = Some(desc.clone());
    }

    if let Some(ref model) = override_config.partial_agent_config.model {
        merged.model = Some(model.clone());
    }

    if let Some(ref temp) = override_config.partial_agent_config.temperature {
        merged.temperature = Some(*temp);
    }

    if let Some(ref max_tokens) = override_config.partial_agent_config.max_tokens {
        merged.max_tokens = Some(*max_tokens);
    }

    if let Some(ref prompt) = override_config.partial_agent_config.prompt {
        merged.prompt = Some(prompt.clone());
    }

    if let Some(ref prompt_append) = override_config.prompt_append {
        if let Some(ref mut base_prompt) = merged.prompt {
            base_prompt.push('\n');
            base_prompt.push_str(prompt_append);
        }
    }

    if let Some(ref color) = override_config.partial_agent_config.color {
        merged.color = Some(color.clone());
    }

    if let Some(ref permission) = override_config.partial_agent_config.permission {
        merged.permission = Some(permission.clone());
    }

    if let Some(ref mode) = override_config.partial_agent_config.mode {
        merged.mode = Some(mode.clone());
    }

    if let Some(ref thinking) = override_config.partial_agent_config.thinking {
        merged.thinking = Some(thinking.clone());
    }

    if let Some(ref effort) = override_config.partial_agent_config.reasoning_effort {
        merged.reasoning_effort = Some(effort.clone());
    }

    if let Some(ref verbosity) = override_config.partial_agent_config.text_verbosity {
        merged.text_verbosity = Some(verbosity.clone());
    }

    if let Some(ref skills) = override_config.skills {
        merged.skills = Some(skills.clone());
    }

    merged
}

pub async fn create_builtin_agents(
    disabled_agents: &[String],
    agent_overrides: &AgentOverrides,
    directory: Option<&str>,
    system_default_model: Option<&str>,
    categories: Option<&CategoriesConfig>,
    git_master_config: Option<&GitMasterConfig>,
) -> Result<HashMap<String, AgentConfig>> {
    let mut result = HashMap::new();
    let mut available_agents = Vec::new();

    // 1. Discover External Agents from Root
    if let Some(dir) = directory {
        let root_agents_dir = Path::new(dir).join("agents");
        if root_agents_dir.exists() {
            let agents_json_path = root_agents_dir.join("agents.json");
            if let Ok(content) = std::fs::read_to_string(agents_json_path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(agents_list) = json.get("agents").and_then(|a| a.as_array()) {
                        for agent_val in agents_list {
                            let id = agent_val
                                .get("id")
                                .and_then(|v| v.as_str())
                                .unwrap_or_default();
                            let name = agent_val.get("name").and_then(|v| v.as_str()).unwrap_or(id);
                            let desc = agent_val
                                .get("description")
                                .and_then(|v| v.as_str())
                                .unwrap_or_default();
                            let cat_str = agent_val
                                .get("category")
                                .and_then(|v| v.as_str())
                                .unwrap_or("utility");

                            let category = match cat_str {
                                "engineering" => AgentCategory::Exploration,
                                "specialist" => AgentCategory::Specialist,
                                "comedy" | "entertainment" => AgentCategory::Advisor,
                                _ => AgentCategory::Utility,
                            };

                            available_agents.push(AvailableAgent {
                                name: AgentName::Custom(id.to_string()),
                                description: desc.to_string(),
                                metadata: AgentPromptMetadata {
                                    category,
                                    cost: AgentCost::Cheap,
                                    triggers: vec![],
                                    use_when: None,
                                    avoid_when: None,
                                    dedicated_section: None,
                                    prompt_alias: Some(name.to_string()),
                                    key_trigger: None,
                                },
                            });
                        }
                    }
                }
            }
        }
    }

    let agent_sources = get_agent_sources();
    let agent_metadata = get_agent_metadata();

    for (agent_name, source_fn) in agent_sources.iter() {
        let name_str = agent_name.to_string();

        if name_str == "orchestrator"
            || name_str == "manager"
            || disabled_agents.contains(&name_str)
        {
            continue;
        }

        let override_opt = agent_overrides.overrides.get(&name_str);

        let model = if let Some(override_cfg) = override_opt {
            if let Some(ref override_model) = override_cfg.partial_agent_config.model {
                override_model.clone()
            } else {
                system_default_model.unwrap_or("default-model").to_string()
            }
        } else {
            system_default_model.unwrap_or("default-model").to_string()
        };

        let mut config = build_agent(source_fn, &model, categories, git_master_config);

        if name_str == "researcher" {
            if let Some(_dir) = directory {
                if let Some(ref mut prompt) = config.prompt {
                    let env_context = create_env_context();
                    prompt.push_str(&env_context);
                }
            }
        }

        if let Some(override_cfg) = override_opt {
            config = merge_agent_config(&config, override_cfg);
        }

        result.insert(name_str.clone(), config.clone());

        if let Some(metadata) = agent_metadata.get(agent_name) {
            available_agents.push(AvailableAgent {
                name: AgentName::Builtin(agent_name.clone()),
                description: config.description.clone().unwrap_or_default(),
                metadata: metadata.clone(),
            });
        }
    }

    if !disabled_agents.contains(&"orchestrator".to_string()) {
        let orchestrator_override = agent_overrides.overrides.get("orchestrator");

        let orchestrator_model = if let Some(override_cfg) = orchestrator_override {
            if let Some(ref override_model) = override_cfg.partial_agent_config.model {
                override_model.clone()
            } else {
                system_default_model.unwrap_or("default-model").to_string()
            }
        } else {
            system_default_model.unwrap_or("default-model").to_string()
        };

        let orchestrator_config = create_orchestrator_agent(
            &orchestrator_model,
            Some(&available_agents),
            None,
            None,
            None,
        );

        result.insert("orchestrator".to_string(), orchestrator_config);
    }

    if !disabled_agents.contains(&"manager".to_string()) {
        let manager_override = agent_overrides.overrides.get("manager");

        let manager_model = if let Some(override_cfg) = manager_override {
            if let Some(ref override_model) = override_cfg.partial_agent_config.model {
                override_model.clone()
            } else {
                system_default_model.unwrap_or("default-model").to_string()
            }
        } else {
            system_default_model.unwrap_or("default-model").to_string()
        };

        let manager_context = crate::agents::manager::ManagerContext {
            model: Some(manager_model),
            available_agents: Some(available_agents.clone()),
            available_skills: Some(vec![]),
            user_categories: None, // We don't have CategoriesConfig to HashMap mapping yet
        };

        let manager_config = create_manager_agent(&manager_context);

        result.insert("manager".to_string(), manager_config);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_env_context() {
        let context = create_env_context();
        assert!(context.contains("<omo-env>"));
        assert!(context.contains("Current date:"));
        assert!(context.contains("Current time:"));
    }
}
