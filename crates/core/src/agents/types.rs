use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Equivalent to AgentConfig from the TypeScript version

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub agent_type: String,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub extension_name: Option<String>,
    #[serde(default)]
    pub rate_limit: RateLimit,
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub priority: u8,
    #[serde(default)]
    pub enabled: bool,
    // Additional fields that might be in AgentConfig
    pub description: Option<String>,
    pub model: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<usize>,
    pub prompt: Option<String>,
    pub color: Option<String>,
    pub permission: Option<HashMap<String, String>>,
    pub mode: Option<String>,
    pub thinking: Option<ThinkingConfig>,
    pub reasoning_effort: Option<String>,
    pub text_verbosity: Option<String>, // Added for Auditor
    pub skills: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    #[serde(default = "default_rpm")]
    pub requests_per_minute: u32,
    #[serde(default = "default_rpd")]
    pub requests_per_day: u32,
}

fn default_rpm() -> u32 {
    60
}
fn default_rpd() -> u32 {
    2000
}

impl Default for RateLimit {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            requests_per_day: 2000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingConfig {
    #[serde(rename = "type")]
    pub thinking_type: String,
    #[serde(rename = "budgetTokens")]
    pub budget_tokens: Option<i32>,
}

/// Agent category for grouping in Orchestrator prompt sections
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentCategory {
    #[serde(rename = "exploration")]
    Exploration,
    #[serde(rename = "specialist")]
    Specialist,
    #[serde(rename = "advisor")]
    Advisor,
    #[serde(rename = "utility")]
    Utility,
}

/// Cost classification for Tool Selection table
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgentCost {
    #[serde(rename = "FREE")]
    Free,
    #[serde(rename = "CHEAP")]
    Cheap,
    #[serde(rename = "EXPENSIVE")]
    Expensive,
}

/// Delegation trigger for Orchestrator prompt's Delegation Table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationTrigger {
    /// Domain of work (e.g., "Frontend UI/UX")
    pub domain: String,
    /// When to delegate (e.g., "Visual changes only...")
    pub trigger: String,
}

/// Metadata for generating Orchestrator prompt sections dynamically
/// This allows adding/removing agents without manually updating the Orchestrator prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPromptMetadata {
    /// Category for grouping in prompt sections
    pub category: AgentCategory,

    /// Cost classification for Tool Selection table
    pub cost: AgentCost,

    /// Domain triggers for Delegation Table
    pub triggers: Vec<DelegationTrigger>,

    /// When to use this agent (for detailed sections)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_when: Option<Vec<String>>,

    /// When NOT to use this agent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avoid_when: Option<Vec<String>>,

    /// Optional dedicated prompt section (markdown) - for agents like Expert that have special sections
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dedicated_section: Option<String>,

    /// Nickname/alias used in prompt (e.g., "Expert" instead of "oracle")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_alias: Option<String>,

    /// Key triggers that should appear in Phase 0 (e.g., "External library mentioned → fire researcher")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_trigger: Option<String>,
}

/// Check if a model is a GPT model
pub fn is_gpt_model(model: &str) -> bool {
    model.starts_with("openai/") || model.starts_with("github-copilot/gpt-")
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum BuiltinAgentName {
    #[serde(rename = "orchestrator")]
    Orchestrator,
    #[serde(rename = "planner")]
    Planner,
    #[serde(rename = "consultant")]
    Consultant,
    #[serde(rename = "expert")]
    Expert,
    #[serde(rename = "researcher")]
    Researcher,
    #[serde(rename = "explorer")]
    Explorer,
    #[serde(rename = "observer")]
    Observer,
    #[serde(rename = "auditor")]
    Auditor,
    #[serde(rename = "manager")]
    Manager,
    #[serde(rename = "orchestrator-junior")]
    OrchestratorJunior,
}

impl std::fmt::Display for BuiltinAgentName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuiltinAgentName::Orchestrator => write!(f, "orchestrator"),
            BuiltinAgentName::Planner => write!(f, "planner"),
            BuiltinAgentName::Consultant => write!(f, "consultant"),
            BuiltinAgentName::Expert => write!(f, "expert"),
            BuiltinAgentName::Researcher => write!(f, "researcher"),
            BuiltinAgentName::Explorer => write!(f, "explorer"),
            BuiltinAgentName::Observer => write!(f, "observer"),
            BuiltinAgentName::Auditor => write!(f, "auditor"),
            BuiltinAgentName::Manager => write!(f, "manager"),
            BuiltinAgentName::OrchestratorJunior => write!(f, "orchestrator-junior"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OverridableAgentName {
    #[serde(rename = "build")]
    Build,
    Builtin(BuiltinAgentName),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum AgentName {
    Builtin(BuiltinAgentName),
    Custom(String),
}

impl std::fmt::Display for AgentName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentName::Builtin(b) => write!(f, "{}", b),
            AgentName::Custom(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOverrideConfig {
    #[serde(flatten)]
    pub partial_agent_config: PartialAgentConfig,
    #[serde(rename = "prompt_append")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_append: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skills: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<HashMap<String, bool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_options: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartialAgentConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<ThinkingConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_verbosity: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentOverrides {
    #[serde(flatten)]
    pub overrides: HashMap<String, AgentOverrideConfig>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            agent_type: String::new(),
            command: None,
            args: None,
            extension_name: None,
            rate_limit: RateLimit::default(),
            capabilities: vec![],
            priority: 0,
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
}
