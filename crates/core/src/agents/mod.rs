pub mod auditor;
pub mod consultant;
pub mod creator;
pub mod expert;
pub mod explorer;
pub mod extractor;
pub mod manager;
pub mod observer;
pub mod orchestrator;
pub mod orchestrator_junior;
pub mod planner;
pub mod prompt_builder;
pub mod register;
pub mod researcher;
pub mod router;
pub mod types;
pub mod utils;

pub use auditor::{create_auditor_agent, AUDITOR_PROMPT_METADATA};
pub use consultant::{create_consultant_agent, CONSULTANT_PROMPT_METADATA};
pub use creator::AgentCreator;
pub use expert::{create_expert_agent, EXPERT_PROMPT_METADATA};
pub use explorer::{create_explorer_agent, EXPLORER_PROMPT_METADATA};
pub use extractor::AgentExecutor;
pub use manager::{create_manager_agent, MANAGER_PROMPT_METADATA};
pub use observer::{create_observer_agent, OBSERVER_PROMPT_METADATA};
pub use orchestrator::create_orchestrator_agent;
pub use orchestrator_junior::create_orchestrator_junior_agent;
pub use planner::{create_planner_agent, PLANNER_PROMPT_METADATA};
pub use prompt_builder::{
    build_anti_patterns_section, build_category_skills_delegation_guide, build_delegation_table,
    build_expert_section, build_explorer_section, build_hard_blocks_section,
    build_key_triggers_section, build_researcher_section, build_tool_selection_table,
    build_ultrawork_section, categorize_tools, AvailableAgent, AvailableCategory, AvailableSkill,
    AvailableTool, SkillLocation, ToolCategory,
};
pub use register::AgentRegistry;
pub use researcher::{create_researcher_agent, RESEARCHER_PROMPT_METADATA};
pub use router::AgentRouter;
pub use types::{
    is_gpt_model, AgentCategory, AgentConfig, AgentCost, AgentName, AgentOverrideConfig,
    AgentOverrides, AgentPromptMetadata, BuiltinAgentName, DelegationTrigger, PartialAgentConfig,
    RateLimit, ThinkingConfig,
};
pub use utils::{build_agent, create_builtin_agents, create_env_context};
